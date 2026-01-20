//! Fish shell integration
//!
//! Provides the Fish shell implementation of the Shell trait, including:
//! - Alias generation for the `oops` command
//! - History reading (Fish has its own history mechanism)
//! - Alias parsing from fish functions and aliases

use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};

use anyhow::Result;

use super::Shell;
use crate::cli::THEFUCK_ARGUMENT_PLACEHOLDER;

/// Fish shell implementation.
#[derive(Debug, Clone, Default)]
pub struct Fish;

impl Fish {
    /// Creates a new Fish shell instance.
    pub fn new() -> Self {
        Self
    }

    /// Get the list of overridden aliases that should be excluded from alias expansion.
    fn get_overridden_aliases(&self) -> Vec<String> {
        let overridden = env::var("THEFUCK_OVERRIDDEN_ALIASES")
            .or_else(|_| env::var("TF_OVERRIDDEN_ALIASES"))
            .unwrap_or_default();

        let mut defaults: Vec<String> = vec![
            "cd".to_string(),
            "grep".to_string(),
            "ls".to_string(),
            "man".to_string(),
            "open".to_string(),
        ];

        for alias in overridden.split(',') {
            let trimmed = alias.trim();
            if !trimmed.is_empty() && !defaults.contains(&trimmed.to_string()) {
                defaults.push(trimmed.to_string());
            }
        }

        defaults.sort();
        defaults
    }

    /// Get fish functions by running `fish -ic functions`.
    fn get_functions(&self, overridden: &[String]) -> HashMap<String, String> {
        let mut functions = HashMap::new();

        let output = Command::new("fish")
            .args(["-ic", "functions"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for func in stdout.trim().split('\n') {
                let func = func.trim();
                if !func.is_empty() && !overridden.contains(&func.to_string()) {
                    functions.insert(func.to_string(), func.to_string());
                }
            }
        }

        functions
    }

    /// Get fish aliases by running `fish -ic alias`.
    fn get_raw_aliases(&self, overridden: &[String]) -> HashMap<String, String> {
        let mut aliases = HashMap::new();

        let output = Command::new("fish")
            .args(["-ic", "alias"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let alias_out = stdout.trim();

            if alias_out.is_empty() {
                return aliases;
            }

            for alias_line in alias_out.split('\n') {
                // Remove 'alias ' prefix and split by ' ' or '='
                let line = alias_line.replace("alias ", "");

                // Try splitting by space first, then by '='
                let parts: Option<(String, String)> = if let Some(pos) = line.find(' ') {
                    let (name, value) = line.split_at(pos);
                    Some((name.to_string(), value[1..].to_string()))
                } else if let Some(pos) = line.find('=') {
                    let (name, value) = line.split_at(pos);
                    Some((name.to_string(), value[1..].to_string()))
                } else {
                    None
                };

                if let Some((name, value)) = parts {
                    if !overridden.contains(&name) {
                        aliases.insert(name, value);
                    }
                }
            }
        }

        aliases
    }

    /// Gets the history file path for fish.
    fn get_history_file(&self) -> String {
        dirs::config_dir()
            .map(|p| p.join("fish").join("fish_history"))
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| "~/.config/fish/fish_history".to_string())
    }
}

impl Shell for Fish {
    fn name(&self) -> &str {
        "fish"
    }

    fn app_alias(&self, alias_name: &str, _instant_mode: bool) -> String {
        // Fish shell alias with history modification
        // The alter_history behavior is always enabled for the Rust port
        // as it matches the expected Python behavior
        format!(
            r#"function {name} -d "Correct your previous console command"
    set -l fucked_up_command $history[1]
    env TF_SHELL=fish TF_ALIAS={name} PYTHONIOENCODING=utf-8 oops $fucked_up_command {placeholder} $argv | read -l unfucked_command
    if [ "$unfucked_command" != "" ]
        eval $unfucked_command
        builtin history delete --exact --case-sensitive -- $fucked_up_command
        builtin history merge
    end
end
"#,
            name = alias_name,
            placeholder = THEFUCK_ARGUMENT_PLACEHOLDER,
        )
    }

    fn get_history(&self) -> Vec<String> {
        // Fish doesn't use TF_HISTORY environment variable like bash/zsh
        // It reads directly from history. For now, return empty vec
        // as the alias passes history via command line arguments.
        Vec::new()
    }

    fn get_aliases(&self) -> HashMap<String, String> {
        let overridden = self.get_overridden_aliases();
        let mut aliases = self.get_functions(&overridden);
        let raw_aliases = self.get_raw_aliases(&overridden);
        aliases.extend(raw_aliases);
        aliases
    }

    fn and_(&self, commands: &[&str]) -> String {
        // Fish uses "; and " for command chaining
        commands.join("; and ")
    }

    fn or_(&self, commands: &[&str]) -> String {
        // Fish uses "; or " for or operations
        commands.join("; or ")
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

        // Fish history format: "- cmd: <command>\n  when: <timestamp>\n"
        let entry = format!("- cmd: {}\n  when: {}\n", command, timestamp);

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
    fn test_fish_name() {
        let fish = Fish::new();
        assert_eq!(fish.name(), "fish");
    }

    #[test]
    fn test_fish_and_operator() {
        let fish = Fish::new();
        assert_eq!(fish.and_(&["cmd1", "cmd2"]), "cmd1; and cmd2");
        assert_eq!(
            fish.and_(&["cmd1", "cmd2", "cmd3"]),
            "cmd1; and cmd2; and cmd3"
        );
    }

    #[test]
    fn test_fish_or_operator() {
        let fish = Fish::new();
        assert_eq!(fish.or_(&["cmd1", "cmd2"]), "cmd1; or cmd2");
        assert_eq!(
            fish.or_(&["cmd1", "cmd2", "cmd3"]),
            "cmd1; or cmd2; or cmd3"
        );
    }

    #[test]
    fn test_fish_alias_generation() {
        let fish = Fish::new();
        let alias = fish.app_alias("fuck", false);
        assert!(alias.contains("function fuck"));
        assert!(alias.contains("$history[1]"));
        assert!(alias.contains("TF_SHELL=fish"));
        assert!(alias.contains("TF_ALIAS=fuck"));
        assert!(alias.contains("eval $unfucked_command"));
        assert!(alias.contains("builtin history delete"));
        assert!(alias.contains("builtin history merge"));
    }

    #[test]
    fn test_fish_alias_custom_name() {
        let fish = Fish::new();
        let alias = fish.app_alias("oops", false);
        assert!(alias.contains("function oops"));
        assert!(alias.contains("TF_ALIAS=oops"));
    }

    #[test]
    fn test_builtin_commands() {
        let fish = Fish::new();
        let builtins = fish.get_builtin_commands();
        assert!(builtins.contains(&"cd"));
        assert!(builtins.contains(&"alias"));
    }
}
