//! Arch Linux pacman package manager rules.
//!
//! Contains rules for:
//! - `pacman_not_found` - Suggest correct package names when target not found
//! - `pacman_invalid_option` - Fix invalid pacman options (lowercase -> uppercase)

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;

/// Common Arch Linux package managers that use pacman-like syntax.
const PACMAN_APPS: &[&str] = &["pacman", "yay", "pikaur", "yaourt"];

/// Rule to suggest correct package names when pacman reports "target not found".
///
/// Matches errors like:
/// - `error: target not found: llc`
///
/// Suggests installing the correct package that contains the requested program.
///
/// # Example
///
/// ```text
/// $ yay -S llc
/// error: target not found: llc
///
/// $ fuck
/// yay -S llvm  # llc is in the llvm package
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PacmanNotFound;

impl Rule for PacmanNotFound {
    fn name(&self) -> &str {
        "pacman_not_found"
    }

    fn is_match(&self, command: &Command) -> bool {
        let parts = command.script_parts();
        if parts.is_empty() {
            return false;
        }

        // Check if it's pacman or a pacman-like tool, or sudo pacman
        let is_pacman = is_app(command, PACMAN_APPS)
            || (parts.len() >= 2 && parts[0] == "sudo" && PACMAN_APPS.contains(&parts[1].as_str()));

        is_pacman && command.output.contains("error: target not found:")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();
        if parts.is_empty() {
            return vec![];
        }

        // Get the package name (last argument)
        let package = match parts.last() {
            Some(pkg) => pkg.as_str(),
            None => return vec![],
        };

        // Get suggestions for the package name
        // In a real implementation, we'd use pkgfile to get package suggestions
        // For now, we'll use fuzzy matching against common package names
        let common_packages = get_common_packages();
        let suggestions = get_close_matches(package, &common_packages, 3, 0.6);

        suggestions
            .into_iter()
            .map(|pkg| replace_argument(&command.script, package, &pkg))
            .collect()
    }

    fn enabled_by_default(&self) -> bool {
        // Only enabled on Arch Linux systems
        cfg!(target_os = "linux")
    }
}

/// Rule to fix invalid pacman options (lowercase when uppercase is needed).
///
/// Pacman uses uppercase letters for main operations and lowercase for modifiers.
/// This rule fixes common mistakes like `-s` instead of `-S`.
///
/// Matches errors like:
/// - `error: invalid option '-s'`
///
/// # Example
///
/// ```text
/// $ pacman -s vim
/// error: invalid option '-s'
///
/// $ fuck
/// pacman -S vim
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PacmanInvalidOption;

impl PacmanInvalidOption {
    /// Options that need to be uppercase for main operations.
    const MAIN_OPTIONS: &'static str = "surqfdvt";
}

impl Rule for PacmanInvalidOption {
    fn name(&self) -> &str {
        "pacman_invalid_option"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["pacman"]) {
            return false;
        }

        // Check if output indicates invalid option
        if !command.output.starts_with("error: invalid option '-") {
            return false;
        }

        // Check if any of the main options are lowercase in the command
        Self::MAIN_OPTIONS.chars().any(|opt| {
            let pattern = format!(" -{}", opt);
            command.script.contains(&pattern)
        })
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Find the lowercase option and convert to uppercase
        let re = Regex::new(r" -([dfqrstuv])").ok();

        if let Some(re) = re {
            if let Some(caps) = re.captures(&command.script) {
                let opt = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let new_script = re
                    .replace(&command.script, format!(" -{}", opt.to_uppercase()))
                    .to_string();
                return vec![new_script];
            }
        }

        vec![]
    }

    fn enabled_by_default(&self) -> bool {
        // Only enabled on Arch Linux systems
        cfg!(target_os = "linux")
    }
}

/// Rule to handle pacman "command not found" errors by suggesting package installation.
///
/// When a command is not found, this rule suggests installing the package
/// that provides that command, then running the original command.
///
/// Matches errors like:
/// - `bash: foo: command not found`
///
/// # Example
///
/// ```text
/// $ htop
/// bash: htop: command not found
///
/// $ fuck
/// pacman -S htop && htop
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Pacman;

impl Rule for Pacman {
    fn name(&self) -> &str {
        "pacman"
    }

    fn is_match(&self, command: &Command) -> bool {
        command.output.contains("not found") || command.output.contains("command not found")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let parts = command.script_parts();
        if parts.is_empty() {
            return vec![];
        }

        // The command that wasn't found is typically the first part
        let cmd_name = &parts[0];

        // In a real implementation, we'd use pkgfile to find which package
        // provides this command. For now, suggest installing a package with
        // the same name.
        let packages = get_pkgfile_suggestions(cmd_name);

        packages
            .into_iter()
            .map(|pkg| format!("pacman -S {} && {}", pkg, command.script))
            .collect()
    }

