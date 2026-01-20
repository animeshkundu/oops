//! pip package manager rules (Python).
//!
//! Contains rules for:
//! - `pip_install` - Suggest pip install when "No module named" error
//! - `pip_unknown_command` - Suggest similar pip commands when command not recognized

use crate::core::{is_app, Command, Rule};
use crate::utils::replace_argument;
use regex::Regex;

/// Rule to suggest pip install with --user when permission denied.
///
/// Matches errors like:
/// - `pip install package` -> "Permission denied"
///
/// Suggests adding --user flag, or if that's already present, using sudo.
#[derive(Debug, Clone, Copy, Default)]
pub struct PipInstall;

impl Rule for PipInstall {
    fn name(&self) -> &str {
        "pip_install"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["pip", "pip2", "pip3"]) {
            return false;
        }

        // Check if it's an install command
        let parts = command.script_parts();
        let has_install = parts.iter().any(|p| p == "install");

        has_install && command.output.contains("Permission denied")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // If --user is not present, try adding it
        if !command.script.contains("--user") {
            return vec![command.script.replace(" install ", " install --user ")];
        }

        // If --user is already present, try with sudo (removing --user)
        let without_user = command.script.replace(" --user", "");
        vec![format!("sudo {}", without_user)]
    }

    fn priority(&self) -> i32 {
        // Higher priority for common permission issues
        900
    }
}

/// Rule to suggest correct pip commands when command is not recognized.
///
/// Matches errors like:
/// - `ERROR: unknown command "instal", maybe you meant "install"`
///
/// Suggests the correct command from the error message.
#[derive(Debug, Clone, Copy, Default)]
pub struct PipUnknownCommand;

impl PipUnknownCommand {
    /// Extract the broken command and suggested fix from pip error output.
    fn get_broken_and_fix(output: &str) -> Option<(String, String)> {
        // Pattern: ERROR: unknown command "broken", maybe you meant "fixed"
        let re = Regex::new(r#"ERROR: unknown command "([^"]+)".*maybe you meant "([^"]+)""#).ok()?;
        let caps = re.captures(output)?;

        let broken = caps.get(1)?.as_str().to_string();
        let fixed = caps.get(2)?.as_str().to_string();

        Some((broken, fixed))
    }
}

impl Rule for PipUnknownCommand {
    fn name(&self) -> &str {
        "pip_unknown_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["pip", "pip2", "pip3"]) {
            return false;
        }

        command.output.contains("unknown command") && command.output.contains("maybe you meant")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let (broken, fixed) = match Self::get_broken_and_fix(&command.output) {
            Some((b, f)) => (b, f),
            None => return vec![],
        };

        vec![replace_argument(&command.script, &broken, &fixed)]
    }
}

/// Rule to suggest pip install when "No module named" error occurs in Python.
///
/// This is useful when running Python scripts that import missing packages.
///
/// Matches errors like:
/// - `ModuleNotFoundError: No module named 'requests'`
///
/// Suggests installing the missing module.
#[derive(Debug, Clone, Copy, Default)]
pub struct PipModuleNotFound;

