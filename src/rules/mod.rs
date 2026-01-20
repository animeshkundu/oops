//! Correction rules for common command-line errors.
//!
//! This module contains all the rules used to detect and correct command failures.
//! Rules are organized by category:
//!
//! - [`git`] - Git-related rules
//! - [`sudo`] - Permission denied fixes (prepend sudo)
//! - [`cd`] - Directory navigation fixes
//! - [`typo`] - Common command typo corrections
//! - [`no_command`] - Command not found fixes
//! - [`package_managers`] - Package manager rules
//! - [`cloud`] - Cloud and network rules (AWS, Azure, Heroku, SSH, etc.)
//! - [`system`] - System and file operation rules (ls, cp, rm, mkdir, etc.)
//! - [`devtools`] - Development tool rules (Go, Java, Maven, Gradle, Terraform, etc.)
//! - [`frameworks`] - Language and framework rules (Python, Rails, React Native, Yarn, npm, etc.)
//! - [`shell_utils`] - Shell utility rules (grep, sed, adb, hg, history, etc.)
//! - [`misc`] - Miscellaneous correction rules

pub mod cd;
pub mod cloud;
pub mod devtools;
pub mod docker;
pub mod frameworks;
pub mod git;
pub mod misc;
pub mod no_command;
pub mod package_managers;
pub mod shell_utils;
pub mod sudo;
pub mod system;
pub mod typo;

use crate::core::Rule;

// Re-export commonly used rules
pub use cd::{CdCorrection, CdCs, CdMkdir, CdParent};
pub use no_command::NoCommand;
pub use sudo::Sudo;
pub use typo::{PythonCommand, SlLs, Systemctl};

/// Returns all built-in rules as boxed trait objects.
///
/// This function creates instances of all correction rules and returns them
/// ready for registration with the rule system. Rules are returned in a
/// reasonable priority order, but the actual execution priority is determined
/// by each rule's `priority()` method.
///
/// # Example
///
/// ```
/// use oops::rules::get_all_rules;
///
/// let rules = get_all_rules();
/// println!("Loaded {} rules", rules.len());
/// ```
pub fn get_all_rules() -> Vec<Box<dyn Rule>> {
    let mut rules: Vec<Box<dyn Rule>> = vec![];

    // High priority rules (quick fixes)
    rules.push(Box::new(Sudo));
    rules.push(Box::new(CdParent));
    rules.push(Box::new(CdMkdir));
    rules.push(Box::new(CdCorrection));
    rules.push(Box::new(CdCs));

    // Typo rules
    rules.push(Box::new(SlLs));
    rules.push(Box::new(PythonCommand));
    rules.push(Box::new(Systemctl));

    // Command not found (lower priority, does more work)
    rules.push(Box::new(NoCommand));

    // Add git rules (push, checkout, add, branch, common, not_command)
    rules.extend(git::all_rules());

    // Add package manager rules
    rules.extend(package_managers::all_rules());

    // Add docker and container rules
    rules.extend(docker::all_rules());

    // Add cloud and network rules
    rules.extend(cloud::all_rules());

    // Add system and file operation rules
    rules.extend(system::all_rules());

    // Add language and framework rules
    rules.extend(frameworks::all_rules());

    // Add shell utility rules
    rules.extend(shell_utils::all_rules());

    // Add development tool rules (Go, Java, Maven, Gradle, Terraform, etc.)
    rules.extend(devtools::all_rules());

    // Add miscellaneous rules
    rules.extend(misc::all_rules());

    rules
}

/// Returns the count of all registered rules.
///
/// This is useful for debugging and statistics.
pub fn rule_count() -> usize {
    get_all_rules().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_rules_not_empty() {
        let rules = get_all_rules();
        assert!(!rules.is_empty(), "Should have at least one rule");
    }

    #[test]
    fn test_all_rules_have_names() {
        let rules = get_all_rules();
        for rule in rules {
            assert!(!rule.name().is_empty(), "All rules should have names");
        }
    }

    #[test]
    fn test_no_duplicate_rule_names() {
        let rules = get_all_rules();
        let mut names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
        let original_len = names.len();
        names.sort();
        names.dedup();
        assert_eq!(
            names.len(),
            original_len,
            "Rule names should be unique"
        );
    }

    #[test]
    fn test_rule_count() {
        let count = rule_count();
        let rules = get_all_rules();
        assert_eq!(count, rules.len());
    }
}
