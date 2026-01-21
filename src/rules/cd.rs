//! CD-related correction rules.
//!
//! This module contains rules for fixing common `cd` command errors:
//!
//! - [`CdParent`] - Fixes "cd.." to "cd .."
//! - [`CdMkdir`] - Creates missing directory then cd into it
//! - [`CdCorrection`] - Fuzzy matches directory names for typos
//! - [`CdCs`] - Fixes "cs" typo to "cd" (common due to keyboard proximity)

use crate::core::{is_app, Command, Rule};
use crate::utils::get_close_matches;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Rule that fixes missing space in "cd.." to "cd ..".
///
/// This is a common typo where users forget the space between `cd` and `..`.
///
/// # Example
///
/// ```
/// use oops::rules::cd::CdParent;
/// use oops::core::{Command, Rule};
///
/// let rule = CdParent;
/// let cmd = Command::new("cd..", "cd..: command not found");
/// assert!(rule.is_match(&cmd));
/// assert_eq!(rule.get_new_command(&cmd), vec!["cd .."]);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CdParent;

impl Rule for CdParent {
    fn name(&self) -> &str {
        "cd_parent"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Match "cd.." or "cd..." etc. - cd followed immediately by dots
        let script = cmd.script.trim();

        // Check for pattern: cd followed by dots with no space
        if let Some(rest) = script.strip_prefix("cd") {
            if rest.starts_with('.') && !rest.starts_with(' ') {
                return true;
            }
        }

        false
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let script = cmd.script.trim();

        // Extract the part after "cd"
        if let Some(rest) = script.strip_prefix("cd") {
            // Add space between cd and the rest
            let fixed = format!("cd {}", rest.trim());
            return vec![fixed];
        }

        vec![]
    }

    fn requires_output(&self) -> bool {
        // Can match based on script alone
        false
    }
}

/// Rule that creates a missing directory and then cd into it.
///
/// When `cd` fails because the directory doesn't exist, this rule suggests
/// creating it first with `mkdir -p` and then changing to it.
///
/// # Example
///
/// ```
/// use oops::rules::cd::CdMkdir;
/// use oops::core::{Command, Rule};
///
/// let rule = CdMkdir;
/// let cmd = Command::new("cd new_project/src", "cd: no such file or directory: new_project/src");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CdMkdir;

impl Rule for CdMkdir {
    fn name(&self) -> &str {
        "cd_mkdir"
    }

