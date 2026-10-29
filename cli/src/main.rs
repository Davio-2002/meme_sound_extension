mod config;
mod hooks;
mod install;
mod sound;

use std::borrow::Cow;
use std::env;
use std::fs;
use std::path::{self, Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use config::Config;
use hooks::Shell;

/// Plays a meme sound when a terminal command fails.
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Play the sound unless it is turned off.
    Play {
        /// Return immediately and play in a background process.
        #[arg(long)]
        detach: bool,
        /// Exit code of the failed command; codes from `ignore_codes` stay silent.
        #[arg(long)]
        exit_code: Option<i32>,
    },
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
    /// Print the shell hook to load from the shell config.
    Init { shell: Shell },
    /// Add the hook to shell configs; without arguments, to every shell found.
    Install { shells: Vec<Shell> },
    /// Remove the hook from shell configs; without arguments, from every shell.
    Uninstall { shells: Vec<Shell> },
}

fn main() -> Result<()> {
    let command = Cli::parse().command;
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
            print!("{}", shell.hook());
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
    println!("Playing {}...", sound_label(config));
    sound::play(sound::load(config.sound.as_deref()))?;
    println!("Done. If you heard nothing, check the default output device and its volume.");
    if config.muted {
        println!("The sound is off, so hooks stay silent. Run `faaah on` to turn it on.");
    }
    Ok(())
}

fn set_sound(config: &mut Config, file: &Path) -> Result<()> {
    let file = path::absolute(file)?;
    let audio = fs::read(&file).with_context(|| format!("cannot read {}", file.display()))?;
    sound::decode(Cow::Owned(audio))?;
    config.sound = Some(file);
    config.save()?;
    println!("Sound set to {}", sound_label(config));
    Ok(())
}

fn reset_sound(config: &mut Config) -> Result<()> {
    config.sound = None;
    config.save()?;
    println!("Sound set to the built-in one");
    Ok(())
}

fn set_muted(config: &mut Config, muted: bool) -> Result<()> {
    config.muted = muted;
    config.save()?;
    println!("Sound is {}", on_off(muted));
    Ok(())
}

fn status(config: &Config) -> Result<()> {
    println!("Sound:  {}", on_off(config.muted));
    println!("File:   {}", sound_label(config));
    println!("Silent: exit codes {:?}", config.ignore_codes);
    let hooks = installed_hooks();
    let hooks = if hooks.is_empty() {
        "none".to_string()
    } else {
        hooks.join(", ")
    };
    println!("Hooks:  {hooks}");
    println!("Config: {}", Config::path()?.display());
    Ok(())
}

fn install(shells: Vec<Shell>) -> Result<()> {
    let shells = if shells.is_empty() {
        install::detect()
    } else {
        shells
    };
    if shells.is_empty() {
        bail!("no supported shell found; name one, for example `faaah install bash`");
    }
    for shell in shells {
        for file in install::config_files(shell)? {
            let result = if install::add(shell, &file)? {
                "added to"
            } else {
                "already in"
            };
            println!("{}: {result} {}", shell.name(), file.display());
        }
    }
    if !install::is_on_path("faaah") {
        let dir = env::current_exe()?
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        println!(
            "Warning: faaah is not on PATH, so the hooks cannot run it. Add {} to PATH.",
            dir.display()
        );
    }
    println!("Open a new terminal to use it.");
    Ok(())
}

fn uninstall(shells: Vec<Shell>) -> Result<()> {
    let shells = if shells.is_empty() {
        Shell::value_variants().to_vec()
    } else {
        shells
    };
    let mut removed_any = false;
    for shell in shells {
        let Ok(files) = install::config_files(shell) else {
            continue;
        };
        for file in files {
            if install::remove(shell, &file)? {
                println!("{}: removed from {}", shell.name(), file.display());
                removed_any = true;
            }
        }
    }
    if !removed_any {
        println!("Nothing to remove");
    }
    Ok(())
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
        Some(path) if path.is_file() => path.display().to_string(),
        Some(path) => format!("{} (missing, using built-in)", path.display()),
    }
}

fn on_off(muted: bool) -> &'static str {
    if muted { "off" } else { "on" }
}