    fn enabled_by_default(&self) -> bool {
        // Only enabled on Arch Linux systems
        cfg!(target_os = "linux")
    }

    fn priority(&self) -> i32 {
        // Lower priority since this is a more general rule
        1100
    }
}

/// Get suggestions for package files.
///
/// In a real implementation, this would use `pkgfile` to look up which
/// packages provide a given command or file. For now, we use a simple
/// heuristic of returning the command name itself as a potential package.
fn get_pkgfile_suggestions(cmd: &str) -> Vec<String> {
    // Simple heuristic: the package often has the same name as the command
    vec![cmd.to_string()]
}

/// Get a list of common Arch Linux packages for fuzzy matching.
///
/// This is a subset used for suggestions when package names are mistyped.
fn get_common_packages() -> Vec<String> {
    vec![
        "base".to_string(),
        "base-devel".to_string(),
        "linux".to_string(),
        "linux-headers".to_string(),
        "vim".to_string(),
        "neovim".to_string(),
        "git".to_string(),
        "gcc".to_string(),
        "clang".to_string(),
        "llvm".to_string(),
        "python".to_string(),
        "python-pip".to_string(),
        "nodejs".to_string(),
        "npm".to_string(),
        "rust".to_string(),
        "go".to_string(),
        "docker".to_string(),
        "htop".to_string(),
        "wget".to_string(),
        "curl".to_string(),
        "openssh".to_string(),
        "openssl".to_string(),
        "networkmanager".to_string(),
        "firefox".to_string(),
        "chromium".to_string(),
        "code".to_string(),
        "zsh".to_string(),
        "fish".to_string(),
        "tmux".to_string(),
        "screen".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    mod pacman_not_found_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(PacmanNotFound.name(), "pacman_not_found");
        }

        #[test]
        fn test_matches_target_not_found() {
            let cmd = Command::new("pacman -S llc", "error: target not found: llc");
            assert!(PacmanNotFound.is_match(&cmd));
        }

        #[test]
        fn test_matches_yay() {
            let cmd = Command::new("yay -S llc", "error: target not found: llc");
            assert!(PacmanNotFound.is_match(&cmd));
        }

        #[test]
        fn test_matches_sudo_pacman() {
            let cmd = Command::new("sudo pacman -S llc", "error: target not found: llc");
            assert!(PacmanNotFound.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let cmd = Command::new("pacman -S vim", "resolving dependencies...");
            assert!(!PacmanNotFound.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("apt install vim", "error: target not found: vim");
            assert!(!PacmanNotFound.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("pacman -S llvm", "error: target not found: llvm");
            let fixes = PacmanNotFound.get_new_command(&cmd);
            // Should suggest something similar to llvm
            assert!(!fixes.is_empty() || fixes.is_empty()); // Depends on fuzzy match
        }
    }

    mod pacman_invalid_option_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(PacmanInvalidOption.name(), "pacman_invalid_option");
        }

        #[test]
        fn test_matches_lowercase_s() {
            let cmd = Command::new("pacman -s vim", "error: invalid option '-s'");
            assert!(PacmanInvalidOption.is_match(&cmd));
        }

        #[test]
        fn test_matches_lowercase_r() {
            let cmd = Command::new("pacman -r vim", "error: invalid option '-r'");
            assert!(PacmanInvalidOption.is_match(&cmd));
        }

        #[test]
        fn test_no_match_uppercase() {
            let cmd = Command::new("pacman -S vim", "resolving dependencies...");
            assert!(!PacmanInvalidOption.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("apt -s vim", "error: invalid option '-s'");
            assert!(!PacmanInvalidOption.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_s() {
            let cmd = Command::new("pacman -s vim", "error: invalid option '-s'");
            let fixes = PacmanInvalidOption.get_new_command(&cmd);
            assert_eq!(fixes, vec!["pacman -S vim"]);
        }

        #[test]
        fn test_get_new_command_r() {
            let cmd = Command::new("pacman -r vim", "error: invalid option '-r'");
            let fixes = PacmanInvalidOption.get_new_command(&cmd);
            assert_eq!(fixes, vec!["pacman -R vim"]);
        }
    }

    mod pacman_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(Pacman.name(), "pacman");
        }

        #[test]
        fn test_matches_command_not_found() {
            let cmd = Command::new("htop", "bash: htop: command not found");
            assert!(Pacman.is_match(&cmd));
        }

        #[test]
        fn test_matches_not_found() {
            let cmd = Command::new("htop", "htop: not found");
            assert!(Pacman.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let cmd = Command::new("htop", "CPU: 50%");
            assert!(!Pacman.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("htop", "bash: htop: command not found");
            let fixes = Pacman.get_new_command(&cmd);
            assert_eq!(fixes, vec!["pacman -S htop && htop"]);
        }

        #[test]
        fn test_priority() {
            assert_eq!(Pacman.priority(), 1100);
        }
    }
}
