//! APT package manager rules (Debian/Ubuntu).
//!
//! Contains rules for:
//! - `apt_get` - Suggest sudo when permission denied
//! - `apt_get_search` - Use apt-cache search instead of apt-get search
//! - `apt_invalid_operation` - Fix invalid apt operations
//! - `apt_list_upgradable` - Suggest apt list --upgradable

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;

/// Common APT operations for fuzzy matching.
const APT_OPERATIONS: &[&str] = &[
    "install",
    "remove",
    "purge",
    "autoremove",
    "update",
    "upgrade",
    "full-upgrade",
    "search",
    "show",
    "list",
    "edit-sources",
    "satisfy",
    "source",
    "build-dep",
    "download",
    "changelog",
    "depends",
    "rdepends",
    "policy",
];

/// Common APT-GET operations for fuzzy matching.
const APT_GET_OPERATIONS: &[&str] = &[
    "install",
    "remove",
    "purge",
    "autoremove",
    "update",
    "upgrade",
    "dist-upgrade",
    "dselect-upgrade",
    "source",
    "build-dep",
    "download",
    "clean",
    "autoclean",
    "check",
    "changelog",
    "indextargets",
];

/// Common APT-CACHE operations for fuzzy matching.
const APT_CACHE_OPERATIONS: &[&str] = &[
    "add",
    "gencaches",
    "showpkg",
    "showsrc",
    "stats",
    "dump",
    "dumpavail",
    "unmet",
    "search",
    "show",
    "depends",
    "rdepends",
    "pkgnames",
    "dotty",
    "xvcg",
    "policy",
    "madison",
];

/// Rule to suggest using sudo when apt-get/apt fails with permission denied.
///
/// Matches commands like:
/// - `apt install vim` -> "Permission denied"
/// - `apt-get update` -> "E: Could not open lock file"
///
/// Suggests prefixing with `sudo`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AptGet;

impl Rule for AptGet {
    fn name(&self) -> &str {
        "apt_get"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["apt", "apt-get"]) {
            return false;
        }

        // Check for permission denied errors
        command.output.contains("Permission denied")
            || command.output.contains("E: Could not open lock file")
            || command.output.contains("are you root?")
            || command.output.contains("must be run as root")
            || command
                .output
                .contains("dpkg: error: requested operation requires superuser privilege")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // If already prefixed with sudo, don't add it again
        if command.script.starts_with("sudo ") {
            return vec![];
        }
        vec![format!("sudo {}", command.script)]
    }

    fn priority(&self) -> i32 {
        // Give apt_get a higher priority (lower number) since permission errors are common
        900
    }
}

/// Rule to suggest apt-cache search when apt-get search is used.
///
/// apt-get doesn't have a search command; that's provided by apt-cache.
///
/// Matches:
/// - `apt-get search vim`
///
/// Suggests:
/// - `apt-cache search vim`
#[derive(Debug, Clone, Copy, Default)]
pub struct AptGetSearch;

impl Rule for AptGetSearch {
    fn name(&self) -> &str {
        "apt_get_search"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["apt-get"]) {
            return false;
        }

        // Check if the command is "apt-get search"
        command.script.contains("apt-get search")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Simple regex replacement: apt-get -> apt-cache
        if let Ok(re) = Regex::new(r"^apt-get") {
            let new_cmd = re.replace(&command.script, "apt-cache").to_string();
            if new_cmd != command.script {
                return vec![new_cmd];
            }
        }

        // Fallback: simple string replace
        vec![command.script.replace("apt-get", "apt-cache")]
    }

    fn requires_output(&self) -> bool {
        // This rule can match based solely on the command script
        false
    }
}

/// Rule to fix invalid apt/apt-get/apt-cache operations.
///
/// Matches errors like:
/// - `E: Invalid operation uninstall`
///
/// Common mistakes include using 'uninstall' instead of 'remove'.
///
/// # Example
///
/// ```text
/// $ apt uninstall vim
/// E: Invalid operation uninstall
///
/// $ fuck
/// apt remove vim
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct AptInvalidOperation;

