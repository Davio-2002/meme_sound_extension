use std::env;
use std::io::Cursor;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, Player};

pub const BUILT_IN: &[u8] = include_bytes!("../assets/faaah.wav");

/// Plays WAV or MP3 `audio` on the default output device and waits until it ends.
pub fn play(audio: &'static [u8]) -> Result<()> {
    let mut device = DeviceSinkBuilder::open_default_sink().context("no audio output device")?;
    device.log_on_drop(false);
    let decoder = Decoder::new(Cursor::new(audio)).context("unsupported audio format")?;
    let player = Player::connect_new(device.mixer());
    player.append(decoder);
    player.sleep_until_end();
    Ok(())
}

/// Starts `faaah play` in a separate process and returns without waiting for it.
pub fn play_in_background() -> Result<()> {
    Command::new(env::current_exe()?)
        .arg("play")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to start the background player")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_sound_decodes() {
        assert!(Decoder::new(Cursor::new(BUILT_IN)).is_ok());
    }
}
