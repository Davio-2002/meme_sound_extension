use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[value(alias = "pwsh")]
    Powershell,
    /// cmd with Clink
    Cmd,
}

/// Returns the hook that runs `faaah play` after a failed command.
pub fn script(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => include_str!("hooks/faaah.bash"),
        Shell::Zsh => include_str!("hooks/faaah.zsh"),
        Shell::Fish => include_str!("hooks/faaah.fish"),
        Shell::Powershell => include_str!("hooks/faaah.ps1"),
        Shell::Cmd => include_str!("hooks/faaah.lua"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Checks the hook syntax with `check_command` if it is installed.
    fn assert_valid_syntax(shell: Shell, check_command: &[&str]) {
        let path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        std::fs::write(&path, script(shell)).unwrap();
        let Ok(output) = Command::new(check_command[0])
            .args(&check_command[1..])
            .arg(&path)
            .output()
        else {
            eprintln!("{} not found, skipped", check_command[0]);
            return;
        };
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn bash_hook_syntax() {
        assert_valid_syntax(Shell::Bash, &["bash", "-n"]);
    }

    #[test]
    fn zsh_hook_syntax() {
        assert_valid_syntax(Shell::Zsh, &["zsh", "-n"]);
    }

    #[test]
    fn fish_hook_syntax() {
        assert_valid_syntax(Shell::Fish, &["fish", "--no-execute"]);
    }

    #[test]
    fn cmd_hook_syntax() {
        assert_valid_syntax(Shell::Cmd, &["luac", "-p"]);
    }
}