impl AptInvalidOperation {
    /// Get operations for a given apt command.
    fn get_operations(app: &str) -> Vec<String> {
        match app {
            "apt" => APT_OPERATIONS.iter().map(|s| s.to_string()).collect(),
            "apt-get" => APT_GET_OPERATIONS.iter().map(|s| s.to_string()).collect(),
            "apt-cache" => APT_CACHE_OPERATIONS.iter().map(|s| s.to_string()).collect(),
            _ => APT_OPERATIONS.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Extract the invalid operation from the error message.
    fn get_invalid_operation(output: &str) -> Option<String> {
        // The invalid operation is typically at the end of the error message
        // "E: Invalid operation <operation>"
        let parts: Vec<&str> = output.split_whitespace().collect();
        parts.last().map(|s| s.to_string())
    }
}

impl Rule for AptInvalidOperation {
    fn name(&self) -> &str {
        "apt_invalid_operation"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["apt", "apt-get", "apt-cache"]) {
            return false;
        }

        command.output.contains("E: Invalid operation")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();
        if parts.is_empty() {
            return vec![];
        }

        let app = &parts[0];

        // Get the invalid operation from output
        let invalid_operation = match Self::get_invalid_operation(&command.output) {
            Some(op) => op,
            None => return vec![],
        };

        // Special case: 'uninstall' should be 'remove'
        if invalid_operation == "uninstall" {
            return vec![command.script.replace("uninstall", "remove")];
        }

        let operations = Self::get_operations(app);
        let suggestions = get_close_matches(&invalid_operation, &operations, 3, 0.6);

        suggestions
            .into_iter()
            .map(|op| replace_argument(&command.script, &invalid_operation, &op))
            .collect()
    }

    fn enabled_by_default(&self) -> bool {
        // Only enabled on Debian/Ubuntu-based systems
        cfg!(target_os = "linux")
    }
}

/// Rule to suggest `apt list --upgradable` when apt update shows upgradable packages.
///
/// When running `apt update`, if there are upgradable packages, apt suggests
/// running `apt list --upgradable` to see them.
///
/// Matches output containing:
/// - `apt list --upgradable`
///
/// # Example
///
/// ```text
/// $ apt update
/// Hit:1 http://archive.ubuntu.com/ubuntu focal InRelease
/// ...
/// 5 packages can be upgraded. Run 'apt list --upgradable' to see them.
///
/// $ fuck
/// apt list --upgradable
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct AptListUpgradable;

impl Rule for AptListUpgradable {
    fn name(&self) -> &str {
        "apt_list_upgradable"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["apt"]) {
            return false;
        }

        command.output.contains("apt list --upgradable")
    }

    fn get_new_command(&self, _command: &Command) -> Vec<String> {
        vec!["apt list --upgradable".to_string()]
    }

    fn enabled_by_default(&self) -> bool {
        // Only enabled on Debian/Ubuntu-based systems
        cfg!(target_os = "linux")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod apt_get_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(AptGet.name(), "apt_get");
        }

        #[test]
        fn test_matches_permission_denied() {
            let cmd = Command::new(
                "apt install vim",
                "E: Could not open lock file /var/lib/dpkg/lock-frontend - open (13: Permission denied)",
            );
            assert!(AptGet.is_match(&cmd));
        }

        #[test]
        fn test_matches_apt_get_permission_denied() {
            let cmd = Command::new(
                "apt-get install vim",
                "E: Could not open lock file /var/lib/dpkg/lock - open (13: Permission denied)",
            );
            assert!(AptGet.is_match(&cmd));
        }

        #[test]
        fn test_matches_are_you_root() {
            let cmd = Command::new("apt update", "are you root?");
            assert!(AptGet.is_match(&cmd));
        }

        #[test]
        fn test_matches_must_be_root() {
            let cmd = Command::new("apt upgrade", "This operation must be run as root");
            assert!(AptGet.is_match(&cmd));
        }

