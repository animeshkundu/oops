//! CLI argument parsing for oops
//!
//! Uses clap derive API to define the command-line interface.

use clap::Parser;

/// Special placeholder used by shell aliases to separate oops args from command args.
/// When the shell alias is invoked, it passes this placeholder followed by the previous
/// command and its arguments.
/// Note: We keep "THEFUCK_ARGUMENT_PLACEHOLDER" for backwards compatibility with existing shell configs.
pub const THEFUCK_ARGUMENT_PLACEHOLDER: &str = "THEFUCK_ARGUMENT_PLACEHOLDER";

/// A blazingly fast command-line typo corrector
#[derive(Parser, Debug, Clone)]
#[command(
    name = "oops",
    version,
    about = "A blazingly fast command-line typo corrector",
    long_about = None
)]
pub struct Cli {
    /// Generate shell alias
    #[arg(long)]
    pub alias: bool,

    /// Skip confirmation (auto-execute the first suggestion)
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,

    /// Retry if fix fails
    #[arg(short = 'r', long)]
    pub repeat: bool,

    /// Enable debug output
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Enable experimental instant mode
    #[arg(long = "enable-experimental-instant-mode")]
    pub instant_mode: bool,

    /// Force specific command (bypass rule matching)
    #[arg(long = "force-command")]
    pub force_command: Option<String>,

    /// Shell logger mode (internal use by shell integration)
    #[arg(long = "shell-logger")]
    pub shell_logger: Option<String>,

    /// Command arguments (from shell alias)
    ///
    /// These are typically passed by the shell alias after the
    /// THEFUCK_ARGUMENT_PLACEHOLDER separator.
    #[arg(trailing_var_arg = true)]
    pub command: Vec<String>,
}

impl Cli {
    /// Parse command line arguments, handling the special placeholder.
    ///
    /// Shell aliases pass arguments in the form:
    /// `thefuck [options] THEFUCK_ARGUMENT_PLACEHOLDER <previous_command> <args>...`
    ///
    /// This method extracts the command portion after the placeholder.
    pub fn parse_with_placeholder() -> Self {
        let mut cli = Self::parse();
        cli.extract_command_after_placeholder();
        cli
    }

    /// Extract the command arguments after the placeholder.
    ///
    /// If the placeholder is present in the command vector, everything
    /// after it becomes the actual command to fix.
    fn extract_command_after_placeholder(&mut self) {
        if let Some(pos) = self
            .command
            .iter()
            .position(|arg| arg == THEFUCK_ARGUMENT_PLACEHOLDER)
        {
            // Everything after the placeholder is the command
            self.command = self.command.split_off(pos + 1);
        }
    }

    /// Get the command string by joining the command arguments.
    pub fn get_command_string(&self) -> Option<String> {
        if self.command.is_empty() {
            None
        } else {
            Some(self.command.join(" "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_extraction() {
        let mut cli = Cli {
            alias: false,
            yes: false,
            repeat: false,
            debug: false,
            instant_mode: false,
            force_command: None,
            shell_logger: None,
            command: vec![
                "THEFUCK_ARGUMENT_PLACEHOLDER".to_string(),
                "git".to_string(),
                "stauts".to_string(),
            ],
        };

        cli.extract_command_after_placeholder();
        assert_eq!(cli.command, vec!["git", "stauts"]);
    }

    #[test]
    fn test_get_command_string() {
        let cli = Cli {
            alias: false,
            yes: false,
            repeat: false,
            debug: false,
            instant_mode: false,
            force_command: None,
            shell_logger: None,
            command: vec!["git".to_string(), "status".to_string()],
        };

        assert_eq!(cli.get_command_string(), Some("git status".to_string()));
    }

    #[test]
    fn test_get_command_string_empty() {
        let cli = Cli {
            alias: false,
            yes: false,
            repeat: false,
            debug: false,
            instant_mode: false,
            force_command: None,
            shell_logger: None,
            command: vec![],
        };

        assert_eq!(cli.get_command_string(), None);
    }

    #[test]
    fn test_no_placeholder() {
        let mut cli = Cli {
            alias: false,
            yes: false,
            repeat: false,
            debug: false,
            instant_mode: false,
            force_command: None,
            shell_logger: None,
            command: vec!["git".to_string(), "status".to_string()],
        };

        cli.extract_command_after_placeholder();
        // Should remain unchanged
        assert_eq!(cli.command, vec!["git", "status"]);
    }
}