    fn priority(&self) -> i32 {
        200
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Must be a cd command
        if !is_app(cmd, &["cd"]) {
            // cd is usually a shell built-in, so check the script directly
            let script = cmd.script.trim();
            if script != "cd" && !script.starts_with("cd ") {
                return false;
            }
        }

        // Check for "no such file or directory" type errors
        let output_lower = cmd.output.to_lowercase();
        output_lower.contains("no such file or directory")
            || output_lower.contains("not a directory")
            || output_lower.contains("does not exist")
            || output_lower.contains("cannot find path")
            || output_lower.contains("the system cannot find the path")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the directory path from the cd command
        let parts = cmd.script_parts();

        if parts.len() < 2 {
            return vec![];
        }

        // Get everything after "cd" as the path
        let dir_path = parts[1..].join(" ");

        // Create mkdir -p command followed by cd
        // Use && for command chaining
        vec![format!("mkdir -p {} && cd {}", dir_path, dir_path)]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that fuzzy matches directory names for typos.
///
/// When `cd` fails because of a typo in the directory name, this rule
/// looks for similar directory names in the current location and suggests them.
///
/// # Example
///
/// ```
/// use oops::rules::cd::CdCorrection;
/// use oops::core::{Command, Rule};
///
/// let rule = CdCorrection;
/// let cmd = Command::new("cd docuemnts", "cd: no such file or directory: docuemnts");
/// // This would suggest "cd documents" if that directory exists
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CdCorrection;

impl CdCorrection {
    /// Get list of directories in the current directory.
    fn get_directories() -> Vec<String> {
        let mut dirs = Vec::new();

        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        dirs.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        dirs
    }

    /// Get list of directories in a specified parent directory.
    fn get_directories_in(parent: &Path) -> Vec<String> {
        let mut dirs = Vec::new();

        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        dirs.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        dirs
    }

    /// Extract the typo directory name from the error output.
    #[allow(dead_code)]
    fn extract_typo_from_output(output: &str) -> Option<String> {
        // Try common error message patterns
        let patterns = [
            // Bash: cd: directory: No such file or directory
            r"cd:\s+(.+?):\s+No such file or directory",
            r"cd:\s+(.+?):\s+not a directory",
            // Zsh: cd:cd:X: no such file or directory: dirname
            r"no such file or directory:\s*(.+)",
            // Fish: cd: The directory 'X' does not exist
            r"The directory '(.+?)' does not exist",
            // PowerShell: Set-Location : Cannot find path 'X'
            r"Cannot find path '(.+?)'",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(output) {
                    if let Some(m) = caps.get(1) {
                        let path = m.as_str().trim();
                        // Return just the last component if it's a path
                        return Path::new(path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .or_else(|| Some(path.to_string()));
                    }
                }
            }
        }

        None
    }
}

impl Rule for CdCorrection {
    fn name(&self) -> &str {
        "cd_correction"
    }

    fn priority(&self) -> i32 {
        300
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Must be a cd command
        let script = cmd.script.trim();
        if !script.starts_with("cd ") {
            return false;
        }

        // Must have an error about directory not found
        let output_lower = cmd.output.to_lowercase();
        output_lower.contains("no such file or directory")
            || output_lower.contains("not a directory")
            || output_lower.contains("does not exist")
            || output_lower.contains("cannot find path")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();

        if parts.len() < 2 {
            return vec![];
        }

        // Get the directory argument
        let dir_arg = &parts[1];
        let path = Path::new(dir_arg);

        // Determine which directory to search in and what name to match
        let (search_dir, typo_name) = if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                // No parent, search current directory
                (None, path.to_string_lossy().to_string())
            } else {
                // Has parent, search in parent directory
                (
                    Some(parent.to_path_buf()),
                    path.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default(),
                )
            }
        } else {
            (None, dir_arg.to_string())
        };

        // Get available directories
        let directories = match &search_dir {
            Some(parent) if parent.exists() => Self::get_directories_in(parent),
            Some(_) => return vec![], // Parent doesn't exist, cd_mkdir should handle
            None => Self::get_directories(),
        };

        if directories.is_empty() {
            return vec![];
        }

        // Find close matches
        let matches = get_close_matches(&typo_name, &directories, 3, 0.6);

        // Generate fixed commands
        matches
            .into_iter()
            .map(|correct_name| {
                if let Some(parent) = &search_dir {
                    format!("cd {}", parent.join(&correct_name).display())
                } else {
                    format!("cd {}", correct_name)
                }
            })
            .collect()
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that fixes "cs" typo to "cd".
///
/// Due to the proximity of the 'd' and 's' keys on the keyboard, typing "cs"
/// instead of "cd" is a common mistake.
///
/// # Example
///
/// ```
/// use oops::rules::cd::CdCs;
/// use oops::core::{Command, Rule};
///
/// let rule = CdCs;
/// let cmd = Command::new("cs /etc/", "cs: command not found");
/// assert!(rule.is_match(&cmd));
/// assert_eq!(rule.get_new_command(&cmd), vec!["cd /etc/"]);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CdCs;

impl Rule for CdCs {
    fn name(&self) -> &str {
        "cd_cs"
    }

    fn priority(&self) -> i32 {
        900
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return false;
        }
        parts[0] == "cs"
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace "cs" with "cd" at the beginning of the script
        if let Some(rest) = cmd.script.strip_prefix("cs") {
            vec![format!("cd{}", rest)]
        } else {
            vec!["cd".to_string()]
        }
    }

    fn requires_output(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // CdParent tests
    mod cd_parent {
        use super::*;

        #[test]
        fn test_name() {
            let rule = CdParent;
            assert_eq!(rule.name(), "cd_parent");
        }

        #[test]
        fn test_matches_cd_dotdot() {
            let rule = CdParent;
            let cmd = Command::new("cd..", "cd..: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_cd_triple_dot() {
            let rule = CdParent;
            let cmd = Command::new("cd...", "cd...: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_normal_cd() {
            let rule = CdParent;
            let cmd = Command::new("cd ..", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_cd_directory() {
            let rule = CdParent;
            let cmd = Command::new("cd Documents", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_double_dot() {
            let rule = CdParent;
            let cmd = Command::new("cd..", "");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cd .."]);
        }

        #[test]
        fn test_get_new_command_triple_dot() {
            let rule = CdParent;
            let cmd = Command::new("cd...", "");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cd ..."]);
        }

        #[test]
        fn test_does_not_require_output() {
            let rule = CdParent;
            assert!(!rule.requires_output());
        }
    }

    // CdMkdir tests
    mod cd_mkdir {
        use super::*;

        #[test]
        fn test_name() {
            let rule = CdMkdir;
            assert_eq!(rule.name(), "cd_mkdir");
        }

        #[test]
        fn test_matches_no_such_directory() {
            let rule = CdMkdir;
            let cmd = Command::new(
                "cd new_project",
                "cd: no such file or directory: new_project",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_does_not_exist() {
            let rule = CdMkdir;
            let cmd = Command::new("cd mydir", "The directory 'mydir' does not exist");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_cannot_find_path() {
            let rule = CdMkdir;
            let cmd = Command::new("cd mydir", "Set-Location : Cannot find path 'mydir'");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = CdMkdir;
            let cmd = Command::new("ls nonexistent", "No such file or directory");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful_cd() {
            let rule = CdMkdir;
            let cmd = Command::new("cd /home", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_simple() {
            let rule = CdMkdir;
            let cmd = Command::new("cd new_project", "no such file or directory");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["mkdir -p new_project && cd new_project"]);
        }

        #[test]
        fn test_get_new_command_nested() {
            let rule = CdMkdir;
            let cmd = Command::new("cd project/src/lib", "no such file or directory");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(
                fixes,
                vec!["mkdir -p project/src/lib && cd project/src/lib"]
            );
        }

        #[test]
        fn test_requires_output() {
            let rule = CdMkdir;
            assert!(rule.requires_output());
        }
    }

    // CdCorrection tests
    mod cd_correction {
        use super::*;

        #[test]
        fn test_name() {
            let rule = CdCorrection;
            assert_eq!(rule.name(), "cd_correction");
        }

        #[test]
        fn test_matches_no_such_directory() {
            let rule = CdCorrection;
            let cmd = Command::new("cd docuemnts", "cd: no such file or directory: docuemnts");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_does_not_exist() {
            let rule = CdCorrection;
            let cmd = Command::new("cd docuemnts", "The directory 'docuemnts' does not exist");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = CdCorrection;
            let cmd = Command::new("ls docuemnts", "No such file or directory");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful_cd() {
            let rule = CdCorrection;
            let cmd = Command::new("cd Documents", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_requires_output() {
            let rule = CdCorrection;
            assert!(rule.requires_output());
        }

        #[test]
        fn test_extract_typo_from_output_bash() {
            let output = "cd: docuemnts: No such file or directory";
            let typo = CdCorrection::extract_typo_from_output(output);
            assert_eq!(typo, Some("docuemnts".to_string()));
        }

        #[test]
        fn test_extract_typo_from_output_zsh() {
            let output = "no such file or directory: docuemnts";
            let typo = CdCorrection::extract_typo_from_output(output);
            assert_eq!(typo, Some("docuemnts".to_string()));
        }

        #[test]
        fn test_extract_typo_from_output_fish() {
            let output = "The directory 'docuemnts' does not exist";
            let typo = CdCorrection::extract_typo_from_output(output);
            assert_eq!(typo, Some("docuemnts".to_string()));
        }

        #[test]
        fn test_get_directories_returns_vec() {
            // Just verify the function doesn't panic
            let dirs = CdCorrection::get_directories();
            // Directories may or may not exist depending on test environment
            assert!(dirs.len() >= 0);
        }
    }

    // CdCs tests
    mod cd_cs {
        use super::*;

        #[test]
        fn test_name() {
            let rule = CdCs;
            assert_eq!(rule.name(), "cd_cs");
        }

        #[test]
        fn test_matches_cs_command() {
            let rule = CdCs;
            let cmd = Command::new("cs /etc/", "cs: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_cs_no_args() {
            let rule = CdCs;
            let cmd = Command::new("cs", "cs: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_cd() {
            let rule = CdCs;
            let cmd = Command::new("cd /etc/", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = CdCs;
            let cmd = Command::new("ls /etc/", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = CdCs;
            let cmd = Command::new("cs /etc/", "cs: command not found");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cd /etc/"]);
        }

        #[test]
        fn test_get_new_command_no_args() {
            let rule = CdCs;
            let cmd = Command::new("cs", "cs: command not found");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cd"]);
        }

        #[test]
        fn test_does_not_require_output() {
            let rule = CdCs;
            assert!(!rule.requires_output());
        }

        #[test]
        fn test_priority() {
            let rule = CdCs;
            assert_eq!(rule.priority(), 900);
        }
    }

    // Integration tests
    mod integration {
        use super::*;

        #[test]
        fn test_cd_parent_priority_higher_than_correction() {
            let parent = CdParent;
            let correction = CdCorrection;
            // Lower priority number = higher priority
            assert!(parent.priority() < correction.priority());
        }

        #[test]
        fn test_cd_mkdir_priority_between_parent_and_correction() {
            let parent = CdParent;
            let mkdir = CdMkdir;
            let correction = CdCorrection;

            assert!(parent.priority() < mkdir.priority());
            assert!(mkdir.priority() < correction.priority());
        }
    }
}
