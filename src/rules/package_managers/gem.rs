//! Ruby gem package manager rules.
//!
//! Contains rules for:
//! - `gem_unknown_command` - Fix mistyped gem commands

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;

/// Common gem commands for fuzzy matching.
const GEM_COMMANDS: &[&str] = &[
    "build",
    "cert",
    "check",
    "cleanup",
    "contents",
    "dependency",
    "environment",
    "fetch",
    "generate_index",
    "help",
    "info",
    "install",
    "list",
    "lock",
    "mirror",
    "open",
    "outdated",
    "owner",
    "pristine",
    "push",
    "query",
    "rdoc",
    "search",
    "server",
    "signin",
    "signout",
    "sources",
    "specification",
    "stale",
    "uninstall",
    "unpack",
    "update",
    "which",
    "yank",
];

/// Rule to fix mistyped gem commands.
///
/// Matches errors like:
/// - `ERROR:  While executing gem ... (Gem::CommandLineError)`
/// - `Unknown command instal`
///
/// Suggests the correct gem command using fuzzy matching.
///
/// # Example
///
/// ```text
/// $ gem instal rails
/// ERROR:  While executing gem ... (Gem::CommandLineError)
///     Unknown command instal
///
/// $ fuck
/// gem install rails
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GemUnknownCommand;

impl GemUnknownCommand {
    /// Extract the unknown command from the gem error output.
    fn get_unknown_command(output: &str) -> Option<String> {
        let re = Regex::new(r"Unknown command (.+)$").ok()?;
        // Search line by line since the regex uses $ for end of line
        for line in output.lines() {
            if let Some(caps) = re.captures(line) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().trim().to_string());
                }
            }
        }
        None
    }

    /// Get gem commands as owned strings.
    fn get_commands() -> Vec<String> {
        GEM_COMMANDS.iter().map(|s| s.to_string()).collect()
    }
}

impl Rule for GemUnknownCommand {
    fn name(&self) -> &str {
        "gem_unknown_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["gem"]) {
            return false;
        }

        command.output.contains("ERROR:  While executing gem")
            && command.output.contains("Gem::CommandLineError")
            && command.output.contains("Unknown command")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let unknown_cmd = match Self::get_unknown_command(&command.output) {
            Some(cmd) => cmd,
            None => return vec![],
        };

        let commands = Self::get_commands();
        let suggestions = get_close_matches(&unknown_cmd, &commands, 3, 0.6);

        suggestions
            .into_iter()
            .map(|cmd| replace_argument(&command.script, &unknown_cmd, &cmd))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(GemUnknownCommand.name(), "gem_unknown_command");
    }

    #[test]
    fn test_matches_unknown_command() {
        let cmd = Command::new(
            "gem instal rails",
            "ERROR:  While executing gem ... (Gem::CommandLineError)\n    Unknown command instal",
        );
        assert!(GemUnknownCommand.is_match(&cmd));
    }

    #[test]
    fn test_no_match_successful() {
        let cmd = Command::new("gem install rails", "Successfully installed rails-7.0.0");
        assert!(!GemUnknownCommand.is_match(&cmd));
    }

    #[test]
    fn test_no_match_other_command() {
        let cmd = Command::new(
            "npm instal rails",
            "ERROR:  While executing gem ... (Gem::CommandLineError)",
        );
        assert!(!GemUnknownCommand.is_match(&cmd));
    }

    #[test]
    fn test_no_match_missing_unknown_command() {
        let cmd = Command::new(
            "gem instal rails",
            "ERROR:  While executing gem ... (Gem::CommandLineError)",
        );
        assert!(!GemUnknownCommand.is_match(&cmd));
    }

    #[test]
    fn test_get_unknown_command() {
        let output =
            "ERROR:  While executing gem ... (Gem::CommandLineError)\n    Unknown command instal";
        let unknown = GemUnknownCommand::get_unknown_command(output);
        assert_eq!(unknown, Some("instal".to_string()));
    }

    #[test]
    fn test_get_new_command() {
        let cmd = Command::new(
            "gem instal rails",
            "ERROR:  While executing gem ... (Gem::CommandLineError)\n    Unknown command instal",
        );
        let fixes = GemUnknownCommand.get_new_command(&cmd);
        assert!(fixes.contains(&"gem install rails".to_string()));
    }

    #[test]
    fn test_get_new_command_uninstal() {
        let cmd = Command::new(
            "gem uninstal rails",
            "ERROR:  While executing gem ... (Gem::CommandLineError)\n    Unknown command uninstal",
        );
        let fixes = GemUnknownCommand.get_new_command(&cmd);
        assert!(fixes.contains(&"gem uninstall rails".to_string()));
    }

    #[test]
    fn test_get_new_command_serch() {
        let cmd = Command::new(
            "gem serch rails",
            "ERROR:  While executing gem ... (Gem::CommandLineError)\n    Unknown command serch",
        );
        let fixes = GemUnknownCommand.get_new_command(&cmd);
        assert!(fixes.contains(&"gem search rails".to_string()));
    }
}
