//! Bash shell integration
//!
//! Provides the Bash implementation of the Shell trait, including:
//! - Alias generation for the `oops` command
//! - History reading from TF_HISTORY environment variable
//! - Alias parsing from TF_SHELL_ALIASES environment variable

use std::collections::HashMap;
use std::env;

use super::Shell;
use crate::cli::THEFUCK_ARGUMENT_PLACEHOLDER;

/// Bash shell implementation.
#[derive(Debug, Clone, Default)]
pub struct Bash;

impl Bash {
    /// Creates a new Bash shell instance.
    pub fn new() -> Self {
        Self
    }

    /// Parses a single bash alias line.
    ///
    /// Bash aliases are in the format: `alias name='value'` or `alias name="value"`
    ///
    /// # Arguments
    /// * `alias_line` - A single alias definition line
    ///
    /// # Returns
    /// Some((name, value)) if parsing succeeded, None otherwise.
    fn parse_alias(&self, alias_line: &str) -> Option<(String, String)> {
        // Remove "alias " prefix if present
        let line = alias_line.strip_prefix("alias ").unwrap_or(alias_line);

        // Find the first '=' to split name and value
        let eq_pos = line.find('=')?;
        let name = line[..eq_pos].trim().to_string();
        let mut value = line[eq_pos + 1..].trim().to_string();

        // Remove surrounding quotes if present
        if ((value.starts_with('\'') && value.ends_with('\''))
            || (value.starts_with('"') && value.ends_with('"')))
            && value.len() >= 2
        {
            value = value[1..value.len() - 1].to_string();
        }

        if name.is_empty() {
            return None;
        }

        Some((name, value))
    }

    /// Gets the history file name for bash.
    ///
    /// Uses HISTFILE environment variable or defaults to ~/.bash_history
    fn get_history_file(&self) -> String {
        env::var("HISTFILE").unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "~".to_string());
            format!("{}/.bash_history", home)
        })
    }
}

impl Shell for Bash {
    fn name(&self) -> &str {
        "bash"
    }

    fn app_alias(&self, alias_name: &str, instant_mode: bool) -> String {
        // If instant mode is enabled and we're already in instant mode,
        // generate a simpler alias. Otherwise, generate the full function.
        if instant_mode {
            // Check if we're already in instant mode
            if env::var("THEFUCK_INSTANT_MODE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false)
            {
                // Already in instant mode, use regular alias with PS1 marker
                return self.instant_mode_alias(alias_name);
            }
        }

        // Standard bash alias function
        // Note: Variables must be declared WITHIN the function
        format!(
            r#"function {name} () {{
    TF_PYTHONIOENCODING=$PYTHONIOENCODING;
    export TF_SHELL=bash;
    export TF_ALIAS={name};
    export TF_SHELL_ALIASES=$(alias);
    export TF_HISTORY=$(fc -ln -10);
    export PYTHONIOENCODING=utf-8;
    TF_CMD=$(
        oops {placeholder} "$@"
    ) && eval "$TF_CMD";
    unset TF_HISTORY;
    export PYTHONIOENCODING=$TF_PYTHONIOENCODING;
    history -s $TF_CMD;
}}
"#,
            name = alias_name,
            placeholder = THEFUCK_ARGUMENT_PLACEHOLDER,
        )
    }

    fn get_history(&self) -> Vec<String> {
        // Read history from TF_HISTORY environment variable
        // This is set by the shell alias function before calling oops
        let history_str = env::var("TF_HISTORY").unwrap_or_default();

        history_str
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect()
    }

    fn get_aliases(&self) -> HashMap<String, String> {
        // Read aliases from TF_SHELL_ALIASES environment variable
        // This is set by the shell alias function before calling oops
        let aliases_str = env::var("TF_SHELL_ALIASES").unwrap_or_default();

        aliases_str
            .lines()
            .filter_map(|line| self.parse_alias(line))
            .collect()
    }

    fn get_history_file_name(&self) -> Option<String> {
        Some(self.get_history_file())
    }
}

