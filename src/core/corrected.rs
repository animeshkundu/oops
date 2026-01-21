//! CorrectedCommand type representing a suggested command correction.

use crate::config::Settings;
use crate::core::Command;
use anyhow::Result;
use std::fmt;
use std::process::{Command as ProcessCommand, Stdio};
use std::sync::Arc;

/// Type alias for side effect functions.
///
/// A side effect is a function that runs after a corrected command is executed.
/// It receives the original command and the new script that was run.
pub type SideEffect = Arc<dyn Fn(&Command, &str) -> Result<()> + Send + Sync>;

/// Represents a corrected command suggestion.
///
/// A `CorrectedCommand` contains a suggested fix for a failed command,
/// along with metadata about its priority and any side effects that
/// should be executed after the corrected command runs.
///
/// # Example
///
/// ```
/// use oops::core::CorrectedCommand;
///
/// let correction = CorrectedCommand::new("sudo apt install vim", 1000);
/// assert_eq!(correction.script, "sudo apt install vim");
/// assert_eq!(correction.priority, 1000);
/// ```
#[derive(Clone)]
pub struct CorrectedCommand {
    /// The corrected command script to execute.
    pub script: String,
    /// Priority for sorting corrections (lower = higher priority).
    pub priority: i32,
    /// Optional side effect to run after the corrected command executes.
    pub side_effect: Option<SideEffect>,
}

impl CorrectedCommand {
    /// Creates a new CorrectedCommand with the given script and priority.
    ///
    /// # Arguments
    ///
    /// * `script` - The corrected command script
    /// * `priority` - Priority for sorting (lower values = higher priority)
    ///
    /// # Example
    ///
    /// ```
    /// use oops::core::CorrectedCommand;
    ///
    /// let correction = CorrectedCommand::new("git push --force", 900);
    /// ```
    pub fn new(script: impl Into<String>, priority: i32) -> Self {
        Self {
            script: script.into(),
            priority,
            side_effect: None,
        }
    }

    /// Creates a CorrectedCommand with a side effect.
    ///
    /// # Arguments
    ///
    /// * `script` - The corrected command script
    /// * `priority` - Priority for sorting
    /// * `side_effect` - Function to run after the command executes
    ///
    /// # Example
    ///
    /// ```
    /// use oops::core::CorrectedCommand;
    /// use std::sync::Arc;
    ///
    /// let correction = CorrectedCommand::with_side_effect(
    ///     "source ~/.bashrc",
    ///     1000,
    ///     Arc::new(|_old_cmd, _new_script| {
    ///         println!("Reloading shell configuration");
    ///         Ok(())
    ///     }),
    /// );
    /// ```
    pub fn with_side_effect(
        script: impl Into<String>,
        priority: i32,
        side_effect: SideEffect,
    ) -> Self {
        Self {
            script: script.into(),
            priority,
            side_effect: Some(side_effect),
        }
    }

    /// Runs the corrected command and any associated side effects.
    ///
    /// This method executes the corrected command through the shell and,
    /// if successful and a side effect is registered, runs the side effect.
    ///
    /// # Arguments
    ///
    /// * `old_cmd` - The original failed command (passed to side effects)
    /// * `settings` - Application settings containing configuration
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the command execution.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oops::core::{Command, CorrectedCommand};
    ///
    /// let old_cmd = Command::new("apt install vim", "Permission denied");
    /// let correction = CorrectedCommand::new("sudo apt install vim", 1000);
    ///
    /// // In a real scenario, you'd have actual settings
    /// // correction.run(&old_cmd, &settings)?;
    /// ```
    pub fn run(&self, old_cmd: &Command, _settings: &Settings) -> Result<()> {
        // Determine which shell to use
        let (shell, shell_arg) = if cfg!(windows) {
            match std::env::var("TF_SHELL").as_deref() {
                Ok("powershell") | Ok("pwsh") => ("powershell", "-Command"),
                _ => ("cmd", "/C"),
            }
        } else {
            ("sh", "-c")
        };

        // Execute the corrected command
        let status = ProcessCommand::new(shell)
            .arg(shell_arg)
            .arg(&self.script)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        // Run side effect if present and command succeeded
        if status.success() {
            if let Some(ref side_effect) = self.side_effect {
                side_effect(old_cmd, &self.script)?;
            }
            Ok(())
        } else {
            anyhow::bail!(
                "Command exited with status: {}",
                status.code().unwrap_or(-1)
            )
        }
    }

    /// Runs the corrected command without waiting for completion.
    ///
    /// This is useful for commands that should run in the background
    /// or when you don't need to wait for the result.
    pub fn run_detached(&self) -> Result<()> {
        let (shell, shell_arg) = if cfg!(windows) {
            match std::env::var("TF_SHELL").as_deref() {
                Ok("powershell") | Ok("pwsh") => ("powershell", "-Command"),
                _ => ("cmd", "/C"),
            }
        } else {
            ("sh", "-c")
        };

        ProcessCommand::new(shell)
            .arg(shell_arg)
            .arg(&self.script)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(())
    }
}

impl fmt::Debug for CorrectedCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorrectedCommand")
            .field("script", &self.script)
            .field("priority", &self.priority)
            .field("has_side_effect", &self.side_effect.is_some())
            .finish()
    }
}

impl PartialEq for CorrectedCommand {
    fn eq(&self, other: &Self) -> bool {
        self.script == other.script && self.priority == other.priority
    }
}

impl Eq for CorrectedCommand {}

impl PartialOrd for CorrectedCommand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CorrectedCommand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Sort by priority first (lower = higher priority)
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => {
                // Then by script alphabetically for stable sorting
                self.script.cmp(&other.script)
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_corrected_command() {
        let cmd = CorrectedCommand::new("sudo apt install", 1000);
        assert_eq!(cmd.script, "sudo apt install");
        assert_eq!(cmd.priority, 1000);
        assert!(cmd.side_effect.is_none());
    }

    #[test]
    fn test_with_side_effect() {
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        let cmd = CorrectedCommand::with_side_effect(
            "test",
            500,
            Arc::new(move |_, _| {
                called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }),
        );

        assert!(cmd.side_effect.is_some());
    }

    #[test]
    fn test_ordering() {
        let cmd1 = CorrectedCommand::new("aaa", 1000);
        let cmd2 = CorrectedCommand::new("bbb", 500);
        let cmd3 = CorrectedCommand::new("ccc", 1000);

        let mut commands = vec![cmd1.clone(), cmd2.clone(), cmd3.clone()];
        commands.sort();

        assert_eq!(commands[0].script, "bbb"); // Lowest priority value first
        assert_eq!(commands[1].script, "aaa"); // Same priority, alphabetical
        assert_eq!(commands[2].script, "ccc");
    }

    #[test]
    fn test_equality() {
        let cmd1 = CorrectedCommand::new("test", 1000);
        let cmd2 = CorrectedCommand::new("test", 1000);
        let cmd3 = CorrectedCommand::new("test", 500);

        assert_eq!(cmd1, cmd2);
        assert_ne!(cmd1, cmd3);
    }

    #[test]
    fn test_debug_format() {
        let cmd = CorrectedCommand::new("test", 1000);
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("1000"));
    }
}