impl PipModuleNotFound {
    /// Extract the module name from the error output.
    fn get_module_name(output: &str) -> Option<String> {
        // Pattern: No module named 'module' or No module named "module"
        let re = Regex::new(r#"No module named ['"]([\w\-_]+)['""]"#).ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }

    /// Map common module names to their pip package names.
    fn module_to_package(module: &str) -> String {
        // Some modules have different pip package names
        match module {
            "cv2" => "opencv-python".to_string(),
            "PIL" => "Pillow".to_string(),
            "sklearn" => "scikit-learn".to_string(),
            "yaml" => "PyYAML".to_string(),
            "bs4" => "beautifulsoup4".to_string(),
            "dateutil" => "python-dateutil".to_string(),
            _ => module.to_string(),
        }
    }
}

impl Rule for PipModuleNotFound {
    fn name(&self) -> &str {
        "pip_module_not_found"
    }

    fn is_match(&self, command: &Command) -> bool {
        // This rule matches when running Python and getting module not found errors
        if !is_app(command, &["python", "python2", "python3"]) {
            return false;
        }

        command.output.contains("No module named")
            && (command.output.contains("ModuleNotFoundError")
                || command.output.contains("ImportError"))
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let module = match Self::get_module_name(&command.output) {
            Some(m) => m,
            None => return vec![],
        };

        let package = Self::module_to_package(&module);

        // Suggest pip install followed by the original command
        vec![format!("pip install {} && {}", package, command.script)]
    }

    fn priority(&self) -> i32 {
        // Lower priority since this is a two-step fix
        1100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod pip_install_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(PipInstall.name(), "pip_install");
        }

        #[test]
        fn test_matches_permission_denied() {
            let cmd = Command::new(
                "pip install requests",
                "ERROR: Could not install packages due to an OSError: Permission denied",
            );
            assert!(PipInstall.is_match(&cmd));
        }

        #[test]
        fn test_matches_pip3() {
            let cmd = Command::new("pip3 install flask", "Permission denied: '/usr/lib'");
            assert!(PipInstall.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful_install() {
            let cmd = Command::new(
                "pip install requests",
                "Successfully installed requests-2.28.0",
            );
            assert!(!PipInstall.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("npm install requests", "Permission denied");
            assert!(!PipInstall.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_add_user() {
            let cmd = Command::new("pip install requests", "Permission denied");
            let fixes = PipInstall.get_new_command(&cmd);
            assert_eq!(fixes, vec!["pip install --user requests"]);
        }

        #[test]
        fn test_get_new_command_with_user_try_sudo() {
            let cmd = Command::new("pip install --user requests", "Permission denied");
            let fixes = PipInstall.get_new_command(&cmd);
            assert_eq!(fixes, vec!["sudo pip install requests"]);
        }

        #[test]
        fn test_priority() {
            assert_eq!(PipInstall.priority(), 900);
        }
    }

    mod pip_unknown_command_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(PipUnknownCommand.name(), "pip_unknown_command");
        }

        #[test]
        fn test_matches_unknown_command() {
            let cmd = Command::new(
                "pip instal requests",
                r#"ERROR: unknown command "instal", maybe you meant "install""#,
            );
            assert!(PipUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let cmd = Command::new("pip install requests", "Installing collected packages");
            assert!(!PipUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_tool() {
            let cmd = Command::new(
                "npm instal",
                r#"ERROR: unknown command "instal", maybe you meant "install""#,
            );
            assert!(!PipUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_broken_and_fix() {
            let output = r#"ERROR: unknown command "instal", maybe you meant "install""#;
            let result = PipUnknownCommand::get_broken_and_fix(output);
            assert_eq!(
                result,
                Some(("instal".to_string(), "install".to_string()))
            );
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "pip instal requests",
                r#"ERROR: unknown command "instal", maybe you meant "install""#,
            );
            let fixes = PipUnknownCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["pip install requests"]);
        }
    }

    mod pip_module_not_found_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(PipModuleNotFound.name(), "pip_module_not_found");
        }

        #[test]
        fn test_matches_module_not_found() {
            let cmd = Command::new(
                "python script.py",
                "ModuleNotFoundError: No module named 'requests'",
            );
            assert!(PipModuleNotFound.is_match(&cmd));
        }

        #[test]
        fn test_matches_import_error() {
            let cmd = Command::new(
                "python3 app.py",
                "ImportError: No module named 'flask'",
            );
            assert!(PipModuleNotFound.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new(
                "node app.js",
                "ModuleNotFoundError: No module named 'requests'",
            );
            assert!(!PipModuleNotFound.is_match(&cmd));
        }

        #[test]
        fn test_no_match_no_module_error() {
            let cmd = Command::new("python script.py", "SyntaxError: invalid syntax");
            assert!(!PipModuleNotFound.is_match(&cmd));
        }

        #[test]
        fn test_get_module_name() {
            let output = "ModuleNotFoundError: No module named 'requests'";
            let module = PipModuleNotFound::get_module_name(output);
            assert_eq!(module, Some("requests".to_string()));
        }

        #[test]
        fn test_get_module_name_double_quotes() {
            let output = r#"ImportError: No module named "flask""#;
            let module = PipModuleNotFound::get_module_name(output);
            assert_eq!(module, Some("flask".to_string()));
        }

        #[test]
        fn test_module_to_package_mapping() {
            assert_eq!(PipModuleNotFound::module_to_package("cv2"), "opencv-python");
            assert_eq!(PipModuleNotFound::module_to_package("PIL"), "Pillow");
            assert_eq!(
                PipModuleNotFound::module_to_package("sklearn"),
                "scikit-learn"
            );
            assert_eq!(
                PipModuleNotFound::module_to_package("requests"),
                "requests"
            );
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "python script.py",
                "ModuleNotFoundError: No module named 'requests'",
            );
            let fixes = PipModuleNotFound.get_new_command(&cmd);
            assert_eq!(fixes, vec!["pip install requests && python script.py"]);
        }

        #[test]
        fn test_get_new_command_with_mapping() {
            let cmd = Command::new(
                "python3 image.py",
                "ModuleNotFoundError: No module named 'cv2'",
            );
            let fixes = PipModuleNotFound.get_new_command(&cmd);
            assert_eq!(
                fixes,
                vec!["pip install opencv-python && python3 image.py"]
            );
        }

        #[test]
        fn test_priority() {
            assert_eq!(PipModuleNotFound.priority(), 1100);
        }
    }
}
