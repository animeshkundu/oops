//! PATH scanning and executable utilities.
//!
//! This module provides functionality for:
//! - Finding all executables in the system PATH
//! - Checking if a program exists in PATH
//! - Replacing command arguments in scripts

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Cached set of all executables found in PATH.
///
/// This is lazily initialized on first access and cached for the lifetime
/// of the program to avoid repeated filesystem operations.
static ALL_EXECUTABLES: Lazy<HashSet<String>> = Lazy::new(|| {
    let mut executables = HashSet::new();

    // Get PATH environment variable
    let path_env = env::var("PATH").unwrap_or_default();

    // Split PATH using the platform-appropriate separator
    #[cfg(windows)]
    let separator = ';';
    #[cfg(not(windows))]
    let separator = ':';

    for path_str in path_env.split(separator) {
        let path = PathBuf::from(path_str);

        // Skip if path doesn't exist or isn't a directory
        if !path.is_dir() {
            continue;
        }

        // Try to read directory contents
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                // Skip directories
                if entry_path.is_dir() {
                    continue;
                }

                // Check if file is executable
                if is_executable(&entry_path) {
                    if let Some(name) = entry_path.file_name() {
                        let name_str = name.to_string_lossy().to_string();

                        // On Windows, also add the name without extension
                        #[cfg(windows)]
                        {
                            executables.insert(name_str.clone());
                            if let Some(stem) = entry_path.file_stem() {
                                executables.insert(stem.to_string_lossy().to_string());
                            }
                        }

                        #[cfg(not(windows))]
                        executables.insert(name_str);
                    }
                }
            }
        }
    }

    // Filter out oops-related entries to avoid circular corrections
    let tf_entries: HashSet<&str> = ["oops", "thefuck", "fuck", "tf"].iter().cloned().collect();
    executables.retain(|name| !tf_entries.contains(name.as_str()));

    executables
});

/// Check if a file is executable.
///
/// On Unix, checks the executable permission bits.
/// On Windows, checks if the file has an executable extension.
#[cfg(unix)]
fn is_executable(path: &PathBuf) -> bool {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        // Check if any execute bit is set (user, group, or other)
        permissions.mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    // On Windows, check for common executable extensions
    const EXECUTABLE_EXTENSIONS: &[&str] = &["exe", "cmd", "bat", "com", "ps1", "vbs", "js", "msc"];

    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        EXECUTABLE_EXTENSIONS.contains(&ext_lower.as_str())
    } else {
        // Also consider files without extension if they exist in PATH
        // (some tools like WSL might create such files)
        false
    }
}

/// Get all executables in PATH.
///
/// Returns a reference to a cached set of all executable names found
/// in the system PATH. The result is cached for the lifetime of the program.
///
/// # Returns
///
/// A reference to a `HashSet<String>` containing all executable names.
///
/// # Example
///
/// ```
/// use oops::utils::executables::get_all_executables;
///
/// let executables = get_all_executables();
/// if executables.contains("git") {
///     println!("git is available");
/// }
/// ```
pub fn get_all_executables() -> &'static HashSet<String> {
    &ALL_EXECUTABLES
}

/// Check if a program exists in PATH and return its full path.
///
/// Uses the `which` crate to find the full path to a program.
///
/// # Arguments
///
/// * `program` - The name of the program to find
///
/// # Returns
///
/// * `Some(PathBuf)` - The full path to the program if found
/// * `None` - If the program is not found in PATH
///
/// # Example
///
/// ```
/// use oops::utils::executables::which;
///
/// if let Some(path) = which("git") {
///     println!("git is at: {:?}", path);
/// }
/// ```
pub fn which(program: &str) -> Option<PathBuf> {
    ::which::which(program).ok()
}

/// Check if a program exists in PATH.
///
/// This is a convenience function that returns a boolean.
///
/// # Arguments
///
/// * `program` - The name of the program to check
///
/// # Returns
///
/// `true` if the program exists in PATH, `false` otherwise.
pub fn program_exists(program: &str) -> bool {
    which(program).is_some()
}

/// Replace an argument in a command script.
///
/// This function replaces the first occurrence of an argument in a command.
/// It tries to replace at the end of the script first (more common case),
/// then falls back to replacing anywhere in the script.
///
/// # Arguments
///
/// * `script` - The original command script
/// * `from` - The argument to replace
/// * `to` - The replacement argument
///
/// # Returns
///
/// The modified script with the argument replaced.
///
/// # Example
///
/// ```
/// use oops::utils::executables::replace_argument;
///
/// let script = "git statsu";
/// let fixed = replace_argument(script, "statsu", "status");
/// assert_eq!(fixed, "git status");
/// ```
pub fn replace_argument(script: &str, from: &str, to: &str) -> String {
    // Try to replace at the end first (most common case)
    let end_pattern = format!(r" {}$", regex::escape(from));
    if let Ok(re) = Regex::new(&end_pattern) {
        let replacement = format!(" {}", to);
        let result = re.replace(script, replacement.as_str());
        if result != script {
            return result.to_string();
        }
    }

    // Fall back to replacing in the middle (surrounded by spaces)
    let middle_pattern = format!(r" {} ", regex::escape(from));
    if let Ok(re) = Regex::new(&middle_pattern) {
        let replacement = format!(" {} ", to);
        let result = re.replace(script, replacement.as_str());
        if result != script {
            return result.to_string();
        }
    }

    // If still no match, try simple replacement as last resort
    // This handles cases where the argument might be at the start
    let start_pattern = format!(r"^{} ", regex::escape(from));
    if let Ok(re) = Regex::new(&start_pattern) {
        let replacement = format!("{} ", to);
        let result = re.replace(script, replacement.as_str());
        if result != script {
            return result.to_string();
        }
    }

    // No replacement made, return original
    script.to_string()
}

