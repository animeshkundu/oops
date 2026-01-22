//! Zsh shell integration
//!
//! Provides the Zsh implementation of the Shell trait, including:
//! - Alias generation for the `oops` command
//! - History reading from TF_HISTORY environment variable
//! - Alias parsing from TF_SHELL_ALIASES environment variable

use std::collections::HashMap;
use std::env;

use super::Shell;
use crate::cli::THEFUCK_ARGUMENT_PLACEHOLDER;

/// Zsh shell implementation.
#[derive(Debug, Clone, Default)]
pub struct Zsh;

impl Zsh {
    /// Creates a new Zsh shell instance.
    pub fn new() -> Self {
        Self
    }

    /// Parses a single zsh alias line.
    ///
    /// Zsh aliases are in the format: `name='value'` or `name="value"` (without "alias " prefix)
    ///
    /// # Arguments
    /// * `alias_line` - A single alias definition line
    ///
    /// # Returns
    /// Some((name, value)) if parsing succeeded, None otherwise.
    fn parse_alias(&self, alias_line: &str) -> Option<(String, String)> {
        // Find the first '=' to split name and value
        let eq_pos = alias_line.find('=')?;
        let name = alias_line[..eq_pos].trim().to_string();
        let mut value = alias_line[eq_pos + 1..].trim().to_string();

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

    /// Gets the history file name for zsh.
    ///
    /// Uses HISTFILE environment variable or defaults to ~/.zsh_history
    fn get_history_file(&self) -> String {
        env::var("HISTFILE").unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| "~".to_string());
            format!("{}/.zsh_history", home)
        })
    }

    /// Parses a zsh history line to extract the command.
    ///
    /// Zsh history format can be: `: timestamp:0;command` or just `command`
    ///
    /// # Arguments
    /// * `line` - A single history line
    ///
    /// # Returns
    /// The command portion of the history line.
    fn script_from_history(&self, line: &str) -> String {
        // Zsh extended history format: ": timestamp:0;command"
        if let Some(pos) = line.find(';') {
            line[pos + 1..].to_string()
        } else {
            line.to_string()
        }
    }
}

