//! Shell integration module
//!
//! This module provides shell-specific integrations for various shells
//! (Bash, Zsh, Fish, PowerShell, Tcsh). Each shell implementation provides:
//! - Alias generation for the `fuck` command
//! - History reading from environment variables
//! - Shell alias parsing
//! - Command joining with and/or operators

mod bash;
mod fish;
mod powershell;
mod tcsh;
mod zsh;

use std::collections::HashMap;
use std::env;

use anyhow::Result;
use tracing::debug;

pub use bash::Bash;
pub use fish::Fish;
pub use powershell::PowerShell;
pub use tcsh::Tcsh;
pub use zsh::Zsh;

/// Trait defining the interface that all shell implementations must provide.
pub trait Shell: Send + Sync {
    /// Returns the name of the shell (e.g., "bash", "zsh").
    fn name(&self) -> &str;

    /// Generates the shell alias function for oops.
    ///
    /// # Arguments
    /// * `alias_name` - The name of the alias (typically "fuck")
    /// * `instant_mode` - Whether to enable experimental instant mode
    ///
    /// # Returns
    /// A string containing the shell function definition.
    fn app_alias(&self, alias_name: &str, instant_mode: bool) -> String;

    /// Gets the command history from the TF_HISTORY environment variable.
    ///
    /// # Returns
    /// A vector of history entries (most recent last).
    fn get_history(&self) -> Vec<String>;

    /// Parses shell aliases from the TF_SHELL_ALIASES environment variable.
    ///
    /// # Returns
    /// A HashMap mapping alias names to their expanded values.
    fn get_aliases(&self) -> HashMap<String, String>;

    /// Joins commands with the shell's AND operator.
    ///
    /// # Arguments
    /// * `commands` - The commands to join
    ///
    /// # Returns
    /// A string with commands joined by " && ".
    fn and_(&self, commands: &[&str]) -> String {
        commands.join(" && ")
    }

    /// Joins commands with the shell's OR operator.
    ///
    /// # Arguments
    /// * `commands` - The commands to join
    ///
    /// # Returns
    /// A string with commands joined by " || ".
    fn or_(&self, commands: &[&str]) -> String {
        commands.join(" || ")
    }

    /// Adds a command to the shell's history.
    ///
    /// Note: For most shells, history is modified at the shell level via
    /// the alias function, so this is a no-op. Some shells (like Fish)
    /// may require explicit history manipulation.
    ///
    /// # Arguments
    /// * `command` - The command to add to history
    fn put_to_history(&self, command: &str) -> Result<()> {
        let _ = command; // Silence unused warning
        Ok(())
    }

    /// Returns a list of shell builtin commands.
    ///
    /// # Returns
    /// A slice of builtin command names.
    fn get_builtin_commands(&self) -> &[&str] {
        &[
            "alias", "bg", "bind", "break", "builtin", "case", "cd", "command", "compgen",
            "complete", "continue", "declare", "dirs", "disown", "echo", "enable", "eval", "exec",
            "exit", "export", "fc", "fg", "getopts", "hash", "help", "history", "if", "jobs",
            "kill", "let", "local", "logout", "popd", "printf", "pushd", "pwd", "read", "readonly",
            "return", "set", "shift", "shopt", "source", "suspend", "test", "times", "trap",
            "type", "typeset", "ulimit", "umask", "unalias", "unset", "until", "wait", "while",
        ]
    }

    /// Returns the history file path for this shell.
    fn get_history_file_name(&self) -> Option<String> {
        None
    }
}

/// Registry of known shells.
type ShellFactory = fn() -> Box<dyn Shell>;

static SHELLS: &[(&str, ShellFactory)] = &[
    ("bash", || Box::new(Bash::new())),
    ("zsh", || Box::new(Zsh::new())),
    ("fish", || Box::new(Fish::new())),
    ("powershell", || Box::new(PowerShell::new())),
    ("pwsh", || Box::new(PowerShell::new())),
    ("tcsh", || Box::new(Tcsh::new())),
    ("csh", || Box::new(Tcsh::new())),
];

/// Detects the current shell from environment variables or process tree.
///
/// Detection order:
/// 1. TF_SHELL environment variable (set by the alias function)
/// 2. Process tree inspection (fallback)
///
/// # Returns
/// A boxed Shell implementation for the detected shell.
/// Falls back to Bash if no shell can be detected.
pub fn detect_shell() -> Box<dyn Shell> {
    // First, try TF_SHELL environment variable
    if let Ok(shell_name) = env::var("TF_SHELL") {
        debug!("Detected shell from TF_SHELL: {}", shell_name);
        if let Some(shell) = get_shell_by_name(&shell_name) {
            return shell;
        }
    }

    // Try to detect from process tree
    if let Some(shell) = detect_shell_from_process() {
        return shell;
    }

    // Fallback to bash
    debug!("Falling back to bash shell");
    Box::new(Bash::new())
}