/// Replace all occurrences of an argument in a command script.
///
/// Unlike `replace_argument`, this replaces all occurrences.
///
/// # Arguments
///
/// * `script` - The original command script
/// * `from` - The argument to replace
/// * `to` - The replacement argument
///
/// # Returns
///
/// The modified script with all occurrences replaced.
pub fn replace_argument_all(script: &str, from: &str, to: &str) -> String {
    // Split by whitespace, replace matching parts, rejoin
    // This is more reliable than regex for handling multiple occurrences
    script
        .split_whitespace()
        .map(|part| if part == from { to } else { part })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_executables() {
        let executables = get_all_executables();
        // Should have found at least some executables
        assert!(!executables.is_empty());

        // On most systems, these common commands should exist
        #[cfg(unix)]
        {
            // At least one of these should exist
            let common_unix = ["ls", "cat", "echo", "sh"];
            let has_common = common_unix.iter().any(|cmd| executables.contains(*cmd));
            assert!(has_common, "Expected at least one common Unix command");
        }

        #[cfg(windows)]
        {
            // At least one of these should exist
            let common_windows = ["cmd", "powershell", "notepad"];
            let has_common = common_windows.iter().any(|cmd| executables.contains(*cmd));
            assert!(has_common, "Expected at least one common Windows command");
        }
    }

    #[test]
    fn test_get_all_executables_excludes_oops() {
        let executables = get_all_executables();
        // Should not contain oops/thefuck entries to prevent circular corrections
        assert!(!executables.contains("oops"));
        assert!(!executables.contains("thefuck"));
        assert!(!executables.contains("fuck"));
    }

    #[test]
    fn test_which_existing_program() {
        #[cfg(unix)]
        let program = "ls";
        #[cfg(windows)]
        let program = "cmd";

        let result = which(program);
        assert!(result.is_some());
        assert!(result.unwrap().exists());
    }

    #[test]
    fn test_which_nonexistent_program() {
        let result = which("nonexistent_program_xyz_123");
        assert!(result.is_none());
    }

    #[test]
    fn test_program_exists() {
        #[cfg(unix)]
        let program = "ls";
        #[cfg(windows)]
        let program = "cmd";

        assert!(program_exists(program));
        assert!(!program_exists("nonexistent_program_xyz_123"));
    }

    #[test]
    fn test_replace_argument_at_end() {
        let script = "git statsu";
        let fixed = replace_argument(script, "statsu", "status");
        assert_eq!(fixed, "git status");
    }

    #[test]
    fn test_replace_argument_in_middle() {
        let script = "git statsu --verbose";
        let fixed = replace_argument(script, "statsu", "status");
        assert_eq!(fixed, "git status --verbose");
    }

    #[test]
    fn test_replace_argument_at_start() {
        let script = "gti status";
        let fixed = replace_argument(script, "gti", "git");
        assert_eq!(fixed, "git status");
    }

    #[test]
    fn test_replace_argument_no_match() {
        let script = "git status";
        let fixed = replace_argument(script, "nonexistent", "something");
        assert_eq!(fixed, "git status");
    }

    #[test]
    fn test_replace_argument_partial_no_match() {
        // Should not match partial words
        let script = "git statuses";
        let fixed = replace_argument(script, "status", "something");
        // Should not replace "status" within "statuses"
        assert_eq!(fixed, "git statuses");
    }

    #[test]
    fn test_replace_argument_special_characters() {
        let script = "grep foo.bar file";
        let fixed = replace_argument(script, "foo.bar", "foo\\.bar");
        assert_eq!(fixed, "grep foo\\.bar file");
    }

    #[test]
    fn test_replace_argument_all() {
        let script = "echo test test test";
        let fixed = replace_argument_all(script, "test", "hello");
        // All occurrences should be replaced
        assert!(!fixed.contains("test"));
        assert!(fixed.contains("hello"));
    }

    #[test]
    fn test_replace_argument_with_flags() {
        let script = "npm instal -g typescript";
        let fixed = replace_argument(script, "instal", "install");
        assert_eq!(fixed, "npm install -g typescript");
    }
}
