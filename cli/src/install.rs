use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::ValueEnum;

use crate::hooks::Shell;

const BLOCK_START: &str = ">>> faaah >>>";
const BLOCK_END: &str = "<<< faaah <<<";
const POWERSHELLS: [&str; 2] = ["pwsh", "powershell"];

/// Shells found on this system.
pub fn detect() -> Vec<Shell> {
    Shell::value_variants()
        .iter()
        .copied()
        .filter(|&shell| is_present(shell))
        .collect()
}

fn is_present(shell: Shell) -> bool {
    match shell {
        Shell::Bash | Shell::Zsh | Shell::Fish => {
            is_on_path(shell.name())
                || config_files(shell).is_ok_and(|files| files.iter().any(|file| file.exists()))
        }
        Shell::Powershell => POWERSHELLS.iter().any(|exe| is_on_path(exe)),
        Shell::Cmd => clink_dir().is_some_and(|dir| dir.is_dir()),
    }
}

/// Config files that should load the hook of `shell`.
pub fn config_files(shell: Shell) -> Result<Vec<PathBuf>> {
    let home = dirs::home_dir().context("cannot find the home directory")?;
    let files = match shell {
        Shell::Bash => vec![home.join(".bashrc")],
        Shell::Zsh => vec![env_dir("ZDOTDIR").unwrap_or(home).join(".zshrc")],
        Shell::Fish => {
            let config_dir = env_dir("XDG_CONFIG_HOME").unwrap_or(home.join(".config"));
            vec![config_dir.join("fish").join("config.fish")]
        }
        Shell::Powershell => powershell_profiles(),
        Shell::Cmd => clink_dir()
            .filter(|dir| dir.is_dir())
            .map(|dir| dir.join("faaah.lua"))
            .into_iter()
            .collect(),
    };
    if files.is_empty() {
        bail!("{} not found", shell.name());
    }
    Ok(files)
}

/// Profile paths reported by each installed PowerShell.
fn powershell_profiles() -> Vec<PathBuf> {
    POWERSHELLS
        .iter()
        .filter_map(|exe| {
            let output = Command::new(exe)
                .args(["-NoProfile", "-NonInteractive", "-Command", "$PROFILE"])
                .output()
                .ok()?;
            let profile = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (!profile.is_empty()).then(|| PathBuf::from(profile))
        })
        .collect()
}

/// Clink loads every Lua script from its profile folder.
fn clink_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("clink"))
}

fn env_dir(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

pub fn is_on_path(name: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path)
        .any(|dir| dir.join(name).is_file() || dir.join(format!("{name}.exe")).is_file())
}

/// Adds the hook block to `file`; returns `false` if it is already there.
pub fn add(shell: Shell, file: &Path) -> Result<bool> {
    let text = read(file)?.unwrap_or_default();
    let updated = with_block(&text, shell);
    if updated == text {
        return Ok(false);
    }
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    write(file, &updated)?;
    Ok(true)
}

/// Removes the hook block from `file`; returns `false` if it is not there.
pub fn remove(shell: Shell, file: &Path) -> Result<bool> {
    let Some(text) = read(file)? else {
        return Ok(false);
    };
    let updated = without_block(&text, shell);
    if updated == text {
        return Ok(false);
    }
    write(file, &updated)?;
    Ok(true)
}

pub fn has_block(shell: Shell, file: &Path) -> bool {
    let (start, _) = markers(shell);
    read(file).is_ok_and(|text| {
        text.is_some_and(|text| text.lines().any(|line| line.trim_end() == start))
    })
}

/// Reads `file`, or returns `None` if it does not exist.
fn read(file: &Path) -> Result<Option<String>> {
    match fs::read_to_string(file) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("cannot read {}", file.display())),
    }
}

fn write(file: &Path, text: &str) -> Result<()> {
    fs::write(file, text).with_context(|| format!("cannot write {}", file.display()))
}

