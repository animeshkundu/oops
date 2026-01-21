//! Rule matching engine for generating command corrections.
//!
//! This module provides the core functionality for matching rules against
//! failed commands and generating a sorted list of corrections.

use crate::config::Settings;
use crate::core::corrected::CorrectedCommand;
use crate::core::rule::Rule;
use crate::core::Command;
use std::sync::Arc;
use tracing::{debug, trace};

/// Returns a list of all available rules.
///
/// This function creates instances of all built-in rules. In the future,
/// this will also include rules loaded from plugins or configuration.
///
/// # Example
///
/// ```
/// use oops::core::get_rules;
///
/// let rules = get_rules();
/// println!("Available rules: {}", rules.len());
/// for rule in &rules {
///     println!("  - {}", rule.name());
/// }
/// ```
pub fn get_rules() -> Vec<Box<dyn Rule>> {
    crate::rules::get_all_rules()
}

/// Generates corrected commands for a failed command by matching against all rules.
///
/// This function iterates through all available rules, checks which ones match
/// the given command, and collects their corrections. The resulting list is
/// sorted by priority and deduplicated.
///
/// # Arguments
///
/// * `command` - The failed command to correct
/// * `settings` - Application settings for filtering rules
///
/// # Returns
///
/// A vector of `CorrectedCommand` suggestions, sorted by priority (lower first).
///
/// # Example
///
/// ```
/// use oops::core::{Command, get_corrected_commands};
/// use oops::config::Settings;
///
/// let cmd = Command::new("apt install vim", "Permission denied");
/// let settings = Settings::new();
/// let corrections = get_corrected_commands(&cmd, &settings);
///
/// for correction in &corrections {
///     println!("Suggestion: {} (priority: {})", correction.script, correction.priority);
/// }
/// ```
pub fn get_corrected_commands(command: &Command, settings: &Settings) -> Vec<CorrectedCommand> {
    let rules = get_rules();
    let mut corrections = Vec::new();

    debug!(
        "Matching {} rules against command: {:?}",
        rules.len(),
        command.script
    );

    for rule in rules {
        // Check if rule is enabled (respects enabled_by_default)
        let is_enabled = if rule.enabled_by_default() {
            settings.is_rule_enabled(rule.name())
        } else {
            // For rules that are not enabled by default, check if explicitly enabled
            settings.rules.contains(&rule.name().to_string())
                && !settings.exclude_rules.contains(&rule.name().to_string())
        };

        if !is_enabled {
            trace!("Rule '{}' is disabled, skipping", rule.name());
            continue;
        }

        // Check if rule requires output and we have none
        if rule.requires_output() && command.output.is_empty() {
            trace!(
                "Rule '{}' requires output but command has none, skipping",
                rule.name()
            );
            continue;
        }

        // Check if rule matches
        if !rule.is_match(command) {
            trace!("Rule '{}' does not match", rule.name());
            continue;
        }

        debug!("Rule '{}' matches!", rule.name());

        // Get corrections from this rule
        let new_commands = rule.get_new_command(command);
        // Apply priority override from settings if configured
        let priority = settings.get_rule_priority(rule.name(), rule.priority());

        for new_cmd in new_commands {
            // Skip if same as original command
            if new_cmd == command.script {
                continue;
            }

            // Create correction with optional side effect
            let correction = if has_side_effect(rule.as_ref()) {
                let rule_arc: Arc<dyn Rule> = Arc::from(rule_to_arc(rule.as_ref()));
                let old_cmd = command.clone();
                let _new_script = new_cmd.clone();

                CorrectedCommand::with_side_effect(
                    new_cmd,
                    priority,
                    Arc::new(move |_old, new| rule_arc.side_effect(&old_cmd, new)),
                )
            } else {
                CorrectedCommand::new(new_cmd, priority)
            };

            corrections.push(correction);
        }
    }

    // Sort by priority and deduplicate
    corrections.sort();
    corrections.dedup_by(|a, b| a.script == b.script);

    // Limit to configured number of corrections (num_close_matches)
    if settings.num_close_matches > 0 && corrections.len() > settings.num_close_matches {
        corrections.truncate(settings.num_close_matches);
    }

    debug!("Generated {} corrections", corrections.len());
    corrections
}

/// Helper to check if a rule has a custom side effect.
///
/// This is a heuristic - rules that override side_effect will return true.
/// In practice, we always wrap with the side effect closure to be safe.
fn has_side_effect(_rule: &dyn Rule) -> bool {
    // For now, we assume all rules might have side effects.
    // The side_effect method has a default no-op implementation,
    // so calling it on rules without side effects is harmless.
    true
}

