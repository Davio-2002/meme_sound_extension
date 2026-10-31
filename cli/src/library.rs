use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};

use crate::config::Config;
use crate::sound;

pub struct Sound {
    pub name: String,
    pub path: PathBuf,
}

pub fn dir() -> Result<PathBuf> {
    Ok(Config::dir()?.join("sounds"))
}

/// Saved sounds sorted by name; a missing folder means none.
pub fn list(dir: &Path) -> Result<Vec<Sound>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err).with_context(|| format!("cannot read {}", dir.display())),
    };
    let mut sounds = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if audio_extension(&path).is_none() {
            continue;
        }
        if let Some(name) = path.file_stem().and_then(OsStr::to_str) {
            sounds.push(Sound {
                name: name.into(),
                path,
            });
        }
    }
    sounds.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(sounds)
}

pub fn find(dir: &Path, name: &str) -> Result<Sound> {
    list(dir)?
        .into_iter()
        .find(|sound| sound.name == name)
        .with_context(|| format!("no saved sound named {name}\nsee them with `faaah sound`"))
}

/// Copies `file` into `dir`, named after the file unless `name` is given.
pub fn add(dir: &Path, file: &Path, name: Option<&str>) -> Result<Sound> {
    let extension = audio_extension(file).context("only .wav and .mp3 files can be saved")?;
    let name = match name {
        Some(name) => name,
        None => file
            .file_stem()
            .and_then(OsStr::to_str)
            .context("the file name can’t be a sound name\ngive one with `--name`")?,
    };
    ensure!(
        sound::built_in(name).is_none(),
        "{name} is a built-in sound already\npick another name with `--name`"
    );
    ensure!(
        is_valid_name(name),
        "{name:?} can’t be a sound name\nuse letters, digits, spaces, `-`, `_` and `.`, or give one with `--name`"
    );
    if list(dir)?.iter().any(|sound| sound.name == name) {
        bail!(
            "a sound named {name} is saved already\npick another with `--name`, or run `faaah sound remove {name}` first"
        );
    }
    fs::create_dir_all(dir)?;
    let path = dir.join(format!("{name}.{extension}"));
    fs::copy(file, &path).with_context(|| format!("cannot copy to {}", path.display()))?;
    Ok(Sound {
        name: name.into(),
        path,
    })
}

pub fn remove(dir: &Path, name: &str) -> Result<Sound> {
    let sound = find(dir, name)?;
    fs::remove_file(&sound.path)
        .with_context(|| format!("cannot delete {}", sound.path.display()))?;
    Ok(sound)
}

fn audio_extension(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    matches!(extension.as_str(), "wav" | "mp3").then_some(extension)
}

fn is_valid_name(name: &str) -> bool {
    !name.starts_with('.')
        && name.trim() == name
        && !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || " -_.".contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(dir: &Path, file: &str) -> PathBuf {
        let path = dir.join(file);
        fs::write(&path, b"audio").unwrap();
        path
    }

    #[test]
    fn added_sound_is_listed_and_found() {
        let src = tempfile::tempdir().unwrap();
        let lib = tempfile::tempdir().unwrap();
        add(lib.path(), &source(src.path(), "Bruh.WAV"), None).unwrap();
        add(lib.path(), &source(src.path(), "x.mp3"), Some("vine boom")).unwrap();
        let names: Vec<_> = list(lib.path())
            .unwrap()
            .into_iter()
            .map(|sound| sound.name)
            .collect();
        assert_eq!(names, ["Bruh", "vine boom"]);
        assert_eq!(
            find(lib.path(), "vine boom").unwrap().path,
            lib.path().join("vine boom.mp3")
        );
    }

    #[test]
    fn duplicate_name_is_rejected() {
        let src = tempfile::tempdir().unwrap();
        let lib = tempfile::tempdir().unwrap();
        add(lib.path(), &source(src.path(), "bruh.wav"), None).unwrap();
        assert!(add(lib.path(), &source(src.path(), "bruh.mp3"), None).is_err());
    }

    #[test]
    fn bad_names_and_formats_are_rejected() {
        let src = tempfile::tempdir().unwrap();
        let lib = tempfile::tempdir().unwrap();
        let wav = source(src.path(), "ok.wav");
        for name in [sound::DEFAULT, "../up", ".hidden", "", " pad"] {
            assert!(add(lib.path(), &wav, Some(name)).is_err(), "{name:?}");
        }
        assert!(add(lib.path(), &source(src.path(), "song.ogg"), None).is_err());
        assert!(list(lib.path()).unwrap().is_empty());
    }

    #[test]
    fn removed_sound_is_gone() {
        let src = tempfile::tempdir().unwrap();
        let lib = tempfile::tempdir().unwrap();
        let sound = add(lib.path(), &source(src.path(), "bruh.wav"), None).unwrap();
        remove(lib.path(), "bruh").unwrap();
        assert!(!sound.path.exists());
        assert!(remove(lib.path(), "bruh").is_err());
    }

    #[test]
    fn missing_folder_lists_nothing() {
        let lib = tempfile::tempdir().unwrap();
        assert!(list(&lib.path().join("sounds")).unwrap().is_empty());
    }
}
