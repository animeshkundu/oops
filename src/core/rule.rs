//! Rule trait and helper functions for command correction rules.

use crate::core::Command;
use anyhow::Result;
use std::marker::PhantomData;

/// A rule for correcting failed commands.
///
/// Rules are the core mechanism for detecting and fixing command errors.
/// Each rule checks if it matches a failed command and provides one or
/// more suggested corrections.
///
/// # Implementing a Rule
///
/// ```
/// use oops::core::{Command, Rule};
/// use anyhow::Result;
///
/// struct SudoRule;
///
/// impl Rule for SudoRule {
///     fn name(&self) -> &str {
///         "sudo"
///     }
///
///     fn is_match(&self, command: &Command) -> bool {
///         command.output.contains("Permission denied")
///             || command.output.contains("EACCES")
///     }
///
///     fn get_new_command(&self, command: &Command) -> Vec<String> {
///         vec![format!("sudo {}", command.script)]
///     }
/// }
/// ```
pub trait Rule: Send + Sync {
    /// Returns the unique name of this rule.
    ///
    /// The name is used for configuration (enabling/disabling rules)
    /// and for debugging/logging purposes.
    fn name(&self) -> &str;

    /// Returns the priority of this rule.
    ///
    /// Lower values indicate higher priority. The default priority is 1000.
    /// Rules with lower priority values will have their corrections shown first.
    fn priority(&self) -> i32 {
        1000
    }

    /// Returns whether this rule is enabled by default.
    ///
    /// Some rules may be experimental or have side effects that make them
    /// unsuitable for automatic enabling.
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Returns whether this rule requires command output to match.
    ///
    /// Some rules can match based solely on the command script without
    /// needing to examine the output. Setting this to `false` allows
    /// such rules to match even when output capture failed.
    fn requires_output(&self) -> bool {
        true
    }

    /// Checks if this rule matches the given command.
    ///
    /// Returns `true` if this rule can provide a correction for the command.
    fn is_match(&self, command: &Command) -> bool;

    /// Returns one or more corrected command scripts.
    ///
    /// This method is only called if `is_match` returns `true`.
    /// Returns an empty vector if no corrections can be generated.
    fn get_new_command(&self, command: &Command) -> Vec<String>;

    /// Performs any side effects needed after the corrected command runs.
    ///
    /// Some rules need to perform additional actions after the corrected
    /// command is executed, such as updating shell aliases or environment
    /// variables.
    ///
    /// # Arguments
    ///
    /// * `old_cmd` - The original failed command
    /// * `new_script` - The corrected command script that was executed
    fn side_effect(&self, _old_cmd: &Command, _new_script: &str) -> Result<()> {
        Ok(())
    }
}

/// Checks if a command starts with any of the given application names.
///
/// This is a helper function commonly used in rule implementations to
/// check if a command is invoking a specific application.
///
/// # Arguments
///
/// * `command` - The command to check
/// * `app_names` - A slice of application names to match against
///
/// # Example
///
/// ```
/// use oops::core::{Command, is_app};
///
/// let cmd = Command::new("git push origin master", "");
/// assert!(is_app(&cmd, &["git"]));
/// assert!(!is_app(&cmd, &["svn", "hg"]));
/// ```
pub fn is_app(command: &Command, app_names: &[&str]) -> bool {
    let parts = command.script_parts();
    if parts.is_empty() {
        return false;
    }

    let first_part = &parts[0];

    // Check exact match first
    for &app in app_names {
        if first_part == app {
            return true;
        }
    }

    // Also check if the command ends with the app name (for paths like /usr/bin/git)
    for &app in app_names {
        if first_part.ends_with(&format!("/{}", app))
            || first_part.ends_with(&format!("\\{}", app))
            || first_part.ends_with(&format!("/{}.exe", app))
            || first_part.ends_with(&format!("\\{}.exe", app))
        {
            return true;
        }
    }

    false
}

/// A wrapper that makes a rule only match for specific applications.
///
/// This is useful for creating app-specific rules that should only
/// trigger when certain commands are being used.
///
/// # Example
///
/// ```
/// use oops::core::{Command, Rule, for_app};
///
/// struct GitPushRule;
///
/// impl Rule for GitPushRule {
///     fn name(&self) -> &str { "git_push" }
///     fn is_match(&self, cmd: &Command) -> bool {
///         cmd.output.contains("failed to push")
///     }
///     fn get_new_command(&self, cmd: &Command) -> Vec<String> {
///         vec![format!("{} --force", cmd.script)]
///     }
/// }
///
/// // This rule will only match if the command starts with "git"
/// let git_only_rule = for_app(GitPushRule, &["git"]);
/// ```
pub struct ForAppRule<R: Rule> {
    inner: R,
    app_names: Vec<String>,
    _marker: PhantomData<R>,
}

