//! Windows Chocolatey package manager rules.
//!
//! Contains rules for:
//! - `choco_install` - Suggest `.install` suffix for metapackages

use crate::core::{is_app, Command, Rule};

/// Rule to suggest adding `.install` suffix when installing Chocolatey packages.
///
/// Some Chocolatey packages are metapackages that don't actually install anything.
/// The actual installer is in a package with `.install` suffix.
///
/// Matches errors like:
/// - `Installing the following packages:`
///
/// # Example
///
/// ```text
/// $ choco install python
/// Installing the following packages:
/// python
/// ...
///
/// $ fuck
/// choco install python.install
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ChocoInstall;

impl ChocoInstall {
    /// Checks if a script part is a package name (not a flag or parameter).
    fn is_package_name(part: &str) -> bool {
        // Skip flags and options
        if part.starts_with('-') {
            return false;
        }
        // Skip parameter values (contain = or /)
        if part.contains('=') || part.contains('/') {
            return false;
        }
        // Skip command names
        if part == "choco" || part == "cinst" || part == "install" {
            return false;
        }
        true
    }
}

impl Rule for ChocoInstall {
    fn name(&self) -> &str {
        "choco_install"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["choco", "cinst"]) {
            return false;
        }

        // Check if it's a choco install command or cinst
        let is_install =
            command.script.starts_with("choco install") || command.script.contains("cinst");

        is_install && command.output.contains("Installing the following packages")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();

        // Find the package name and add .install suffix
        for part in parts {
            if Self::is_package_name(part) && !part.ends_with(".install") {
                let new_pkg = format!("{}.install", part);
                return vec![command.script.replace(part, &new_pkg)];
            }
        }

        vec![]
    }

    fn enabled_by_default(&self) -> bool {
        // Only enabled on Windows
        cfg!(windows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(ChocoInstall.name(), "choco_install");
    }

    #[test]
    fn test_matches_choco_install() {
        let cmd = Command::new(
            "choco install python",
            "Installing the following packages:\npython\n...",
        );
        assert!(ChocoInstall.is_match(&cmd));
    }

    #[test]
    fn test_matches_cinst() {
        let cmd = Command::new(
            "cinst python",
            "Installing the following packages:\npython\n...",
        );
        assert!(ChocoInstall.is_match(&cmd));
    }

    #[test]
    fn test_no_match_successful() {
        let cmd = Command::new("choco install python", "python v3.9.0 installed");
        assert!(!ChocoInstall.is_match(&cmd));
    }

    #[test]
    fn test_no_match_other_command() {
        let cmd = Command::new("npm install python", "Installing the following packages:");
        assert!(!ChocoInstall.is_match(&cmd));
    }

    #[test]
    fn test_no_match_choco_search() {
        let cmd = Command::new("choco search python", "Installing the following packages:");
        assert!(!ChocoInstall.is_match(&cmd));
    }

    #[test]
    fn test_get_new_command() {
        let cmd = Command::new(
            "choco install python",
            "Installing the following packages:\npython",
        );
        let fixes = ChocoInstall.get_new_command(&cmd);
        assert_eq!(fixes, vec!["choco install python.install"]);
    }

    #[test]
    fn test_get_new_command_cinst() {
        let cmd = Command::new("cinst python", "Installing the following packages:\npython");
        let fixes = ChocoInstall.get_new_command(&cmd);
        assert_eq!(fixes, vec!["cinst python.install"]);
    }

    #[test]
    fn test_get_new_command_with_flags() {
        let cmd = Command::new(
            "choco install python -y",
            "Installing the following packages:\npython",
        );
        let fixes = ChocoInstall.get_new_command(&cmd);
        assert_eq!(fixes, vec!["choco install python.install -y"]);
    }

    #[test]
    fn test_get_new_command_already_has_suffix() {
        let cmd = Command::new(
            "choco install python.install",
            "Installing the following packages:\npython.install",
        );
        let fixes = ChocoInstall.get_new_command(&cmd);
        // Should not add .install again
        assert!(fixes.is_empty() || !fixes[0].contains(".install.install"));
    }

    #[test]
    fn test_is_package_name() {
        assert!(ChocoInstall::is_package_name("python"));
        assert!(ChocoInstall::is_package_name("nodejs"));
        assert!(!ChocoInstall::is_package_name("choco"));
        assert!(!ChocoInstall::is_package_name("install"));
        assert!(!ChocoInstall::is_package_name("-y"));
        assert!(!ChocoInstall::is_package_name("--yes"));
        assert!(!ChocoInstall::is_package_name("source=url"));
        assert!(!ChocoInstall::is_package_name("/silent"));
    }
}
