//! Fedora DNF package manager rules.
//!
//! Contains rules for:
//! - `dnf_no_such_command` - Fix mistyped DNF commands

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;

/// Common DNF operations for fuzzy matching.
const DNF_OPERATIONS: &[&str] = &[
    "install",
    "remove",
    "upgrade",
    "update",
    "downgrade",
    "autoremove",
    "clean",
    "list",
    "info",
    "search",
    "provides",
    "repolist",
    "repoinfo",
    "repoquery",
    "repository-packages",
    "check-update",
    "group",
    "history",
    "makecache",
    "mark",
    "module",
    "reinstall",
    "swap",
    "updateinfo",
    "alias",
    "builddep",
    "changelog",
    "config-manager",
    "copr",
    "debug-dump",
    "debug-restore",
    "debuginfo-install",
    "download",
    "needs-restarting",
    "playground",
    "repoclosure",
    "repodiff",
    "repograph",
    "repomanage",
    "reposync",
];

/// Rule to fix mistyped DNF commands.
///
/// Matches errors like:
/// - `No such command: instal.`
///
/// Suggests the correct DNF command using fuzzy matching.
///
/// # Example
///
/// ```text
/// $ dnf instal vim
/// No such command: instal. Please use /usr/bin/dnf --help
///
/// $ fuck
/// dnf install vim
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DnfNoSuchCommand;

impl DnfNoSuchCommand {
    /// Extract the mistyped command from the DNF error output.
    fn get_misspelled_command(output: &str) -> Option<String> {
        let re = Regex::new(r"No such command: ([^.]+)\.").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Get DNF operations as owned strings.
    fn get_operations() -> Vec<String> {
        DNF_OPERATIONS.iter().map(|s| s.to_string()).collect()
    }
}

impl Rule for DnfNoSuchCommand {
    fn name(&self) -> &str {
        "dnf_no_such_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["dnf"]) {
            return false;
        }

        command.output.to_lowercase().contains("no such command")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let misspelled = match Self::get_misspelled_command(&command.output) {
            Some(cmd) => cmd,
            None => return vec![],
        };

        let operations = Self::get_operations();
        let suggestions = get_close_matches(&misspelled, &operations, 3, 0.6);

        suggestions
            .into_iter()
            .map(|op| replace_argument(&command.script, &misspelled, &op))
            .collect()
    }

    fn enabled_by_default(&self) -> bool {
        // Only enabled on systems with DNF
        cfg!(target_os = "linux")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(DnfNoSuchCommand.name(), "dnf_no_such_command");
    }

    #[test]
    fn test_matches_no_such_command() {
        let cmd = Command::new(
            "dnf instal vim",
            "No such command: instal. Please use /usr/bin/dnf --help",
        );
        assert!(DnfNoSuchCommand.is_match(&cmd));
    }

    #[test]
    fn test_matches_case_insensitive() {
        let cmd = Command::new("dnf instal vim", "no such command: instal.");
        assert!(DnfNoSuchCommand.is_match(&cmd));
    }

    #[test]
    fn test_no_match_successful() {
        let cmd = Command::new("dnf install vim", "Installing: vim...");
        assert!(!DnfNoSuchCommand.is_match(&cmd));
    }

    #[test]
    fn test_no_match_other_command() {
        let cmd = Command::new("apt instal vim", "No such command: instal.");
        assert!(!DnfNoSuchCommand.is_match(&cmd));
    }

    #[test]
    fn test_get_misspelled_command() {
        let output = "No such command: instal. Please use /usr/bin/dnf --help";
        let misspelled = DnfNoSuchCommand::get_misspelled_command(output);
        assert_eq!(misspelled, Some("instal".to_string()));
    }

    #[test]
    fn test_get_new_command() {
        let cmd = Command::new("dnf instal vim", "No such command: instal.");
        let fixes = DnfNoSuchCommand.get_new_command(&cmd);
        assert!(fixes.contains(&"dnf install vim".to_string()));
    }

    #[test]
    fn test_get_new_command_upgrade() {
        let cmd = Command::new("dnf upgade", "No such command: upgade.");
        let fixes = DnfNoSuchCommand.get_new_command(&cmd);
        assert!(fixes.contains(&"dnf upgrade".to_string()));
    }

    #[test]
    fn test_get_new_command_search() {
        let cmd = Command::new("dnf serch vim", "No such command: serch.");
        let fixes = DnfNoSuchCommand.get_new_command(&cmd);
        assert!(fixes.contains(&"dnf search vim".to_string()));
    }
}
