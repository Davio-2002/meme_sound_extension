mod config;
mod hooks;
mod install;
mod mascot;
mod sound;
mod ui;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::{self, Path, PathBuf};
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use anstream::{print, println};
use anyhow::{Context, Result, bail};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};

use config::Config;
use hooks::Shell;
use ui::{AMBER, ASH, ROSE, SAGE};

#[derive(Parser)]
#[command(
    version,
    about = about(),
    styles = ui::HELP
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Play the sound and report whether audio works.
    Test,
    /// Show or change the sound.
    Sound {
        /// WAV or MP3 file to play instead of the built-in sound.
        file: Option<PathBuf>,
        /// Go back to the built-in sound.
        #[arg(long, conflicts_with = "file")]
        reset: bool,
    },
    /// Turn the sound on.
    On,
    /// Turn the sound off.
    Off,
    /// Show the current settings.
    Status,
    /// Add the hook to shell configs; without arguments, to every shell found.
    Install { shells: Vec<Shell> },
    /// Remove the hook from shell configs; without arguments, from every shell.
    Uninstall { shells: Vec<Shell> },
    /// Print the shell hook to load from the shell config.
    Init { shell: Shell },
    /// Play the sound unless it is turned off.
    Play {
        /// Return immediately and play in a background process.
        #[arg(long)]
        detach: bool,
        /// Exit code of the failed command; codes from `ignore_codes` stay silent.
        #[arg(long)]
        exit_code: Option<i32>,
    },
}

fn main() -> ExitCode {
    let mut command = Cli::command();
    if ui::fancy() {
        command = command.about(mascot::idle().beside(&[about()]).join("\n"));
    }
    let cli = Cli::from_arg_matches(&command.get_matches_mut()).unwrap_or_else(|err| err.exit());
    let Some(cli_command) = cli.command else {
        command.print_help().ok();
        return ExitCode::SUCCESS;
    };
    match run(cli_command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            ui::error(&err);
            ExitCode::FAILURE
        }
    }
}

fn about() -> String {
    ui::paint(AMBER, "♪") + " Plays a meme sound when a terminal command fails."
}

fn run(command: Command) -> Result<()> {
    let mut config = Config::load()?;
    match command {
        Command::Play { detach, exit_code } => play(&config, detach, exit_code),
        Command::Test => test(&config),
        Command::Sound {
            file: Some(file), ..
        } => set_sound(&mut config, &file),
        Command::Sound { reset: true, .. } => reset_sound(&mut config),
        Command::Sound { .. } => {
            println!("{}", sound_label(&config));
            Ok(())
        }
        Command::On => set_muted(&mut config, false),
        Command::Off => set_muted(&mut config, true),
        Command::Status => status(&config),
        Command::Init { shell } => {
            std::print!("{}", shell.hook());
            Ok(())
        }
        Command::Install { shells } => install(shells),
        Command::Uninstall { shells } => uninstall(shells),
    }
}

fn play(config: &Config, detach: bool, exit_code: Option<i32>) -> Result<()> {
    let ignored = exit_code.is_some_and(|code| config.ignore_codes.contains(&code));
    if config.muted || ignored {
        return Ok(());
    }
    if detach {
        return sound::play_in_background();
    }
    sound::play(sound::load(config.sound.as_deref()))
}

fn test(config: &Config) -> Result<()> {
    let playing = ui::lead(AMBER, "♪", &format!("playing {}…", sound_label(config)));
    let done = ui::lead(
        SAGE,
        "✓",
        "done. Heard nothing? Check your output device and volume.",
    );
    let audio = sound::load(config.sound.as_deref());
    if ui::fancy() {
        race(sound::start(audio)?, &playing, &done);
    } else {
        println!("{playing}");
        sound::play(audio)?;
        println!("{done}");
    }
    if config.muted {
        ui::hint("sound is off, hooks stay quiet · run `faaah on`");
    }
    Ok(())
}

/// Redraws the racing mascot in place until the sound ends.
fn race(playback: sound::Playback, playing: &str, done: &str) {
    for frame in 0.. {
        let finished = !playback.is_playing();
        let text = [
            String::new(),
            playing.into(),
            if finished { done } else { "" }.into(),
        ];
        let lines = mascot::car(frame).beside(&text);
        if frame > 0 {
            print!("\x1b[{}A", lines.len());
        }
        for line in lines {
            println!("\r\x1b[2K{line}");
        }
        if finished {
            break;
        }
        thread::sleep(Duration::from_millis(260));
    }
}

