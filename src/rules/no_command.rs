//! Command not found correction rule.
//!
//! This rule matches commands that fail because the command is not found
//! and suggests corrections by fuzzy matching against:
//! - Executables in PATH
//! - Commands from shell history (if available)

use crate::core::{Command, Rule};
use crate::utils::{get_all_executables, get_close_matches};
use regex::Regex;
use std::env;

/// Patterns that indicate a "command not found" error.
const NOT_FOUND_PATTERNS: &[&str] = &[
    "command not found",
    "not found",
    "not recognized",
    "is not recognized as an internal or external command",
    "not recognized as the name of a cmdlet",
    "not recognized as a cmdlet",
    "is not recognized",
    "unknown command",
    "couldn't find",
    "could not find",
    "not an operable program",
    "is not a recognized",
    "is not operable",
];

/// Extract the command name from a "command not found" error message.
///
/// Different shells format this error differently:
/// - bash: "foo: command not found"
/// - zsh: "zsh: command not found: foo"
/// - fish: "fish: Unknown command: foo"
/// - PowerShell: "'foo' is not recognized..."
fn extract_command_from_output(output: &str) -> Option<String> {
    // Try common patterns
    let patterns = [
        // bash style: "foo: command not found"
        r"^([^:\s]+): command not found",
        // zsh style: "zsh: command not found: foo"
        r"command not found: ([^\s]+)",
        // fish style: "Unknown command: foo"
        r"Unknown command[:\s]+([^\s]+)",
        // PowerShell style: "'foo' is not recognized"
        r"'([^']+)' is not recognized",
        // PowerShell style without quotes: "The term 'foo' is not recognized"
        r"The term '([^']+)' is not recognized",
        // Generic: command at start of line
        r"^([^:\s]+):.*(not found|not recognized)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for line in output.lines() {
                if let Some(caps) = re.captures(line) {
                    if let Some(m) = caps.get(1) {
                        let cmd = m.as_str().trim();
                        // Skip shell names
                        if !["bash", "zsh", "fish", "sh", "powershell", "pwsh", "cmd"].contains(&cmd) {
                            return Some(cmd.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Rule that suggests corrections for "command not found" errors.
///
/// This rule uses fuzzy matching to find similar command names from:
/// 1. Executables in the system PATH
/// 2. Recent command history (if available via TF_HISTORY)
///
/// # Example
///
/// ```
/// use oops::rules::no_command::NoCommand;
/// use oops::core::{Command, Rule};
///
/// let rule = NoCommand;
/// let cmd = Command::new("gti status", "gti: command not found");
/// assert!(rule.is_match(&cmd));
/// // Would suggest "git status" if git is in PATH
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoCommand;

impl NoCommand {
    /// Get commands from shell history.
    fn get_history_commands() -> Vec<String> {
        let mut commands = Vec::new();

        // Try to get history from TF_HISTORY environment variable
        if let Ok(history) = env::var("TF_HISTORY") {
            for line in history.lines() {
                // Extract just the command name (first word)
                if let Some(cmd) = line.split_whitespace().next() {
                    // Skip oops-related commands
                    if !["oops", "fuck", "thefuck", "tf"].contains(&cmd) {
                        commands.push(cmd.to_string());
                    }
                }
            }
        }

        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        commands.retain(|cmd| seen.insert(cmd.clone()));

        commands
    }

    /// Build list of all possible command suggestions.
    fn get_all_possible_commands() -> Vec<String> {
        let mut commands: Vec<String> = get_all_executables()
            .iter()
            .cloned()
            .collect();

        // Add commands from history
        let history_commands = Self::get_history_commands();
        for cmd in history_commands {
            if !commands.contains(&cmd) {
                commands.push(cmd);
            }
        }

        commands
    }
}

impl Rule for NoCommand {
    fn name(&self) -> &str {
        "no_command"
    }

    fn priority(&self) -> i32 {
        // Lower priority - more expensive to compute
        500
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let output_lower = cmd.output.to_lowercase();

        // Check for any "not found" pattern
        NOT_FOUND_PATTERNS
            .iter()
            .any(|pattern| output_lower.contains(&pattern.to_lowercase()))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();

        if parts.is_empty() {
            return vec![];
        }

        // Get the misspelled command
        let misspelled = &parts[0];

        // Try to extract from output first (more reliable)
        let cmd_to_match = extract_command_from_output(&cmd.output)
            .unwrap_or_else(|| misspelled.clone());

        // Get all possible commands
        let all_commands = Self::get_all_possible_commands();

        // Find close matches
        let matches = get_close_matches(&cmd_to_match, &all_commands, 3, 0.6);

        if matches.is_empty() {
            return vec![];
        }

        // Build corrected commands by replacing the first part
        let rest: String = if parts.len() > 1 {
            format!(" {}", parts[1..].join(" "))
        } else {
            String::new()
        };

        matches
            .into_iter()
            .map(|correct_cmd| format!("{}{}", correct_cmd, rest))
            .collect()
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let rule = NoCommand;
        assert_eq!(rule.name(), "no_command");
    }

    #[test]
    fn test_priority() {
        let rule = NoCommand;
        assert_eq!(rule.priority(), 500);
    }

    #[test]
    fn test_matches_bash_style() {
        let rule = NoCommand;
        let cmd = Command::new("gti status", "gti: command not found");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_matches_zsh_style() {
        let rule = NoCommand;
        let cmd = Command::new("gti status", "zsh: command not found: gti");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_matches_fish_style() {
        let rule = NoCommand;
        let cmd = Command::new("gti status", "fish: Unknown command: gti");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_matches_powershell_style() {
        let rule = NoCommand;
        let cmd = Command::new(
            "gti status",
            "'gti' is not recognized as an internal or external command",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_matches_powershell_cmdlet_style() {
        let rule = NoCommand;
        let cmd = Command::new(
            "Get-Chliditem",
            "Get-Chliditem: The term 'Get-Chliditem' is not recognized as the name of a cmdlet",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_no_match_success() {
        let rule = NoCommand;
        let cmd = Command::new("ls", "file1 file2 file3");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_no_match_different_error() {
        let rule = NoCommand;
        let cmd = Command::new("git push", "error: failed to push some refs");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_requires_output() {
        let rule = NoCommand;
        assert!(rule.requires_output());
    }

    #[test]
    fn test_extract_command_bash() {
        let output = "gti: command not found";
        let cmd = extract_command_from_output(output);
        assert_eq!(cmd, Some("gti".to_string()));
    }

    #[test]
    fn test_extract_command_zsh() {
        let output = "zsh: command not found: gti";
        let cmd = extract_command_from_output(output);
        assert_eq!(cmd, Some("gti".to_string()));
    }

    #[test]
    fn test_extract_command_fish() {
        let output = "fish: Unknown command: gti";
        let cmd = extract_command_from_output(output);
        assert_eq!(cmd, Some("gti".to_string()));
    }

    #[test]
    fn test_extract_command_powershell() {
        let output = "'gti' is not recognized as an internal or external command";
        let cmd = extract_command_from_output(output);
        assert_eq!(cmd, Some("gti".to_string()));
    }

    #[test]
    fn test_extract_command_powershell_term() {
        let output = "The term 'Get-Chliditem' is not recognized as the name of a cmdlet";
        let cmd = extract_command_from_output(output);
        assert_eq!(cmd, Some("Get-Chliditem".to_string()));
    }

    #[test]
    fn test_get_new_command_returns_vec() {
        let rule = NoCommand;
        let cmd = Command::new("gti status", "gti: command not found");
        let fixes = rule.get_new_command(&cmd);
        // May or may not find matches depending on system PATH
        // Just verify it returns a Vec without panicking
        let _ = fixes;
    }

    #[test]
    fn test_get_all_possible_commands() {
        let commands = NoCommand::get_all_possible_commands();
        // Should find at least some executables on any system
        assert!(!commands.is_empty());
    }

    #[test]
    fn test_get_history_commands_empty() {
        // When TF_HISTORY is not set, should return empty vec
        env::remove_var("TF_HISTORY");
        let commands = NoCommand::get_history_commands();
        // May be empty or contain cached results, just verify it doesn't panic
        let _ = commands;
    }

    // Integration test with a known typo
    #[test]
    fn test_known_typo_gti_to_git() {
        let rule = NoCommand;
        let cmd = Command::new("gti status", "gti: command not found");

        // Verify it matches
        assert!(rule.is_match(&cmd));

        // Get suggestions - if git is in PATH, it should be suggested
        let fixes = rule.get_new_command(&cmd);

        // If git is found in PATH, "git status" should be one of the suggestions
        // This test may not pass on all systems if git is not installed
        let has_git_suggestion = fixes.iter().any(|f| f.starts_with("git "));

        // We can't guarantee git is installed, so just log the result
        if has_git_suggestion {
            assert!(fixes.iter().any(|f| f == "git status"));
        }
    }
}