/// Moves the hook block to the end of `text`, adding it if missing.
fn with_block(text: &str, shell: Shell) -> String {
    let (start, end) = markers(shell);
    let newline = newline(text);
    let mut updated = without_block(text, shell);
    if !updated.is_empty() {
        if !updated.ends_with('\n') {
            updated.push_str(newline);
        }
        updated.push_str(newline);
    }
    updated.push_str(&join_lines(&[&start, shell.load_line(), &end], newline));
    updated
}

/// Removes the hook block and the blank line before it.
fn without_block(text: &str, shell: Shell) -> String {
    let (start, end) = markers(shell);
    let lines: Vec<&str> = text.lines().collect();
    let Some(first) = lines.iter().position(|line| line.trim_end() == start) else {
        return text.to_string();
    };
    let Some(last) = lines[first..]
        .iter()
        .position(|line| line.trim_end() == end)
        .map(|offset| first + offset)
    else {
        return text.to_string();
    };
    let first = if first > 0 && lines[first - 1].trim().is_empty() {
        first - 1
    } else {
        first
    };
    let kept: Vec<&str> = [&lines[..first], &lines[last + 1..]].concat();
    join_lines(&kept, newline(text))
}

fn markers(shell: Shell) -> (String, String) {
    let comment = shell.comment();
    (
        format!("{comment} {BLOCK_START}"),
        format!("{comment} {BLOCK_END}"),
    )
}

/// Keeps Windows line endings in files that already use them.
fn newline(text: &str) -> &'static str {
    if text.contains("\r\n") { "\r\n" } else { "\n" }
}

fn join_lines(lines: &[&str], newline: &str) -> String {
    lines
        .iter()
        .map(|line| format!("{line}{newline}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASH_BLOCK: &str = "# >>> faaah >>>\neval \"$(faaah init bash)\"\n# <<< faaah <<<\n";

    #[test]
    fn adds_block_to_empty_text() {
        assert_eq!(with_block("", Shell::Bash), BASH_BLOCK);
    }

    #[test]
    fn adds_block_after_a_blank_line() {
        assert_eq!(
            with_block("alias ll='ls -l'", Shell::Bash),
            format!("alias ll='ls -l'\n\n{BASH_BLOCK}")
        );
    }

    #[test]
    fn adding_twice_changes_nothing() {
        let once = with_block("export A=1\n", Shell::Bash);
        assert_eq!(with_block(&once, Shell::Bash), once);
    }

    #[test]
    fn remove_restores_original_text() {
        let original = "export A=1\n";
        assert_eq!(
            without_block(&with_block(original, Shell::Bash), Shell::Bash),
            original
        );
    }

    #[test]
    fn remove_keeps_user_lines_around_block() {
        let text = format!("export A=1\n\n{BASH_BLOCK}export B=2\n");
        assert_eq!(
            without_block(&text, Shell::Bash),
            "export A=1\nexport B=2\n"
        );
    }

    #[test]
    fn keeps_windows_line_endings() {
        let updated = with_block("Set-Alias g git\r\n", Shell::Powershell);
        assert!(updated.ends_with("# <<< faaah <<<\r\n"));
        assert!(!updated.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn cmd_block_uses_lua_comments() {
        assert!(with_block("", Shell::Cmd).starts_with("-- >>> faaah >>>\n"));
    }

    #[test]
    fn add_and_remove_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("fish").join("config.fish");
        assert!(add(Shell::Fish, &file).unwrap());
        assert!(!add(Shell::Fish, &file).unwrap());
        assert!(has_block(Shell::Fish, &file));
        assert!(remove(Shell::Fish, &file).unwrap());
        assert!(!remove(Shell::Fish, &file).unwrap());
        assert!(!has_block(Shell::Fish, &file));
    }

    #[test]
    fn remove_from_missing_file_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(".bashrc");
        assert!(!remove(Shell::Bash, &file).unwrap());
        assert!(!file.exists());
    }
}
