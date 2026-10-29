mod config;
mod sound;

use std::borrow::Cow;
use std::fs;
use std::path::{self, Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use config::Config;

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
}

fn main() -> Result<()> {
    let command = Cli::parse().command;
    let mut config = Config::load()?;
    match command {
        Command::Play { detach } => play(&config, detach),
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
    }
}

fn play(config: &Config, detach: bool) -> Result<()> {
    if config.muted {
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
    println!("Config: {}", Config::path()?.display());
    Ok(())
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
