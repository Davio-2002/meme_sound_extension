use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::sound;

/// Ctrl+C exit codes on Unix and Windows.
const CTRL_C_EXIT_CODES: [i32; 2] = [130, -1073741510];

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub muted: bool,
    /// Name of a built-in sound, or a sound file.
    pub sound: PathBuf,
    /// Exit codes that do not play the sound.
    pub ignore_codes: Vec<i32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            muted: false,
            sound: PathBuf::from(sound::DEFAULT),
            ignore_codes: CTRL_C_EXIT_CODES.to_vec(),
        }
    }
}

impl Config {
    pub fn dir() -> Result<PathBuf> {
        let dir = dirs::config_dir().context("cannot find the config directory")?;
        Ok(dir.join("faaah"))
    }

    pub fn path() -> Result<PathBuf> {
        Ok(Self::dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        Self::read(&Self::path()?)
    }

    pub fn save(&self) -> Result<()> {
        self.write(&Self::path()?)
    }

    /// Reads the config, or returns the default one if the file does not exist.
    fn read(path: &Path) -> Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => return Err(err).with_context(|| format!("cannot read {}", path.display())),
        };
        toml::from_str(&text).with_context(|| format!("invalid config {}", path.display()))
    }

    fn write(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, toml::to_string(self)?)
            .with_context(|| format!("cannot write {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_default() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::read(&dir.path().join("config.toml")).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn write_then_read_keeps_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("faaah").join("config.toml");
        let config = Config {
            muted: true,
            sound: PathBuf::from("/sounds/bruh.mp3"),
            ignore_codes: vec![1, 2],
        };
        config.write(&path).unwrap();
        assert_eq!(Config::read(&path).unwrap(), config);
    }

    #[test]
    fn broken_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "muted = maybe").unwrap();
        assert!(Config::read(&path).is_err());
    }
}