impl<R: Rule> ForAppRule<R> {
    /// Creates a new ForAppRule wrapping the given rule.
    pub fn new(rule: R, app_names: &[&str]) -> Self {
        Self {
            inner: rule,
            app_names: app_names.iter().map(|&s| s.to_string()).collect(),
            _marker: PhantomData,
        }
    }
}

impl<R: Rule> Rule for ForAppRule<R> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn priority(&self) -> i32 {
        self.inner.priority()
    }

    fn enabled_by_default(&self) -> bool {
        self.inner.enabled_by_default()
    }

    fn requires_output(&self) -> bool {
        self.inner.requires_output()
    }

    fn is_match(&self, command: &Command) -> bool {
        let app_refs: Vec<&str> = self.app_names.iter().map(|s| s.as_str()).collect();
        is_app(command, &app_refs) && self.inner.is_match(command)
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        self.inner.get_new_command(command)
    }

    fn side_effect(&self, old_cmd: &Command, new_script: &str) -> Result<()> {
        self.inner.side_effect(old_cmd, new_script)
    }
}

/// Creates a ForAppRule that wraps the given rule for specific applications.
///
/// # Arguments
///
/// * `rule` - The rule to wrap
/// * `app_names` - The application names that must match for this rule to trigger
///
/// # Example
///
/// ```
/// use oops::core::{Rule, for_app};
///
/// struct MyRule;
/// impl Rule for MyRule {
///     fn name(&self) -> &str { "my_rule" }
///     fn is_match(&self, _: &oops::core::Command) -> bool { true }
///     fn get_new_command(&self, _: &oops::core::Command) -> Vec<String> { vec![] }
/// }
///
/// let git_rule = for_app(MyRule, &["git", "hub"]);
/// ```
pub fn for_app<R: Rule>(rule: R, app_names: &[&str]) -> ForAppRule<R> {
    ForAppRule::new(rule, app_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRule;

    impl Rule for TestRule {
        fn name(&self) -> &str {
            "test_rule"
        }

        fn is_match(&self, command: &Command) -> bool {
            command.output.contains("error")
        }

        fn get_new_command(&self, command: &Command) -> Vec<String> {
            vec![format!("fixed {}", command.script)]
        }
    }

    #[test]
    fn test_is_app_exact_match() {
        let cmd = Command::new("git status", "");
        assert!(is_app(&cmd, &["git"]));
        assert!(!is_app(&cmd, &["svn"]));
    }

    #[test]
    fn test_is_app_multiple_names() {
        let cmd = Command::new("git status", "");
        assert!(is_app(&cmd, &["svn", "git", "hg"]));
    }

    #[test]
    fn test_is_app_with_path() {
        let cmd = Command::new("/usr/bin/git status", "");
        assert!(is_app(&cmd, &["git"]));
    }

    #[test]
    fn test_is_app_empty_command() {
        let cmd = Command::new("", "");
        assert!(!is_app(&cmd, &["git"]));
    }

    #[test]
    fn test_for_app_rule_matches() {
        let rule = for_app(TestRule, &["git"]);

        let matching_cmd = Command::new("git push", "error: failed");
        assert!(rule.is_match(&matching_cmd));

        let wrong_app_cmd = Command::new("svn commit", "error: failed");
        assert!(!rule.is_match(&wrong_app_cmd));

        let no_error_cmd = Command::new("git push", "success");
        assert!(!rule.is_match(&no_error_cmd));
    }

    #[test]
    fn test_for_app_rule_delegates() {
        let rule = for_app(TestRule, &["git"]);
        assert_eq!(rule.name(), "test_rule");
        assert_eq!(rule.priority(), 1000);
        assert!(rule.enabled_by_default());
        assert!(rule.requires_output());
    }

    #[test]
    fn test_for_app_rule_get_new_command() {
        let rule = for_app(TestRule, &["git"]);
        let cmd = Command::new("git push", "error");
        let corrections = rule.get_new_command(&cmd);
        assert_eq!(corrections, vec!["fixed git push"]);
    }
}
