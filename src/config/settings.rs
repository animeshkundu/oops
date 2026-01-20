//! Configuration settings for oops.
//!
//! This module defines the `Settings` struct which holds all configuration
//! options for oops. Settings can be loaded from:
//! 1. Default values
//! 2. Configuration file (~/.config/oops/config.toml)
//! 3. Environment variables (THEFUCK_* for backward compatibility)
//! 4. CLI arguments

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main settings structure for oops configuration.
///
/// Settings are loaded in order of priority (later sources override earlier):
/// 1. Default values
/// 2. Settings file (~/.config/oops/config.toml)
/// 3. Environment variables (THEFUCK_* for backward compatibility)
/// 4. CLI arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// List of enabled rules. Use ["ALL"] to enable all rules,
    /// or specify individual rule names.
    pub rules: Vec<String>,

    /// List of rules to exclude from matching.
    pub exclude_rules: Vec<String>,

    /// Whether to require confirmation before executing a fix.
    /// Default: true
    pub require_confirmation: bool,

    /// Maximum time (in seconds) to wait for a command to complete.
    /// Default: 3 seconds
    pub wait_command: u64,

    /// Maximum time (in seconds) to wait for slow commands.
    /// Default: 15 seconds
    pub wait_slow_command: u64,

    /// Disable colored output.
    pub no_colors: bool,

    /// Custom priority overrides for rules.
    /// Higher priority rules are suggested first.
    pub priority: HashMap<String, i32>,

    /// Maximum number of history entries to search.
    /// None means unlimited.
    pub history_limit: Option<usize>,

    /// Whether to alter shell history when executing a fix.
    /// Default: true
    pub alter_history: bool,

    /// List of commands that are known to be slow.
    /// These commands get longer timeouts.
    pub slow_commands: Vec<String>,

    /// Number of close matches to show when suggesting corrections.
    /// Default: 3
    pub num_close_matches: usize,

    /// Path prefixes to exclude when searching for executables.
    pub excluded_search_path_prefixes: Vec<String>,

    /// Extra environment variables to set when running commands.
    pub env: HashMap<String, String>,

    /// Enable experimental instant mode.
    /// This mode pre-executes commands to reduce latency.
    pub instant_mode: bool,

    /// Enable debug output.
    pub debug: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rules: vec!["ALL".to_string()],
            exclude_rules: Vec::new(),
            require_confirmation: true,
            wait_command: 3,
            wait_slow_command: 15,
            no_colors: false,
            priority: HashMap::new(),
            history_limit: None,
            alter_history: true,
            slow_commands: vec![
                "lein".to_string(),
                "react-native".to_string(),
                "gradle".to_string(),
                "./gradlew".to_string(),
                "vagrant".to_string(),
            ],
            num_close_matches: 3,
            excluded_search_path_prefixes: Vec::new(),
            env: HashMap::new(),
            instant_mode: false,
            debug: false,
        }
    }
}

impl Settings {
    /// Create a new Settings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a specific rule is enabled.
    ///
    /// A rule is enabled if:
    /// - "ALL" is in the rules list and the rule is not in exclude_rules, OR
    /// - The rule name is explicitly in the rules list
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        // Check if rule is explicitly excluded
        if self.exclude_rules.contains(&rule_name.to_string()) {
            return false;
        }