        #[test]
        fn test_matches_dpkg_superuser() {
            let cmd = Command::new(
                "apt install vim",
                "dpkg: error: requested operation requires superuser privilege",
            );
            assert!(AptGet.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("npm install vim", "Permission denied");
            assert!(!AptGet.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let cmd = Command::new("apt list --installed", "Listing... Done");
            assert!(!AptGet.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("apt install vim", "Permission denied");
            let fixes = AptGet.get_new_command(&cmd);
            assert_eq!(fixes, vec!["sudo apt install vim"]);
        }

        #[test]
        fn test_get_new_command_apt_get() {
            let cmd = Command::new("apt-get update", "Permission denied");
            let fixes = AptGet.get_new_command(&cmd);
            assert_eq!(fixes, vec!["sudo apt-get update"]);
        }

        #[test]
        fn test_no_double_sudo() {
            let cmd = Command::new("sudo apt install vim", "Permission denied");
            let fixes = AptGet.get_new_command(&cmd);
            assert!(fixes.is_empty());
        }

        #[test]
        fn test_priority() {
            assert_eq!(AptGet.priority(), 900);
        }
    }

    mod apt_get_search_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(AptGetSearch.name(), "apt_get_search");
        }

        #[test]
        fn test_matches_apt_get_search() {
            let cmd = Command::new("apt-get search vim", "E: Invalid operation search");
            assert!(AptGetSearch.is_match(&cmd));
        }

        #[test]
        fn test_no_match_apt_cache_search() {
            let cmd = Command::new("apt-cache search vim", "vim - Vi IMproved");
            assert!(!AptGetSearch.is_match(&cmd));
        }

        #[test]
        fn test_no_match_apt_install() {
            let cmd = Command::new("apt-get install vim", "Reading package lists...");
            assert!(!AptGetSearch.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("npm search vim", "some output");
            assert!(!AptGetSearch.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("apt-get search vim", "E: Invalid operation search");
            let fixes = AptGetSearch.get_new_command(&cmd);
            assert_eq!(fixes, vec!["apt-cache search vim"]);
        }

        #[test]
        fn test_get_new_command_with_args() {
            let cmd = Command::new("apt-get search python3", "E: Invalid operation search");
            let fixes = AptGetSearch.get_new_command(&cmd);
            assert_eq!(fixes, vec!["apt-cache search python3"]);
        }

        #[test]
        fn test_requires_output() {
            assert!(!AptGetSearch.requires_output());
        }
    }

    mod apt_invalid_operation_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(AptInvalidOperation.name(), "apt_invalid_operation");
        }

        #[test]
        fn test_matches_invalid_operation() {
            let cmd = Command::new("apt uninstall vim", "E: Invalid operation uninstall");
            assert!(AptInvalidOperation.is_match(&cmd));
        }

        #[test]
        fn test_matches_apt_get_invalid() {
            let cmd = Command::new("apt-get uninstall vim", "E: Invalid operation uninstall");
            assert!(AptInvalidOperation.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let cmd = Command::new("apt install vim", "Reading package lists...");
            assert!(!AptInvalidOperation.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("npm uninstall vim", "E: Invalid operation uninstall");
            assert!(!AptInvalidOperation.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_uninstall() {
            let cmd = Command::new("apt uninstall vim", "E: Invalid operation uninstall");
            let fixes = AptInvalidOperation.get_new_command(&cmd);
            assert_eq!(fixes, vec!["apt remove vim"]);
        }

        #[test]
        fn test_get_new_command_instal() {
            let cmd = Command::new("apt instal vim", "E: Invalid operation instal");
            let fixes = AptInvalidOperation.get_new_command(&cmd);
            assert!(fixes.contains(&"apt install vim".to_string()));
        }

        #[test]
        fn test_get_new_command_upgade() {
            let cmd = Command::new("apt upgade", "E: Invalid operation upgade");
            let fixes = AptInvalidOperation.get_new_command(&cmd);
            assert!(fixes.contains(&"apt upgrade".to_string()));
        }
    }

    mod apt_list_upgradable_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(AptListUpgradable.name(), "apt_list_upgradable");
        }

        #[test]
        fn test_matches_upgradable() {
            let cmd = Command::new(
                "apt update",
                "5 packages can be upgraded. Run 'apt list --upgradable' to see them.",
            );
            assert!(AptListUpgradable.is_match(&cmd));
        }

        #[test]
        fn test_no_match_no_upgradable() {
            let cmd = Command::new("apt update", "All packages are up to date.");
            assert!(!AptListUpgradable.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("dnf update", "Run 'apt list --upgradable' to see them.");
            assert!(!AptListUpgradable.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "apt update",
                "5 packages can be upgraded. Run 'apt list --upgradable' to see them.",
            );
            let fixes = AptListUpgradable.get_new_command(&cmd);
            assert_eq!(fixes, vec!["apt list --upgradable"]);
        }
    }
}