fn set_sound(config: &mut Config, file: &Path) -> Result<()> {
    let file = path::absolute(file)?;
    let audio = fs::read(&file).with_context(|| format!("can’t read {}", ui::path(&file)))?;
    sound::decode(Cow::Owned(audio))?;
    config.sound = Some(file);
    config.save()?;
    ui::line(
        SAGE,
        "✓",
        &format!("sound set to {}", ui::paint(ASH, &sound_label(config))),
    );
    Ok(())
}

fn reset_sound(config: &mut Config) -> Result<()> {
    config.sound = None;
    config.save()?;
    ui::line(SAGE, "✓", "sound set to the built-in one");
    Ok(())
}

fn set_muted(config: &mut Config, muted: bool) -> Result<()> {
    config.muted = muted;
    config.save()?;
    if muted {
        ui::line(ROSE, "◌", "sound paused. Hooks stay quiet.");
    } else {
        ui::line(SAGE, "●", "sound is on");
    }
    Ok(())
}

fn status(config: &Config) -> Result<()> {
    let sound = if config.muted {
        (ROSE, "◌ off")
    } else {
        (SAGE, "● on")
    };
    let silent = if config.ignore_codes.is_empty() {
        "none".to_string()
    } else {
        let codes: Vec<_> = config.ignore_codes.iter().map(i32::to_string).collect();
        format!("exit {}", codes.join(", "))
    };
    let hooks = installed_hooks();
    let hooks = if hooks.is_empty() {
        "none".to_string()
    } else {
        hooks.join(", ")
    };
    let plain = anstyle::Style::new();
    let lines = ui::status_box(
        &[
            ("sound", sound.0, sound.1.to_string()),
            ("file", plain, sound_label(config)),
            ("silent", plain, silent),
            ("hooks", plain, hooks),
        ],
        &ui::path(&Config::path()?),
    );
    let lines = if ui::fancy() {
        mascot::lens().beside(&lines)
    } else {
        lines
    };
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

fn install(shells: Vec<Shell>) -> Result<()> {
    let shells = if shells.is_empty() {
        install::detect()
    } else {
        shells
    };
    if shells.is_empty() {
        bail!("no supported shell found\nname one, for example `faaah install bash`");
    }
    println!("{}\n", ui::paint(AMBER.bold(), "♪ setting up hooks"));
    let pad = name_width(&shells);
    for shell in shells {
        for file in install::config_files(shell)? {
            let (glyph, result) = if install::add(shell, &file)? {
                (ui::paint(SAGE, "✓"), "added to  ")
            } else {
                (ui::paint(ASH, "·"), "already in")
            };
            println!(
                "   {glyph} {:<pad$} {result} {}",
                shell.name(),
                ui::paint(ASH, &ui::path(&file))
            );
        }
    }
    println!();
    if !install::is_on_path("faaah") {
        let dir = env::current_exe()?
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        ui::line(ROSE, "⚠", "faaah is not on PATH, so hooks can’t find it.");
        ui::hint(&format!("add `{}` to PATH", ui::path(&dir)));
        println!();
    }
    println!(
        "Open a new terminal and you’re all set {}",
        ui::paint(AMBER, "♪")
    );
    Ok(())
}

fn uninstall(shells: Vec<Shell>) -> Result<()> {
    let shells = if shells.is_empty() {
        Shell::value_variants().to_vec()
    } else {
        shells
    };
    let pad = name_width(&shells);
    let mut removed_any = false;
    for shell in shells {
        let Ok(files) = install::config_files(shell) else {
            continue;
        };
        for file in files {
            if install::remove(shell, &file)? {
                println!(
                    "   {} {:<pad$} removed from {}",
                    ui::paint(SAGE, "✓"),
                    shell.name(),
                    ui::paint(ASH, &ui::path(&file))
                );
                removed_any = true;
            }
        }
    }
    if !removed_any {
        ui::line(ASH, "·", "nothing to remove");
    }
    Ok(())
}

fn name_width(shells: &[Shell]) -> usize {
    shells
        .iter()
        .map(|shell| shell.name().len())
        .max()
        .unwrap_or_default()
}

fn installed_hooks() -> Vec<&'static str> {
    Shell::value_variants()
        .iter()
        .filter(|&&shell| {
            install::config_files(shell)
                .is_ok_and(|files| files.iter().any(|file| install::has_block(shell, file)))
        })
        .map(|shell| shell.name())
        .collect()
}

fn sound_label(config: &Config) -> String {
    match &config.sound {
        None => "built-in".into(),
        Some(path) if path.is_file() => ui::path(path),
        Some(path) => format!("{} (missing, using built-in)", ui::path(path)),
    }
}
