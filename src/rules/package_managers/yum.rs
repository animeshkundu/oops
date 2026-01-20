//! CentOS/RHEL YUM package manager rules.
//!
//! Contains rules for:
//! - `yum_invalid_operation` - Fix invalid YUM operations

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};

/// Common YUM operations for fuzzy matching.
const YUM_OPERATIONS: &[&str] = &[
    "install",
    "update",
    "check-update",
    "upgrade",
    "remove",
    "erase",
    "list",
    "info",
    "provides",
    "clean",
    "makecache",
    "groups",
    "search",
    "shell",
    "resolvedep",
    "localinstall",
    "localupdate",
    "deplist",
    "repolist",
    "version",
    "history",
    "help",
    "reinstall",
    "downgrade",
    "swap",
    "autoremove",
];

/// Rule to fix invalid YUM operations.
///
/// Matches errors like:
/// - `No such command: uninstall.`
///
/// Common mistakes include using 'uninstall' instead of 'remove'.
///
/// # Example
///
/// ```text
/// $ yum uninstall vim
/// No such command: uninstall. Please use /usr/bin/yum --help
///
/// $ fuck
/// yum remove vim
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct YumInvalidOperation;

impl YumInvalidOperation {
    /// Get YUM operations as owned strings.
    fn get_operations() -> Vec<String> {
        YUM_OPERATIONS.iter().map(|s| s.to_string()).collect()
    }
}

impl Rule for YumInvalidOperation {
    fn name(&self) -> &str {
        "yum_invalid_operation"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["yum"]) {
            return false;
        }

        command.output.contains("No such command: ")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();
        if parts.len() < 2 {
            return vec![];
        }

        let invalid_operation = &parts[1];

        // Special case: 'uninstall' should be 'remove'
        if invalid_operation == "uninstall" {
            return vec![command.script.replace("uninstall", "remove")];
        }

        let operations = Self::get_operations();
        let suggestions = get_close_matches(invalid_operation, &operations, 3, 0.6);

        suggestions
            .into_iter()
            .map(|op| replace_argument(&command.script, invalid_operation, &op))
            .collect()
    }

    fn enabled_by_default(&self) -> bool {
        // Only enabled on systems with YUM
        cfg!(target_os = "linux")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(YumInvalidOperation.name(), "yum_invalid_operation");
    }

    #[test]
    fn test_matches_no_such_command() {
        let cmd = Command::new("yum uninstall vim", "No such command: uninstall. Please use /usr/bin/yum --help");
        assert!(YumInvalidOperation.is_match(&cmd));
    }

    #[test]
    fn test_no_match_successful() {
        let cmd = Command::new("yum install vim", "Installing: vim...");
        assert!(!YumInvalidOperation.is_match(&cmd));
    }

    #[test]
    fn test_no_match_other_command() {
        let cmd = Command::new("apt uninstall vim", "No such command: uninstall.");
        assert!(!YumInvalidOperation.is_match(&cmd));
    }

    #[test]
    fn test_get_new_command_uninstall() {
        let cmd = Command::new("yum uninstall vim", "No such command: uninstall.");
        let fixes = YumInvalidOperation.get_new_command(&cmd);
        assert_eq!(fixes, vec!["yum remove vim"]);
    }

    #[test]
    fn test_get_new_command_instal() {
        let cmd = Command::new("yum instal vim", "No such command: instal.");
        let fixes = YumInvalidOperation.get_new_command(&cmd);
        assert!(fixes.contains(&"yum install vim".to_string()));
    }

    #[test]
    fn test_get_new_command_upgade() {
        let cmd = Command::new("yum upgade", "No such command: upgade.");
        let fixes = YumInvalidOperation.get_new_command(&cmd);
        assert!(fixes.contains(&"yum upgrade".to_string()));
    }
}
