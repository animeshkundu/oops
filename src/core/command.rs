//! Command type representing a failed shell command.

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::process::{Command as ProcessCommand, Stdio};

/// Represents a command that was executed and potentially failed.
///
/// The `Command` struct stores the original script and its output (combined
/// stderr and stdout). It also lazily parses the script into parts for
/// efficient rule matching.
///
/// # Example
///
/// ```
/// use oops::core::Command;
///
/// let cmd = Command::new("git push", "fatal: not a git repository");
/// assert_eq!(cmd.script, "git push");
/// assert!(cmd.output.contains("not a git repository"));
/// ```
#[derive(Debug, Clone)]
pub struct Command {
    /// The raw command string as entered by the user.
    pub script: String,
    /// Combined stderr + stdout from command execution.
    pub output: String,
    /// Lazily parsed script parts (shell-split).
    script_parts: OnceCell<Vec<String>>,
}

impl Command {
    /// Creates a new Command with the given script and output.
    ///
    /// # Arguments
    ///
    /// * `script` - The raw command string
    /// * `output` - The combined stderr and stdout from execution
    ///
    /// # Example
    ///
    /// ```
    /// use oops::core::Command;
    ///
    /// let cmd = Command::new("apt install vim", "Permission denied");
    /// ```
    pub fn new(script: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            script: script.into(),
            output: output.into(),
            script_parts: OnceCell::new(),
        }
    }

    /// Returns a new Command with the script updated but output preserved.
    ///
    /// This is useful when a rule needs to modify the command script
    /// while keeping the original output for further matching.
    ///
    /// # Example
    ///
    /// ```
    /// use oops::core::Command;
    ///
    /// let cmd = Command::new("git ci -m 'test'", "trace: alias expansion");
    /// let expanded = cmd.with_script("git commit -m 'test'");
    /// assert_eq!(expanded.script, "git commit -m 'test'");
    /// assert_eq!(expanded.output, "trace: alias expansion");
    /// ```
    pub fn with_script(&self, script: impl Into<String>) -> Self {
        Self {
            script: script.into(),
            output: self.output.clone(),
            script_parts: OnceCell::new(),
        }
    }

    /// Returns the script split into parts using shell lexing rules.
    ///
    /// The parts are cached after the first call for efficiency.
    /// If shell lexing fails (e.g., unmatched quotes), falls back to
    /// simple whitespace splitting.
    ///
    /// # Example
    ///
    /// ```
    /// use oops::core::Command;
    ///
    /// let cmd = Command::new("git commit -m 'Initial commit'", "");
    /// let parts = cmd.script_parts();
    /// assert_eq!(parts, &["git", "commit", "-m", "Initial commit"]);
    /// ```
    pub fn script_parts(&self) -> &[String] {
        self.script_parts.get_or_init(|| {
            // Use shlex for proper shell-style parsing
            shlex::split(&self.script).unwrap_or_else(|| {
                // Fallback to simple whitespace split if shlex fails
                self.script.split_whitespace().map(String::from).collect()
            })
        })
    }

    /// Creates a Command by executing a raw script and capturing its output.
    ///
    /// This function executes the command through the system shell and captures
    /// both stdout and stderr as the output.
    ///
    /// # Arguments
    ///
    /// * `raw` - A slice of strings representing the command and its arguments.
    ///           The first element is the command, followed by its arguments.
    ///
    /// # Returns
    ///
    /// A `Result` containing the Command with captured output, or an error
    /// if the command could not be executed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oops::core::Command;
    ///
    /// let cmd = Command::from_raw_script(&["ls", "-la", "/nonexistent"]).unwrap();
    /// assert!(cmd.output.contains("No such file") || cmd.output.contains("cannot access"));
    /// ```
    pub fn from_raw_script(raw: &[impl AsRef<str>]) -> Result<Self> {
        if raw.is_empty() {
            anyhow::bail!("Cannot execute empty command");
        }

        let script = raw
            .iter()
            .map(|s| {
                let s = s.as_ref();
                // Quote arguments that contain spaces or special characters
                if s.contains(' ') || s.contains('"') || s.contains('\'') {
                    shlex::try_quote(s)
                        .map(|q| q.to_string())
                        .unwrap_or_else(|_| format!("\"{}\"", s.replace('"', "\\\"")))
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        // Determine which shell to use
        let (shell, shell_arg) = if cfg!(windows) {
            // On Windows, try to use the shell from TF_SHELL or default to cmd
            match std::env::var("TF_SHELL").as_deref() {
                Ok("powershell") | Ok("pwsh") => ("powershell", "-Command"),
                _ => ("cmd", "/C"),
            }
        } else {
            // On Unix, use sh
            ("sh", "-c")
        };

        let output = ProcessCommand::new(shell)
            .arg(shell_arg)
            .arg(&script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to execute command: {}", script))?;

        // Combine stderr and stdout (stderr first, as it typically contains errors)
        let mut combined_output = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            if !combined_output.is_empty() {
                combined_output.push('\n');
            }
            combined_output.push_str(&stdout);
        }

        Ok(Self::new(script, combined_output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_command() {
        let cmd = Command::new("git status", "On branch master");
        assert_eq!(cmd.script, "git status");
        assert_eq!(cmd.output, "On branch master");
    }

    #[test]
    fn test_script_parts_simple() {
        let cmd = Command::new("git commit -m message", "");
        let parts = cmd.script_parts();
        assert_eq!(parts, &["git", "commit", "-m", "message"]);
    }

    #[test]
    fn test_script_parts_with_quotes() {
        let cmd = Command::new("git commit -m 'Initial commit'", "");
        let parts = cmd.script_parts();
        assert_eq!(parts, &["git", "commit", "-m", "Initial commit"]);
    }

    #[test]
    fn test_script_parts_cached() {
        let cmd = Command::new("git status", "");
        let parts1 = cmd.script_parts();
        let parts2 = cmd.script_parts();
        // Same reference means it was cached
        assert!(std::ptr::eq(parts1, parts2));
    }

    #[test]
    fn test_script_parts_empty() {
        let cmd = Command::new("", "");
        let parts = cmd.script_parts();
        assert!(parts.is_empty());
    }

    #[test]
    fn test_clone() {
        let cmd1 = Command::new("test", "output");
        let cmd2 = cmd1.clone();
        assert_eq!(cmd1.script, cmd2.script);
        assert_eq!(cmd1.output, cmd2.output);
    }
}
