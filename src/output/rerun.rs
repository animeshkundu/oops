//! Command re-execution and output capture
//!
//! This module provides functionality for re-running commands and capturing
//! their output, with support for timeouts and slow command handling.

use std::env;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

/// Default timeout multiplier for slow commands
const SLOW_COMMAND_TIMEOUT_MULTIPLIER: u32 = 15;

/// Re-run a command and capture its output.
///
/// Executes the given script in a shell and captures both stdout and stderr,
/// merging them into a single output string. The process is killed if it
/// exceeds the specified timeout.
///
/// # Arguments
///
/// * `script` - The command script to execute
/// * `timeout` - Maximum duration to wait for the command to complete
///
/// # Returns
///
/// * `Ok(String)` - The merged stdout and stderr output
/// * `Err` - If the command fails to execute or times out
///
/// # Example
///
/// ```no_run
/// use std::time::Duration;
/// use oops::output::rerun::get_output;
///
/// let output = get_output("ls -la", Duration::from_secs(5))?;
/// println!("Output: {}", output);
/// ```
pub fn get_output(script: &str, timeout: Duration) -> Result<String> {
    let shell = get_shell();
    let shell_args = get_shell_args(&shell);

    let mut child = Command::new(&shell)
        .args(&shell_args)
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .envs(env::vars())
        .spawn()
        .with_context(|| format!("Failed to execute command: {}", script))?;

    let start = Instant::now();

    // Get handles to stdout and stderr
    let mut stdout = child.stdout.take().expect("Failed to capture stdout");
    let mut stderr = child.stderr.take().expect("Failed to capture stderr");

    let mut output = String::new();
    let mut stdout_buffer = Vec::new();
    let mut stderr_buffer = Vec::new();

    // Use non-blocking reads with timeout checking
    loop {
        // Check timeout
        if start.elapsed() > timeout {
            // Kill the process if it's still running
            let _ = child.kill();
            let _ = child.wait();
            break;
        }

        // Try to wait for process completion with a short timeout
        match child.try_wait() {
            Ok(Some(_status)) => {
                // Process finished, read remaining output
                stdout.read_to_end(&mut stdout_buffer).ok();
                stderr.read_to_end(&mut stderr_buffer).ok();
                break;
            }
            Ok(None) => {
                // Process still running, continue loop
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => {
                // Error checking status, try to read what we can
                break;
            }
        }
    }

    // Merge stdout and stderr
    if !stdout_buffer.is_empty() {
        output.push_str(&String::from_utf8_lossy(&stdout_buffer));
    }
    if !stderr_buffer.is_empty() {
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&String::from_utf8_lossy(&stderr_buffer));
    }

    Ok(output)
}

/// Get extended output with a longer timeout for slow commands.
///
/// If the script matches any of the slow_commands patterns, uses an extended
/// timeout (base_timeout * SLOW_COMMAND_TIMEOUT_MULTIPLIER).
///
/// # Arguments
///
/// * `script` - The command script to execute
/// * `base_timeout` - Base timeout duration
/// * `slow_commands` - List of command patterns considered slow
///
/// # Returns
///
/// * `Ok(String)` - The merged stdout and stderr output
/// * `Err` - If the command fails to execute or times out
pub fn get_output_with_slow_handling(
    script: &str,
    base_timeout: Duration,
    slow_commands: &[String],
) -> Result<String> {
    let timeout = if is_slow_command(script, slow_commands) {
        base_timeout * SLOW_COMMAND_TIMEOUT_MULTIPLIER
    } else {
        base_timeout
    };

    get_output(script, timeout)
}

