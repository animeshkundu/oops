//! Memoization utilities using the `cached` crate.
//!
//! This module provides cached versions of commonly used functions
//! to improve performance by avoiding redundant computations.

use cached::proc_macro::cached;
use std::path::PathBuf;

/// Finds the path to a program in the system PATH.
///
/// This is a cached wrapper around the `which` crate's functionality.
/// Results are memoized to avoid repeated PATH lookups for the same program.
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
/// use oops::utils::cache::which;
///
/// if let Some(path) = which("git".to_string()) {
///     println!("git is at: {:?}", path);
/// }
/// ```
#[cached(size = 100)]
pub fn which(program: String) -> Option<PathBuf> {
    ::which::which(&program).ok()
}

/// Checks if a program exists in PATH.
///
/// This is a convenience function that returns a boolean instead of an Option.
/// Uses the cached `which` function internally.
///
/// # Arguments
///
/// * `program` - The name of the program to check
///
/// # Returns
///
/// * `true` - If the program exists in PATH
/// * `false` - If the program does not exist in PATH
#[cached(size = 100)]
pub fn program_exists(program: String) -> bool {
    which(program).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_existing_program() {
        // These programs should exist on most systems
        #[cfg(unix)]
        let program = "ls";
        #[cfg(windows)]
        let program = "cmd";

        let result = which(program.to_string());
        assert!(result.is_some());
    }

    #[test]
    fn test_which_nonexistent_program() {
        let result = which("nonexistent_program_xyz_123".to_string());
        assert!(result.is_none());
    }

    #[test]
    fn test_program_exists() {
        #[cfg(unix)]
        let program = "ls";
        #[cfg(windows)]
        let program = "cmd";

        assert!(program_exists(program.to_string()));
        assert!(!program_exists("nonexistent_program_xyz_123".to_string()));
    }

    #[test]
    fn test_which_caching() {
        // Call which multiple times with the same argument
        // The cached version should return the same result
        #[cfg(unix)]
        let program = "ls";
        #[cfg(windows)]
        let program = "cmd";

        let result1 = which(program.to_string());
        let result2 = which(program.to_string());
        assert_eq!(result1, result2);
    }
}
