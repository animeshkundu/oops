//! Tcsh shell integration
//!
//! Provides the Tcsh shell implementation of the Shell trait, including:
//! - Alias generation for the `oops` command
//! - History reading from shell history file
//! - Alias parsing from tcsh alias output

use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};

use anyhow::Result;

use super::Shell;

/// Tcsh shell implementation.
#[derive(Debug, Clone, Default)]
pub struct Tcsh;

impl Tcsh {
    /// Creates a new Tcsh shell instance.
    pub fn new() -> Self {
        Self
    }

    /// Parses a single tcsh alias line.
    ///
    /// Tcsh alias output is tab-separated: `name\tvalue`
    ///
    /// # Arguments
    /// * `alias_line` - A single alias definition line
    ///
    /// # Returns
    /// Some((name, value)) if parsing succeeded, None otherwise.
    fn parse_alias(&self, alias_line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = alias_line.splitn(2, '\t').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// Gets the history file path for tcsh.
    fn get_history_file(&self) -> String {
        env::var("HISTFILE").unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|p| p.join(".history"))
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "~/.history".to_string())
        })
    }
}

impl Shell for Tcsh {
    fn name(&self) -> &str {
        "tcsh"
    }

    fn app_alias(&self, alias_name: &str, _instant_mode: bool) -> String {
        // Tcsh alias that:
        // 1. Sets TF_SHELL and TF_ALIAS environment variables
        // 2. Gets the last command from history
        // 3. Evaluates the oops output
        format!(
            "alias {name} 'setenv TF_SHELL tcsh && setenv TF_ALIAS {name} && \
             set fucked_cmd=`history -h 2 | head -n 1` && \
             eval `oops ${{fucked_cmd}}`'\n",
            name = alias_name
        )
    }

    fn get_history(&self) -> Vec<String> {
        // Tcsh doesn't use TF_HISTORY environment variable
        // History is retrieved via the alias command
        Vec::new()
    }

    fn get_aliases(&self) -> HashMap<String, String> {
        let mut aliases = HashMap::new();

        let output = Command::new("tcsh")
            .args(["-ic", "alias"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.split('\n') {
                if !line.is_empty() && line.contains('\t') {
                    if let Some((name, value)) = self.parse_alias(line) {
                        aliases.insert(name, value);
                    }
                }
            }
        }

        aliases
    }

    fn put_to_history(&self, command: &str) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};

        let history_file = self.get_history_file();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Tcsh history format: "#+<timestamp>\n<command>\n"
        let entry = format!("#+{}\n{}\n", timestamp, command);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(history_file)?;

        file.write_all(entry.as_bytes())?;
        Ok(())
    }

    fn get_history_file_name(&self) -> Option<String> {
        Some(self.get_history_file())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcsh_name() {
        let tcsh = Tcsh::new();
        assert_eq!(tcsh.name(), "tcsh");
    }

    #[test]
    fn test_tcsh_and_operator() {
        let tcsh = Tcsh::new();
        assert_eq!(tcsh.and_(&["cmd1", "cmd2"]), "cmd1 && cmd2");
        assert_eq!(tcsh.and_(&["cmd1", "cmd2", "cmd3"]), "cmd1 && cmd2 && cmd3");
    }

    #[test]
    fn test_tcsh_or_operator() {
        let tcsh = Tcsh::new();
        assert_eq!(tcsh.or_(&["cmd1", "cmd2"]), "cmd1 || cmd2");
        assert_eq!(tcsh.or_(&["cmd1", "cmd2", "cmd3"]), "cmd1 || cmd2 || cmd3");
    }

    #[test]
    fn test_tcsh_alias_generation() {
        let tcsh = Tcsh::new();
        let alias = tcsh.app_alias("fuck", false);
        assert!(alias.contains("alias fuck"));
        assert!(alias.contains("setenv TF_SHELL tcsh"));
        assert!(alias.contains("setenv TF_ALIAS fuck"));
        assert!(alias.contains("history -h 2 | head -n 1"));
        assert!(alias.contains("eval `oops ${fucked_cmd}`"));
    }

    #[test]
    fn test_tcsh_alias_custom_name() {
        let tcsh = Tcsh::new();
        let alias = tcsh.app_alias("oops", false);
        assert!(alias.contains("alias oops"));
        assert!(alias.contains("setenv TF_ALIAS oops"));
    }

    #[test]
    fn test_tcsh_parse_alias() {
        let tcsh = Tcsh::new();
        assert_eq!(
            tcsh.parse_alias("ll\tls -la"),
            Some(("ll".to_string(), "ls -la".to_string()))
        );
        assert_eq!(tcsh.parse_alias("no_tab"), None);
    }

    #[test]
    fn test_tcsh_empty_history() {
        let tcsh = Tcsh::new();
        let history = tcsh.get_history();
        assert!(history.is_empty());
    }

    #[test]
    fn test_builtin_commands() {
        let tcsh = Tcsh::new();
        let builtins = tcsh.get_builtin_commands();
        assert!(builtins.contains(&"cd"));
        assert!(builtins.contains(&"alias"));
    }
}