/// Gets a shell implementation by name.
///
/// # Arguments
/// * `name` - The shell name (e.g., "bash", "zsh")
///
/// # Returns
/// Some(Shell) if a matching implementation exists, None otherwise.
pub fn get_shell_by_name(name: &str) -> Option<Box<dyn Shell>> {
    let name_lower = name.to_lowercase();
    for (shell_name, constructor) in SHELLS {
        if *shell_name == name_lower {
            return Some(constructor());
        }
    }
    None
}

/// Attempts to detect the shell from the process tree.
///
/// This walks up the process tree looking for a known shell process.
#[cfg(unix)]
fn detect_shell_from_process() -> Option<Box<dyn Shell>> {
    use std::fs;
    use std::path::Path;

    // Try to get parent process info from /proc
    let mut pid = std::process::id();

    for _ in 0..10 {
        // Limit iterations to avoid infinite loops
        let stat_path = format!("/proc/{}/stat", pid);
        let comm_path = format!("/proc/{}/comm", pid);

        // Try to read the process name
        let name = if let Ok(comm) = fs::read_to_string(&comm_path) {
            comm.trim().to_string()
        } else if let Ok(stat) = fs::read_to_string(&stat_path) {
            // Parse comm from stat (format: "pid (comm) ...")
            if let Some(start) = stat.find('(') {
                if let Some(end) = stat.find(')') {
                    stat[start + 1..end].to_string()
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        };

        // Check if this is a known shell
        let name_without_ext = Path::new(&name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&name);

        if let Some(shell) = get_shell_by_name(name_without_ext) {
            debug!("Detected shell from process tree: {}", name);
            return Some(shell);
        }

        // Get parent PID
        if let Ok(stat) = fs::read_to_string(&stat_path) {
            let parts: Vec<&str> = stat.split_whitespace().collect();
            if parts.len() > 3 {
                if let Ok(ppid) = parts[3].parse::<u32>() {
                    if ppid == 0 || ppid == pid {
                        break;
                    }
                    pid = ppid;
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    None
}

/// Attempts to detect the shell from the process tree (Windows version).
#[cfg(windows)]
fn detect_shell_from_process() -> Option<Box<dyn Shell>> {
    // On Windows, we rely primarily on TF_SHELL environment variable
    // Process tree inspection is more complex on Windows
    debug!("Process tree detection not fully implemented on Windows");
    None
}

/// Generates the shell alias for the `fuck` command.
///
/// This is called when `oops --alias` is invoked.
///
/// # Returns
/// Ok(()) on success, Err on failure.
pub fn generate_alias() -> Result<()> {
    let shell = detect_shell();
    let alias_name = env::var("TF_ALIAS").unwrap_or_else(|_| "oops".to_string());
    let instant_mode = env::var("THEFUCK_INSTANT_MODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let alias = shell.app_alias(&alias_name, instant_mode);
    print!("{}", alias);

    Ok(())
}

/// Runs the shell logger for instant mode.
///
/// # Arguments
/// * `logger_file` - Path to the log file
///
/// # Returns
/// Ok(()) on success, Err on failure.
pub fn run_shell_logger(logger_file: &str) -> Result<()> {
    debug!("Running shell logger with file: {}", logger_file);
    // TODO: Implement shell logger for instant mode
    // This requires capturing shell output in real-time
    let _ = logger_file;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shell_by_name_bash() {
        let shell = get_shell_by_name("bash");
        assert!(shell.is_some());
        assert_eq!(shell.unwrap().name(), "bash");
    }

    #[test]
    fn test_get_shell_by_name_zsh() {
        let shell = get_shell_by_name("zsh");
        assert!(shell.is_some());
        assert_eq!(shell.unwrap().name(), "zsh");
    }

    #[test]
    fn test_get_shell_by_name_case_insensitive() {
        let shell = get_shell_by_name("BASH");
        assert!(shell.is_some());
        assert_eq!(shell.unwrap().name(), "bash");
    }

    #[test]
    fn test_get_shell_by_name_unknown() {
        let shell = get_shell_by_name("unknown");
        assert!(shell.is_none());
    }

    #[test]
    fn test_shell_and_operator() {
        let shell = Bash::new();
        let result = shell.and_(&["cmd1", "cmd2", "cmd3"]);
        assert_eq!(result, "cmd1 && cmd2 && cmd3");
    }

    #[test]
    fn test_shell_or_operator() {
        let shell = Bash::new();
        let result = shell.or_(&["cmd1", "cmd2"]);
        assert_eq!(result, "cmd1 || cmd2");
    }
}