/// Check if a command is in the slow_commands list.
///
/// A command is considered slow if it starts with any of the patterns
/// in the slow_commands list. Comparison is case-insensitive and
/// matches command prefixes.
///
/// # Arguments
///
/// * `script` - The command script to check
/// * `slow_commands` - List of command patterns considered slow
///
/// # Returns
///
/// `true` if the command matches any slow command pattern
///
/// # Example
///
/// ```
/// use oops::output::rerun::is_slow_command;
///
/// let slow_cmds = vec![
///     "apt".to_string(),
///     "pip".to_string(),
///     "git push".to_string(),
/// ];
///
/// assert!(is_slow_command("apt install vim", &slow_cmds));
/// assert!(is_slow_command("pip install requests", &slow_cmds));
/// assert!(is_slow_command("git push origin main", &slow_cmds));
/// assert!(!is_slow_command("ls -la", &slow_cmds));
/// ```
pub fn is_slow_command(script: &str, slow_commands: &[String]) -> bool {
    let script_lower = script.to_lowercase();

    for slow_cmd in slow_commands {
        let slow_cmd_lower = slow_cmd.to_lowercase();

        // Check if script starts with the slow command
        if script_lower.starts_with(&slow_cmd_lower) {
            // Make sure it's a word boundary (not just a prefix match)
            // e.g., "apt-get" should match "apt" but "aptitude" should match "aptitude"
            let remaining = &script[slow_cmd.len()..];
            if remaining.is_empty()
                || remaining.starts_with(' ')
                || remaining.starts_with('\t')
                || remaining.starts_with('-')
            {
                return true;
            }
        }

        // Also check for commands after sudo, env, etc.
        for prefix in &["sudo ", "sudo -e ", "env ", "time "] {
            if script_lower.starts_with(prefix) {
                let after_prefix = &script_lower[prefix.len()..];
                if after_prefix.starts_with(&slow_cmd_lower) {
                    let offset = prefix.len() + slow_cmd.len();
                    if offset >= script.len() {
                        return true;
                    }
                    let remaining = &script[offset..];
                    if remaining.is_empty()
                        || remaining.starts_with(' ')
                        || remaining.starts_with('\t')
                        || remaining.starts_with('-')
                    {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Get the current shell to use for command execution.
///
/// Uses the TF_SHELL environment variable if set, otherwise falls back
/// to SHELL on Unix or cmd.exe on Windows.
fn get_shell() -> String {
    // First check TF_SHELL
    if let Ok(shell) = env::var("TF_SHELL") {
        return shell;
    }

    // Then SHELL on Unix
    #[cfg(unix)]
    if let Ok(shell) = env::var("SHELL") {
        return shell;
    }

    // Default shells
    #[cfg(windows)]
    {
        // Check for PowerShell
        if let Ok(comspec) = env::var("COMSPEC") {
            if comspec.to_lowercase().contains("powershell") {
                return comspec;
            }
        }
        // Default to cmd.exe
        env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }

    #[cfg(not(windows))]
    "/bin/sh".to_string()
}

/// Get shell arguments for executing a command.
fn get_shell_args(shell: &str) -> Vec<&'static str> {
    let shell_lower = shell.to_lowercase();

    if shell_lower.contains("powershell") || shell_lower.contains("pwsh") {
        vec!["-NoProfile", "-NonInteractive", "-Command"]
    } else if shell_lower.contains("cmd") {
        vec!["/C"]
    } else {
        // Unix shells (bash, zsh, fish, sh, etc.)
        vec!["-c"]
    }
}

/// Execute a command and return whether it succeeded.
///
/// This is useful for running the corrected command and checking if it worked.
///
/// # Arguments
///
/// * `script` - The command script to execute
///
/// # Returns
///
/// `true` if the command exited with status 0, `false` otherwise
pub fn execute_command(script: &str) -> bool {
    let shell = get_shell();
    let shell_args = get_shell_args(&shell);

    match Command::new(&shell)
        .args(&shell_args)
        .arg(script)
        .envs(env::vars())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Execute a command with inherited stdio (for interactive execution).
///
/// This runs the command with stdin, stdout, and stderr connected to the
/// parent process, allowing for interactive commands.
///
/// # Arguments
///
/// * `script` - The command script to execute
///
/// # Returns
///
/// * `Ok(i32)` - The exit code of the command
/// * `Err` - If the command fails to execute
pub fn execute_interactive(script: &str) -> Result<i32> {
    let shell = get_shell();
    let shell_args = get_shell_args(&shell);

    let status = Command::new(&shell)
        .args(&shell_args)
        .arg(script)
        .envs(env::vars())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute command: {}", script))?;

    Ok(status.code().unwrap_or(-1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_slow_command_basic() {
        let slow_commands = vec![
            "apt".to_string(),
            "pip".to_string(),
            "brew".to_string(),
            "git push".to_string(),
            "git pull".to_string(),
        ];

        assert!(is_slow_command("apt install vim", &slow_commands));
        assert!(is_slow_command("apt-get update", &slow_commands));
        assert!(is_slow_command("pip install requests", &slow_commands));
        assert!(is_slow_command("brew install node", &slow_commands));
        assert!(is_slow_command("git push origin main", &slow_commands));
        assert!(is_slow_command("git pull origin main", &slow_commands));
    }

    #[test]
    fn test_is_slow_command_with_sudo() {
        let slow_commands = vec!["apt".to_string(), "pip".to_string()];

        assert!(is_slow_command("sudo apt install vim", &slow_commands));
        assert!(is_slow_command("sudo -E pip install requests", &slow_commands));
    }

    #[test]
    fn test_is_slow_command_non_matching() {
        let slow_commands = vec!["apt".to_string(), "pip".to_string()];

        assert!(!is_slow_command("ls -la", &slow_commands));
        assert!(!is_slow_command("cd /tmp", &slow_commands));
        assert!(!is_slow_command("echo hello", &slow_commands));
        assert!(!is_slow_command("git status", &slow_commands));
    }

    #[test]
    fn test_is_slow_command_case_insensitive() {
        let slow_commands = vec!["APT".to_string(), "PIP".to_string()];

        assert!(is_slow_command("apt install vim", &slow_commands));
        assert!(is_slow_command("pip install requests", &slow_commands));
    }

    #[test]
    fn test_is_slow_command_empty_list() {
        let slow_commands: Vec<String> = vec![];
        assert!(!is_slow_command("apt install vim", &slow_commands));
    }

    #[test]
    fn test_get_shell_args_unix() {
        assert_eq!(get_shell_args("/bin/bash"), vec!["-c"]);
        assert_eq!(get_shell_args("/bin/zsh"), vec!["-c"]);
        assert_eq!(get_shell_args("/usr/bin/fish"), vec!["-c"]);
    }

    #[test]
    fn test_get_shell_args_windows() {
        assert_eq!(get_shell_args("cmd.exe"), vec!["/C"]);
        assert_eq!(get_shell_args("C:\\Windows\\System32\\cmd.exe"), vec!["/C"]);
        assert_eq!(
            get_shell_args("powershell.exe"),
            vec!["-NoProfile", "-NonInteractive", "-Command"]
        );
        assert_eq!(
            get_shell_args("pwsh"),
            vec!["-NoProfile", "-NonInteractive", "-Command"]
        );
    }

    #[test]
    fn test_get_output_simple_command() {
        // This test runs an actual command, so it's platform-specific
        #[cfg(unix)]
        {
            let output = get_output("echo hello", Duration::from_secs(5)).unwrap();
            assert!(output.trim() == "hello");
        }

        #[cfg(windows)]
        {
            let output = get_output("echo hello", Duration::from_secs(5)).unwrap();
            assert!(output.contains("hello"));
        }
    }
}