impl Shell for Zsh {
    fn name(&self) -> &str {
        "zsh"
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

        // Standard zsh alias function
        // Note: Variables must be declared WITHIN the function
        // Note: Zsh uses slightly different syntax than bash (no "function" keyword,
        // different quoting, print -s instead of history -s)
        format!(
            r#"{name} () {{
    TF_PYTHONIOENCODING=$PYTHONIOENCODING;
    export TF_SHELL=zsh;
    export TF_ALIAS={name};
    TF_SHELL_ALIASES=$(alias);
    export TF_SHELL_ALIASES;
    TF_HISTORY="$(fc -ln -10)";
    export TF_HISTORY;
    export PYTHONIOENCODING=utf-8;
    TF_CMD=$(
        oops {placeholder} $@
    ) && eval $TF_CMD;
    unset TF_HISTORY;
    export PYTHONIOENCODING=$TF_PYTHONIOENCODING;
    test -n "$TF_CMD" && print -s $TF_CMD;
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
            .map(|line| self.script_from_history(line))
            .filter(|cmd| !cmd.is_empty())
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

    fn get_builtin_commands(&self) -> &[&str] {
        // Zsh has additional builtins compared to bash
        &[
            "alias",
            "autoload",
            "bg",
            "bindkey",
            "break",
            "builtin",
            "case",
            "cd",
            "chdir",
            "command",
            "compctl",
            "compadd",
            "compdef",
            "compdump",
            "compinit",
            "complete",
            "continue",
            "declare",
            "dirs",
            "disable",
            "disown",
            "echo",
            "echotc",
            "echoti",
            "emulate",
            "enable",
            "eval",
            "exec",
            "exit",
            "export",
            "false",
            "fc",
            "fg",
            "float",
            "functions",
            "getcap",
            "getln",
            "getopts",
            "hash",
            "history",
            "if",
            "integer",
            "jobs",
            "kill",
            "let",
            "limit",
            "local",
            "log",
            "logout",
            "noglob",
            "popd",
            "print",
            "printf",
            "pushd",
            "pushln",
            "pwd",
            "read",
            "readonly",
            "rehash",
            "return",
            "sched",
            "set",
            "setcap",
            "setopt",
            "shift",
            "source",
            "stat",
            "suspend",
            "test",
            "times",
            "trap",
            "true",
            "ttyctl",
            "type",
            "typeset",
            "ulimit",
            "umask",
            "unalias",
            "unfunction",
            "unhash",
            "unlimit",
            "unset",
            "unsetopt",
            "until",
            "vared",
            "wait",
            "whence",
            "where",
            "which",
            "while",
            "zcompile",
            "zformat",
            "zle",
            "zmodload",
            "zparseopts",
            "zprof",
            "zpty",
            "zregexparse",
            "zsocket",
            "zstyle",
            "ztcp",
        ]
    }
}

impl Zsh {
    /// Generates the instant mode alias for zsh.
    ///
    /// This is used when instant mode is already active and we just need
    /// a simple function with PS1 marking for command capture.
    fn instant_mode_alias(&self, alias_name: &str) -> String {
        // User command mark for instant mode detection
        // Zsh requires %{ and %} around non-printing characters
        const USER_COMMAND_MARK: &str = "\x1b[9999;H";

        // Create backspaces to hide the marker
        let backspaces = "\x08".repeat(USER_COMMAND_MARK.len());
        let mark = format!("%{{{}{}%}}", USER_COMMAND_MARK, backspaces);

        format!(
            r#"export PS1="{mark}$PS1";
{name} () {{
    TF_PYTHONIOENCODING=$PYTHONIOENCODING;
    export TF_SHELL=zsh;
    export TF_ALIAS={name};
    TF_SHELL_ALIASES=$(alias);
    export TF_SHELL_ALIASES;
    TF_HISTORY="$(fc -ln -10)";
    export TF_HISTORY;
    export PYTHONIOENCODING=utf-8;
    TF_CMD=$(
        oops {placeholder} $@
    ) && eval $TF_CMD;
    unset TF_HISTORY;
    export PYTHONIOENCODING=$TF_PYTHONIOENCODING;
    test -n "$TF_CMD" && print -s $TF_CMD;
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
    fn test_zsh_name() {
        let zsh = Zsh::new();
        assert_eq!(zsh.name(), "zsh");
    }

    #[test]
    fn test_parse_alias_single_quotes() {
        let zsh = Zsh::new();
        let result = zsh.parse_alias("ll='ls -la'");
        assert_eq!(result, Some(("ll".to_string(), "ls -la".to_string())));
    }

    #[test]
    fn test_parse_alias_double_quotes() {
        let zsh = Zsh::new();
        let result = zsh.parse_alias("ll=\"ls -la\"");
        assert_eq!(result, Some(("ll".to_string(), "ls -la".to_string())));
    }

    #[test]
    fn test_parse_alias_no_quotes() {
        let zsh = Zsh::new();
        let result = zsh.parse_alias("ll=ls");
        assert_eq!(result, Some(("ll".to_string(), "ls".to_string())));
    }

    #[test]
    fn test_parse_alias_invalid() {
        let zsh = Zsh::new();
        let result = zsh.parse_alias("not an alias");
        assert_eq!(result, None);
    }

    #[test]
    fn test_script_from_history_simple() {
        let zsh = Zsh::new();
        let result = zsh.script_from_history("git status");
        assert_eq!(result, "git status");
    }

    #[test]
    fn test_script_from_history_extended_format() {
        let zsh = Zsh::new();
        // Zsh extended history format: ": timestamp:0;command"
        let result = zsh.script_from_history(": 1234567890:0;git status");
        assert_eq!(result, "git status");
    }

    #[test]
    fn test_get_history_from_env() {
        let _guard = crate::test_utils::EnvGuard::new(&["TF_HISTORY"]);
        env::set_var("TF_HISTORY", "git status\ncd /tmp\nls -la");
        let zsh = Zsh::new();
        let history = zsh.get_history();
        assert_eq!(history, vec!["git status", "cd /tmp", "ls -la"]);
    }

    #[test]
    fn test_get_aliases_from_env() {
        let _guard = crate::test_utils::EnvGuard::new(&["TF_SHELL_ALIASES"]);
        env::set_var("TF_SHELL_ALIASES", "ll='ls -la'\ngs='git status'");
        let zsh = Zsh::new();
        let aliases = zsh.get_aliases();
        assert_eq!(aliases.get("ll"), Some(&"ls -la".to_string()));
        assert_eq!(aliases.get("gs"), Some(&"git status".to_string()));
    }

    #[test]
    fn test_app_alias_contains_required_elements() {
        let zsh = Zsh::new();
        let alias = zsh.app_alias("fuck", false);

        // Check that the alias contains required elements
        // Note: Zsh doesn't use "function" keyword
        assert!(alias.contains("fuck () {"));
        assert!(alias.contains("export TF_SHELL=zsh"));
        assert!(alias.contains("export TF_ALIAS=fuck"));
        assert!(alias.contains("TF_SHELL_ALIASES=$(alias)"));
        assert!(alias.contains("export TF_SHELL_ALIASES"));
        assert!(alias.contains("TF_HISTORY=\"$(fc -ln -10)\""));
        assert!(alias.contains("export TF_HISTORY"));
        assert!(alias.contains("oops THEFUCK_ARGUMENT_PLACEHOLDER"));
        assert!(alias.contains("eval $TF_CMD"));
        assert!(alias.contains("print -s $TF_CMD"));
    }

    #[test]
    fn test_app_alias_custom_name() {
        let zsh = Zsh::new();
        let alias = zsh.app_alias("oops", false);

        assert!(alias.contains("oops () {"));
        assert!(alias.contains("export TF_ALIAS=oops"));
    }

    #[test]
    fn test_and_operator() {
        let zsh = Zsh::new();
        let result = zsh.and_(&["cmd1", "cmd2", "cmd3"]);
        assert_eq!(result, "cmd1 && cmd2 && cmd3");
    }

    #[test]
    fn test_or_operator() {
        let zsh = Zsh::new();
        let result = zsh.or_(&["cmd1", "cmd2"]);
        assert_eq!(result, "cmd1 || cmd2");
    }

    #[test]
    fn test_builtin_commands() {
        let zsh = Zsh::new();
        let builtins = zsh.get_builtin_commands();

        assert!(builtins.contains(&"cd"));
        assert!(builtins.contains(&"alias"));
        assert!(builtins.contains(&"export"));
        assert!(builtins.contains(&"print")); // Zsh-specific
        assert!(builtins.contains(&"setopt")); // Zsh-specific
        assert!(builtins.contains(&"zle")); // Zsh-specific
    }
}
