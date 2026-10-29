mod sound;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Plays a meme sound when a terminal command fails.
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Play the sound.
    Play {
        /// Return immediately and play in a background process.
        #[arg(long)]
        detach: bool,
    },
    /// Play the sound and report whether audio works.
    Test,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Play { detach: true } => sound::play_in_background(),
        Command::Play { detach: false } => sound::play(sound::BUILT_IN),
        Command::Test => test(),
    }
}

fn test() -> Result<()> {
    println!("Playing the built-in sound...");
    sound::play(sound::BUILT_IN)?;
    println!("Done. If you heard nothing, check the default output device and its volume.");
    Ok(())
}
