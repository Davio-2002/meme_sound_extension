use std::borrow::Cow;
use std::env;
use std::fs::{self, File, TryLockError};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

pub type Audio = Cow<'static, [u8]>;

pub const BUILT_INS: [(&str, &[u8]); 2] = [
    ("faaah", include_bytes!("../assets/faaah.wav")),
    ("spongebob", include_bytes!("../assets/spongebob.mp3")),
];

pub const DEFAULT: &str = BUILT_INS[0].0;

pub fn built_in(name: &str) -> Option<&'static [u8]> {
    BUILT_INS
        .iter()
        .find(|(built_in, _)| *built_in == name)
        .map(|(_, audio)| *audio)
}

/// Returns the built-in sound `sound` names, or reads it as a file, falling back to the default sound.
pub fn load(sound: &Path) -> Audio {
    let default = || Cow::Borrowed(built_in(DEFAULT).unwrap());
    if let Some(audio) = sound.to_str().and_then(built_in) {
        return Cow::Borrowed(audio);
    }
    fs::read(sound).map_or_else(|_| default(), Cow::Owned)
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

/// Plays `audio` unless another `faaah` process is playing already, so failures in a row do not overlap.
pub fn play_alone(audio: Audio) -> Result<()> {
    let Some(_lock) = try_lock(&lock_path()?)? else {
        return Ok(());
    };
    play(audio)
}

fn lock_path() -> Result<PathBuf> {
    let dir = dirs::cache_dir()
        .context("cannot find the cache directory")?
        .join("faaah");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("play.lock"))
}

/// The OS releases the lock when the process exits, so a crash never leaves it stuck.
fn try_lock(path: &Path) -> Result<Option<File>> {
    let file = File::create(path).with_context(|| format!("cannot open {}", path.display()))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(err)) => Err(err.into()),
    }
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
    fn built_in_sounds_decode() {
        for (name, _) in BUILT_INS {
            assert!(decode(load(Path::new(name))).is_ok(), "{name}");
        }
    }

    #[test]
    fn missing_custom_sound_falls_back_to_default() {
        assert_eq!(
            load(Path::new("/no/such/file.wav")),
            built_in(DEFAULT).unwrap()
        );
    }

    #[test]
    fn non_audio_data_is_rejected() {
        assert!(decode(Cow::Borrowed(b"not audio")).is_err());
    }

    #[test]
    fn second_lock_is_refused_until_first_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("play.lock");
        let first = try_lock(&path).unwrap();
        assert!(first.is_some());
        assert!(try_lock(&path).unwrap().is_none());
        drop(first);
        assert!(try_lock(&path).unwrap().is_some());
    }
}
