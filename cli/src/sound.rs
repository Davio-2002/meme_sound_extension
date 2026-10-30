use std::borrow::Cow;
use std::env;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

pub type Audio = Cow<'static, [u8]>;

const BUILT_IN: &[u8] = include_bytes!("../assets/faaah.wav");

/// Reads the custom sound, or returns the built-in one if `custom` is `None` or unreadable.
pub fn load(custom: Option<&Path>) -> Audio {
    custom
        .and_then(|path| fs::read(path).ok())
        .map_or(Cow::Borrowed(BUILT_IN), Cow::Owned)
}

pub fn decode(audio: Audio) -> Result<Decoder<Cursor<Audio>>> {
    Decoder::new(Cursor::new(audio)).context("not a WAV or MP3 file")
}

pub struct Playback {
    _device: MixerDeviceSink,
    player: Player,
}

impl Playback {
    pub fn is_playing(&self) -> bool {
        !self.player.empty()
    }
}

pub fn start(audio: Audio) -> Result<Playback> {
    let mut device = DeviceSinkBuilder::open_default_sink().context("no audio output device")?;
    device.log_on_drop(false);
    let player = Player::connect_new(device.mixer());
    player.append(decode(audio)?);
    Ok(Playback {
        _device: device,
        player,
    })
}

/// Plays `audio` on the default output device and waits until it ends.
pub fn play(audio: Audio) -> Result<()> {
    start(audio)?.player.sleep_until_end();
    Ok(())
}

/// Starts `faaah play` in a separate process and returns without waiting for it.
pub fn play_in_background() -> Result<()> {
    let mut command = Command::new(env::current_exe()?);
    command
        .arg("play")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // Out of the shell's job control, so the terminal never waits for or signals it.
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    #[cfg(windows)]
    {
        const DETACHED_PROCESS: u32 = 0x8;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x200;
        std::os::windows::process::CommandExt::creation_flags(
            &mut command,
            DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
        );
    }
    command
        .spawn()
        .context("failed to start the background player")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_sound_decodes() {
        assert!(decode(load(None)).is_ok());
    }

    #[test]
    fn missing_custom_sound_falls_back_to_built_in() {
        assert_eq!(load(Some(Path::new("/no/such/file.wav"))), BUILT_IN);
    }

    #[test]
    fn non_audio_data_is_rejected() {
        assert!(decode(Cow::Borrowed(b"not audio")).is_err());
    }
}