        // Check if ALL rules are enabled or this specific rule
        self.rules.contains(&"ALL".to_string()) || self.rules.contains(&rule_name.to_string())
    }

    /// Get the priority for a specific rule.
    ///
    /// Returns the custom priority if set, otherwise the default (1000).
    pub fn get_rule_priority(&self, rule_name: &str, default_priority: i32) -> i32 {
        self.priority
            .get(rule_name)
            .copied()
            .unwrap_or(default_priority)
    }

    /// Check if a command is considered "slow".
    ///
    /// Slow commands get longer timeouts.
    pub fn is_slow_command(&self, command: &str) -> bool {
        // Get the first word (command name) from the command string
        let cmd_name = command.split_whitespace().next().unwrap_or("");

        self.slow_commands
            .iter()
            .any(|slow_cmd| cmd_name == slow_cmd || cmd_name.ends_with(slow_cmd))
    }

    /// Get the appropriate wait time for a command.
    ///
    /// Returns `wait_slow_command` for slow commands, `wait_command` otherwise.
    pub fn get_wait_time(&self, command: &str) -> u64 {
        if self.is_slow_command(command) {
            self.wait_slow_command
        } else {
            self.wait_command
        }
    }

    /// Merge settings from another Settings instance.
    ///
    /// Values from `other` override values in `self` only if they differ from defaults.
    /// This is useful for layering configuration from multiple sources.
    pub fn merge(&mut self, other: &Settings) {
        let defaults = Settings::default();

        // Only override if different from default
        if other.rules != defaults.rules {
            self.rules = other.rules.clone();
        }
        if other.exclude_rules != defaults.exclude_rules {
            self.exclude_rules = other.exclude_rules.clone();
        }
        if other.require_confirmation != defaults.require_confirmation {
            self.require_confirmation = other.require_confirmation;
        }
        if other.wait_command != defaults.wait_command {
            self.wait_command = other.wait_command;
        }
        if other.wait_slow_command != defaults.wait_slow_command {
            self.wait_slow_command = other.wait_slow_command;
        }
        if other.no_colors != defaults.no_colors {
            self.no_colors = other.no_colors;
        }
        if other.priority != defaults.priority {
            self.priority.extend(other.priority.clone());
        }
        if other.history_limit != defaults.history_limit {
            self.history_limit = other.history_limit;
        }
        if other.alter_history != defaults.alter_history {
            self.alter_history = other.alter_history;
        }
        if other.slow_commands != defaults.slow_commands {
            self.slow_commands = other.slow_commands.clone();
        }
        if other.num_close_matches != defaults.num_close_matches {
            self.num_close_matches = other.num_close_matches;
        }
        if other.excluded_search_path_prefixes != defaults.excluded_search_path_prefixes {
            self.excluded_search_path_prefixes = other.excluded_search_path_prefixes.clone();
        }
        if other.env != defaults.env {
            self.env.extend(other.env.clone());
        }
        if other.instant_mode != defaults.instant_mode {
            self.instant_mode = other.instant_mode;
        }
        if other.debug != defaults.debug {
            self.debug = other.debug;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.rules, vec!["ALL"]);
        assert!(settings.exclude_rules.is_empty());
        assert!(settings.require_confirmation);
        assert_eq!(settings.wait_command, 3);
        assert_eq!(settings.wait_slow_command, 15);
        assert!(!settings.no_colors);
        assert!(settings.priority.is_empty());
        assert!(settings.history_limit.is_none());
        assert!(settings.alter_history);
        assert_eq!(settings.num_close_matches, 3);
        assert!(!settings.instant_mode);
        assert!(!settings.debug);
    }

    #[test]
    fn test_is_rule_enabled_all() {
        let settings = Settings::default();
        assert!(settings.is_rule_enabled("git_push"));
        assert!(settings.is_rule_enabled("sudo"));
    }

    #[test]
    fn test_is_rule_enabled_excluded() {
        let mut settings = Settings::default();
        settings.exclude_rules = vec!["sudo".to_string()];
        assert!(settings.is_rule_enabled("git_push"));
        assert!(!settings.is_rule_enabled("sudo"));
    }

    #[test]
    fn test_is_rule_enabled_specific() {
        let mut settings = Settings::default();
        settings.rules = vec!["sudo".to_string(), "git_push".to_string()];
        assert!(settings.is_rule_enabled("git_push"));
        assert!(settings.is_rule_enabled("sudo"));
        assert!(!settings.is_rule_enabled("cd_mkdir"));
    }

    #[test]
    fn test_get_rule_priority() {
        let mut settings = Settings::default();
        settings.priority.insert("sudo".to_string(), 500);

        assert_eq!(settings.get_rule_priority("sudo", 1000), 500);
        assert_eq!(settings.get_rule_priority("git_push", 1000), 1000);
    }

    #[test]
    fn test_is_slow_command() {
        let settings = Settings::default();
        assert!(settings.is_slow_command("gradle build"));
        assert!(settings.is_slow_command("./gradlew test"));
        assert!(settings.is_slow_command("lein repl"));
        assert!(settings.is_slow_command("vagrant up"));
        assert!(!settings.is_slow_command("git status"));
        assert!(!settings.is_slow_command("ls -la"));
    }

    #[test]
    fn test_get_wait_time() {
        let settings = Settings::default();
        assert_eq!(settings.get_wait_time("git status"), 3);
        assert_eq!(settings.get_wait_time("gradle build"), 15);
    }

    #[test]
    fn test_merge_settings() {
        let mut base = Settings::default();
        let mut override_settings = Settings::default();

        override_settings.debug = true;
        override_settings.wait_command = 5;
        override_settings.priority.insert("sudo".to_string(), 100);

        base.merge(&override_settings);

        assert!(base.debug);
        assert_eq!(base.wait_command, 5);
        assert_eq!(base.get_rule_priority("sudo", 1000), 100);
        // Should keep defaults for unmodified values
        assert!(base.require_confirmation);
    }

    #[test]
    fn test_serialization() {
        let settings = Settings::default();
        let toml_str = toml::to_string(&settings).unwrap();
        let parsed: Settings = toml::from_str(&toml_str).unwrap();

        assert_eq!(settings.rules, parsed.rules);
        assert_eq!(settings.wait_command, parsed.wait_command);
    }
}