impl Bash {
    /// Generates the instant mode alias for bash.
    ///
    /// This is used when instant mode is already active and we just need
    /// a simple function with PS1 marking for command capture.
    fn instant_mode_alias(&self, alias_name: &str) -> String {
        // User command mark for instant mode detection
        const USER_COMMAND_MARK: &str = "\x1b[9999;H";

        // Create backspaces to hide the marker
        let backspaces = "\x08".repeat(USER_COMMAND_MARK.len());
        let mark = format!("{}{}", USER_COMMAND_MARK, backspaces);

        format!(
            r#"export PS1="{mark}$PS1";
function {name} () {{
    TF_PYTHONIOENCODING=$PYTHONIOENCODING;
    export TF_SHELL=bash;
    export TF_ALIAS={name};
    export TF_SHELL_ALIASES=$(alias);
    export TF_HISTORY=$(fc -ln -10);
    export PYTHONIOENCODING=utf-8;
    TF_CMD=$(
        oops {placeholder} "$@"
    ) && eval "$TF_CMD";
    unset TF_HISTORY;
    export PYTHONIOENCODING=$TF_PYTHONIOENCODING;
    history -s $TF_CMD;
}}
"#,
            name = alias_name,
            placeholder = THEFUCK_ARGUMENT_PLACEHOLDER,
            mark = mark,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_name() {
        let bash = Bash::new();
        assert_eq!(bash.name(), "bash");
    }

    #[test]
    fn test_parse_alias_single_quotes() {
        let bash = Bash::new();
        let result = bash.parse_alias("alias ll='ls -la'");
        assert_eq!(result, Some(("ll".to_string(), "ls -la".to_string())));
    }

    #[test]
    fn test_parse_alias_double_quotes() {
        let bash = Bash::new();
        let result = bash.parse_alias("alias ll=\"ls -la\"");
        assert_eq!(result, Some(("ll".to_string(), "ls -la".to_string())));
    }

    #[test]
    fn test_parse_alias_no_quotes() {
        let bash = Bash::new();
        let result = bash.parse_alias("alias ll=ls");
        assert_eq!(result, Some(("ll".to_string(), "ls".to_string())));
    }

    #[test]
    fn test_parse_alias_without_prefix() {
        let bash = Bash::new();
        let result = bash.parse_alias("ll='ls -la'");
        assert_eq!(result, Some(("ll".to_string(), "ls -la".to_string())));
    }

    #[test]
    fn test_parse_alias_invalid() {
        let bash = Bash::new();
        let result = bash.parse_alias("not an alias");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_history_from_env() {
        env::set_var("TF_HISTORY", "git status\ncd /tmp\nls -la");
        let bash = Bash::new();
        let history = bash.get_history();
        assert_eq!(history, vec!["git status", "cd /tmp", "ls -la"]);
        env::remove_var("TF_HISTORY");
    }

    #[test]
    fn test_get_aliases_from_env() {
        env::set_var(
            "TF_SHELL_ALIASES",
            "alias ll='ls -la'\nalias gs='git status'",
        );
        let bash = Bash::new();
        let aliases = bash.get_aliases();
        assert_eq!(aliases.get("ll"), Some(&"ls -la".to_string()));
        assert_eq!(aliases.get("gs"), Some(&"git status".to_string()));
        env::remove_var("TF_SHELL_ALIASES");
    }

    #[test]
    fn test_app_alias_contains_required_elements() {
        let bash = Bash::new();
        let alias = bash.app_alias("fuck", false);

        // Check that the alias contains required elements
        assert!(alias.contains("function fuck ()"));
        assert!(alias.contains("export TF_SHELL=bash"));
        assert!(alias.contains("export TF_ALIAS=fuck"));
        assert!(alias.contains("export TF_SHELL_ALIASES=$(alias)"));
        assert!(alias.contains("export TF_HISTORY=$(fc -ln -10)"));
        assert!(alias.contains("oops THEFUCK_ARGUMENT_PLACEHOLDER"));
        assert!(alias.contains("eval \"$TF_CMD\""));
        assert!(alias.contains("history -s $TF_CMD"));
    }

    #[test]
    fn test_app_alias_custom_name() {
        let bash = Bash::new();
        let alias = bash.app_alias("oops", false);

        assert!(alias.contains("function oops ()"));
        assert!(alias.contains("export TF_ALIAS=oops"));
    }

    #[test]
    fn test_and_operator() {
        let bash = Bash::new();
        let result = bash.and_(&["cmd1", "cmd2", "cmd3"]);
        assert_eq!(result, "cmd1 && cmd2 && cmd3");
    }

    #[test]
    fn test_or_operator() {
        let bash = Bash::new();
        let result = bash.or_(&["cmd1", "cmd2"]);
        assert_eq!(result, "cmd1 || cmd2");
    }

    #[test]
    fn test_builtin_commands() {
        let bash = Bash::new();
        let builtins = bash.get_builtin_commands();

        assert!(builtins.contains(&"cd"));
        assert!(builtins.contains(&"alias"));
        assert!(builtins.contains(&"export"));
        assert!(builtins.contains(&"history"));
    }
}
