mod config;
mod hooks;
mod install;
mod library;
mod mascot;
mod sound;
mod ui;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
    /// List saved sounds, or save and pick the one that plays on failures.
    Sound {
        #[command(subcommand)]
        action: Option<SoundAction>,
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

#[derive(Subcommand)]
enum SoundAction {
    /// Save a copy of a WAV or MP3 file and play it on failures.
    Add {
        file: PathBuf,
        /// Name to save it under; the file name by default.
        #[arg(long)]
        name: Option<String>,
    },
    /// Play a built-in or saved sound on failures.
    Use { name: String },
    /// Delete a saved sound.
    Remove { name: String },
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
        Command::Sound { action: None } => list_sounds(&config),
        Command::Sound {
            action: Some(SoundAction::Add { file, name }),
        } => add_sound(&mut config, &file, name.as_deref()),
        Command::Sound {
            action: Some(SoundAction::Use { name }),
        } => use_sound(&mut config, &name),
        Command::Sound {
            action: Some(SoundAction::Remove { name }),
        } => remove_sound(&mut config, &name),
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
    sound::play_alone(sound::load(&config.sound))
}

fn test(config: &Config) -> Result<()> {
    let playing = ui::lead(AMBER, "♪", &format!("playing {}…", sound_label(config)));
    let done = ui::lead(
        SAGE,
        "✓",
        "done. Heard nothing? Check your output device and volume.",
    );
    let audio = sound::load(&config.sound);
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

fn list_sounds(config: &Config) -> Result<()> {
    let saved = library::list(&library::dir()?)?;
    let current = sound_label(config);
    println!("{}\n", ui::paint(AMBER.bold(), "♪ sounds"));
    let names = sound::BUILT_INS
        .iter()
        .map(|(name, _)| *name)
        .chain(saved.iter().map(|s| s.name.as_str()));
    let mut current_listed = false;
    for name in names {
        if name == current {
            current_listed = true;
            println!("   {} {name}", ui::paint(SAGE, "●"));
        } else {
            println!("   {} {}", ui::paint(ASH, "·"), ui::paint(ASH, name));
        }
    }
    if !current_listed {
        println!("   {} {current}", ui::paint(SAGE, "●"));
    }
    println!();
    ui::hint("pick one with `faaah sound use <name>` · save one with `faaah sound add <file>`");
    Ok(())
}

fn add_sound(config: &mut Config, file: &Path, name: Option<&str>) -> Result<()> {
    let audio = fs::read(file).with_context(|| format!("can’t read {}", ui::path(file)))?;
    sound::decode(Cow::Owned(audio))?;
    let saved = library::add(&library::dir()?, file, name)?;
    config.sound = saved.path;
    config.save()?;
    ui::line(
        SAGE,
        "✓",
        &format!("saved {} and playing it on failures", saved.name),
    );
    Ok(())
}

fn use_sound(config: &mut Config, name: &str) -> Result<()> {
    config.sound = if sound::built_in(name).is_some() {
        PathBuf::from(name)
    } else {
        library::find(&library::dir()?, name)?.path
    };
    config.save()?;
    ui::line(SAGE, "✓", &format!("playing {name} on failures"));
    Ok(())
}

fn remove_sound(config: &mut Config, name: &str) -> Result<()> {
    let removed = library::remove(&library::dir()?, name)?;
    ui::line(SAGE, "✓", &format!("deleted {name}"));
    if config.sound == removed.path {
        config.sound = PathBuf::from(sound::DEFAULT);
        config.save()?;
        ui::hint(&format!(
            "it was the current sound, so {} plays now",
            sound::DEFAULT
        ));
    }
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

/// Name of a built-in or saved sound, or the path of a file set by hand in the config.
fn sound_label(config: &Config) -> String {
    let path = &config.sound;
    if let Some(name) = path.to_str().filter(|name| sound::built_in(name).is_some()) {
        return name.into();
    }
    if !path.is_file() {
        return format!("{} (missing, using {})", ui::path(path), sound::DEFAULT);
    }
    let saved = library::dir().is_ok_and(|dir| path.parent() == Some(dir.as_path()));
    match path.file_stem().and_then(|stem| stem.to_str()) {
        Some(name) if saved => name.into(),
        _ => ui::path(path),
    }
}