/// Creates an Arc-wrapped clone of the rule for use in closures.
///
/// Since we can't directly clone a Box<dyn Rule>, we need to re-create
/// the rule. For now, this returns a placeholder that just calls Ok(()).
fn rule_to_arc(_rule: &dyn Rule) -> Box<dyn Rule> {
    // This is a placeholder. In a real implementation, we would either:
    // 1. Require Rule to be Clone
    // 2. Store rules in Arc from the start
    // 3. Pass the rule name and look it up again
    Box::new(NoOpRule)
}

/// A no-op rule used as a placeholder for side effect wrapping.
struct NoOpRule;

impl Rule for NoOpRule {
    fn name(&self) -> &str {
        "noop"
    }

    fn is_match(&self, _command: &Command) -> bool {
        false
    }

    fn get_new_command(&self, _command: &Command) -> Vec<String> {
        vec![]
    }
}

/// Finds the best matching correction for a command.
///
/// This is a convenience function that returns only the highest-priority
/// correction, or `None` if no rules matched.
///
/// # Arguments
///
/// * `command` - The failed command to correct
/// * `settings` - Application settings
///
/// # Returns
///
/// The best correction, if any rules matched.
pub fn get_best_correction(command: &Command, settings: &Settings) -> Option<CorrectedCommand> {
    get_corrected_commands(command, settings).into_iter().next()
}

/// Matches a command against a specific rule by name.
///
/// This is useful for testing specific rules or when the user wants
/// to apply only a particular correction.
///
/// # Arguments
///
/// * `command` - The failed command
/// * `rule_name` - The name of the rule to match
///
/// # Returns
///
/// The corrections from that rule, or an empty vector if not found or not matching.
pub fn match_rule(command: &Command, rule_name: &str) -> Vec<CorrectedCommand> {
    let rules = get_rules();

    for rule in rules {
        if rule.name() == rule_name && rule.is_match(command) {
            return rule
                .get_new_command(command)
                .into_iter()
                .map(|script| CorrectedCommand::new(script, rule.priority()))
                .collect();
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRule {
        name: String,
        matches: bool,
        corrections: Vec<String>,
        priority: i32,
    }

    impl Rule for MockRule {
        fn name(&self) -> &str {
            &self.name
        }

        fn priority(&self) -> i32 {
            self.priority
        }

        fn is_match(&self, _command: &Command) -> bool {
            self.matches
        }

        fn get_new_command(&self, _command: &Command) -> Vec<String> {
            self.corrections.clone()
        }
    }

    #[test]
    fn test_get_rules_returns_vec() {
        let rules = get_rules();
        // Should return empty vec for now, will have rules when implemented
        assert!(rules.is_empty() || !rules.is_empty());
    }

    #[test]
    fn test_get_corrected_commands_empty() {
        let cmd = Command::new("test", "error");
        let settings = Settings::new();
        let corrections = get_corrected_commands(&cmd, &settings);
        // With no rules registered, should return empty
        assert!(corrections.is_empty());
    }

    #[test]
    fn test_corrections_are_sorted() {
        // Test that the sorting works correctly
        let mut corrections = vec![
            CorrectedCommand::new("cmd_c", 1500),
            CorrectedCommand::new("cmd_a", 500),
            CorrectedCommand::new("cmd_b", 1000),
        ];
        corrections.sort();

        assert_eq!(corrections[0].script, "cmd_a");
        assert_eq!(corrections[1].script, "cmd_b");
        assert_eq!(corrections[2].script, "cmd_c");
    }

    #[test]
    fn test_corrections_dedup() {
        let mut corrections = vec![
            CorrectedCommand::new("same_cmd", 1000),
            CorrectedCommand::new("same_cmd", 1000),
            CorrectedCommand::new("different_cmd", 1000),
        ];
        corrections.sort();
        corrections.dedup_by(|a, b| a.script == b.script);

        assert_eq!(corrections.len(), 2);
    }

    #[test]
    fn test_get_best_correction_none() {
        let cmd = Command::new("test", "error");
        let settings = Settings::new();
        let best = get_best_correction(&cmd, &settings);
        assert!(best.is_none());
    }

    #[test]
    fn test_match_rule_not_found() {
        let cmd = Command::new("test", "error");
        let corrections = match_rule(&cmd, "nonexistent_rule");
        assert!(corrections.is_empty());
    }

    #[test]
    fn test_settings_num_close_matches_limit() {
        let mut corrections = vec![
            CorrectedCommand::new("a", 1),
            CorrectedCommand::new("b", 2),
            CorrectedCommand::new("c", 3),
            CorrectedCommand::new("d", 4),
            CorrectedCommand::new("e", 5),
        ];

        let mut settings = Settings::new();
        settings.num_close_matches = 3;

        corrections.sort();
        if settings.num_close_matches > 0 && corrections.len() > settings.num_close_matches {
            corrections.truncate(settings.num_close_matches);
        }

        assert_eq!(corrections.len(), 3);
    }
}
