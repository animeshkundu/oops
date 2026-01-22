//! Cargo package manager rules (Rust).
//!
//! Contains rules for:
//! - `cargo_no_command` - Suggest similar cargo subcommands when command not recognized

use crate::core::{is_app, Command, Rule};
use crate::utils::replace_argument;
use regex::Regex;

/// Rule to suggest similar cargo subcommands when "no such subcommand" error.
///
/// Matches errors like:
/// - `error: no such subcommand: 'buidl'. Did you mean 'build'?`
///
/// Suggests the correct subcommand from the error message.
#[derive(Debug, Clone, Copy, Default)]
pub struct CargoNoCommand;

impl CargoNoCommand {
    /// Extract the broken subcommand from script parts.
    fn get_broken_subcommand(parts: &[String]) -> Option<String> {
        // The subcommand is typically the second element (after "cargo")
        parts.get(1).cloned()
    }

    /// Extract the suggested fix from cargo's error output.
    fn get_suggested_fix(output: &str) -> Option<String> {
        // Pattern: Did you mean `fix`?
        let re = Regex::new(r"Did you mean `([^`]+)`").ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }
}

impl Rule for CargoNoCommand {
    fn name(&self) -> &str {
        "cargo_no_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["cargo"]) {
            return false;
        }

        // Check for "no such subcommand" or "no such command" with a suggestion
        let output_lower = command.output.to_lowercase();
        (output_lower.contains("no such subcommand") || output_lower.contains("no such command"))
            && command.output.contains("Did you mean")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let broken = match Self::get_broken_subcommand(command.script_parts()) {
            Some(b) => b,
            None => return vec![],
        };

        let fixed = match Self::get_suggested_fix(&command.output) {
            Some(f) => f,
            None => return vec![],
        };

        vec![replace_argument(&command.script, &broken, &fixed)]
    }
}

/// Rule to suggest cargo subcommands when there's a similar command available.
///
/// This rule uses fuzzy matching as a fallback when cargo doesn't provide
/// a suggestion.
#[derive(Debug, Clone, Copy, Default)]
pub struct CargoWrongCommand;

impl CargoWrongCommand {
    /// Common cargo subcommands for fuzzy matching.
    const CARGO_SUBCOMMANDS: &'static [&'static str] = &[
        "build",
        "check",
        "clean",
        "doc",
        "new",
        "init",
        "add",
        "remove",
        "run",
        "test",
        "bench",
        "update",
        "search",
        "publish",
        "install",
        "uninstall",
        "clippy",
        "fmt",
        "fix",
        "tree",
        "vendor",
        "verify-project",
        "version",
        "help",
    ];
}

impl Rule for CargoWrongCommand {
    fn name(&self) -> &str {
        "cargo_wrong_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["cargo"]) {
            return false;
        }

        // Only match if cargo shows "no such subcommand" without a "Did you mean"
        let output_lower = command.output.to_lowercase();
        (output_lower.contains("no such subcommand") || output_lower.contains("no such command"))
            && !command.output.contains("Did you mean")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();
        let broken = match parts.get(1) {
            Some(b) => b,
            None => return vec![],
        };

        // Use fuzzy matching to find the closest subcommand
        let subcommands: Vec<String> = Self::CARGO_SUBCOMMANDS
            .iter()
            .map(|s| s.to_string())
            .collect();

        let matches = crate::utils::get_close_matches(broken, &subcommands, 1, 0.6);

        if let Some(fixed) = matches.into_iter().next() {
            return vec![replace_argument(&command.script, broken, &fixed)];
        }

        vec![]
    }

    fn priority(&self) -> i32 {
        // Lower priority than CargoNoCommand since this uses fuzzy matching
        1100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod cargo_no_command_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(CargoNoCommand.name(), "cargo_no_command");
        }

        #[test]
        fn test_matches_no_such_subcommand() {
            let cmd = Command::new(
                "cargo buidl",
                "error: no such subcommand: `buidl`\n\n\tDid you mean `build`?",
            );
            assert!(CargoNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_matches_no_such_command() {
            let cmd = Command::new(
                "cargo tset",
                "error: no such command: `tset`\n\n\tDid you mean `test`?",
            );
            assert!(CargoNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let cmd = Command::new("cargo build", "Compiling myproject v0.1.0");
            assert!(!CargoNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_tool() {
            let cmd = Command::new(
                "npm buidl",
                "error: no such subcommand\n\tDid you mean `build`?",
            );
            assert!(!CargoNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_without_suggestion() {
            let cmd = Command::new("cargo xyz", "error: no such subcommand: `xyz`");
            assert!(!CargoNoCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_broken_subcommand() {
            let parts = vec!["cargo".to_string(), "buidl".to_string()];
            let broken = CargoNoCommand::get_broken_subcommand(&parts);
            assert_eq!(broken, Some("buidl".to_string()));
        }

        #[test]
        fn test_get_suggested_fix() {
            let output = "error: no such subcommand: `buidl`\n\n\tDid you mean `build`?";
            let fix = CargoNoCommand::get_suggested_fix(output);
            assert_eq!(fix, Some("build".to_string()));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "cargo buidl",
                "error: no such subcommand: `buidl`\n\n\tDid you mean `build`?",
            );
            let fixes = CargoNoCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cargo build"]);
        }

        #[test]
        fn test_get_new_command_with_args() {
            let cmd = Command::new(
                "cargo buidl --release",
                "error: no such subcommand: `buidl`\n\n\tDid you mean `build`?",
            );
            let fixes = CargoNoCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cargo build --release"]);
        }

        #[test]
        fn test_get_new_command_test_typo() {
            let cmd = Command::new(
                "cargo tset",
                "error: no such subcommand: `tset`\n\n\tDid you mean `test`?",
            );
            let fixes = CargoNoCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cargo test"]);
        }
    }

    mod cargo_wrong_command_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(CargoWrongCommand.name(), "cargo_wrong_command");
        }

        #[test]
        fn test_matches_no_suggestion() {
            let cmd = Command::new("cargo bildx", "error: no such subcommand: `bildx`");
            assert!(CargoWrongCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_with_suggestion() {
            // CargoNoCommand should handle this case
            let cmd = Command::new(
                "cargo buidl",
                "error: no such subcommand: `buidl`\n\n\tDid you mean `build`?",
            );
            assert!(!CargoWrongCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let cmd = Command::new("cargo build", "Compiling...");
            assert!(!CargoWrongCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_fuzzy() {
            let cmd = Command::new("cargo bilud", "error: no such subcommand: `bilud`");
            let fixes = CargoWrongCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cargo build"]);
        }

        #[test]
        fn test_get_new_command_tset() {
            let cmd = Command::new("cargo tset", "error: no such subcommand: `tset`");
            let fixes = CargoWrongCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cargo test"]);
        }

        #[test]
        fn test_priority() {
            assert_eq!(CargoWrongCommand.priority(), 1100);
        }
    }
}
