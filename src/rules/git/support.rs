//! Git support utilities for implementing git-related rules.
//!
//! This module provides helper functions and a wrapper type for git rules.

use regex::Regex;
use std::path::Path;
use std::process::Command as ProcessCommand;

// Re-export core types for use by git rules
pub use crate::core::{Command, Rule};

/// Check if command is a git command (git or hub).
pub fn is_git_command(cmd: &Command) -> bool {
    is_app(cmd, &["git", "hub"])
}

/// Check if a command uses one of the specified app names.
pub fn is_app(cmd: &Command, app_names: &[&str]) -> bool {
    let parts = cmd.script_parts();
    if let Some(first) = parts.first() {
        let basename = Path::new(first)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(first.as_str());
        // Remove .exe extension on Windows
        let basename = basename.strip_suffix(".exe").unwrap_or(basename);
        return app_names.contains(&basename);
    }
    false
}

/// Expand git aliases from trace output.
///
/// When GIT_TRACE is enabled, git outputs alias expansion information.
/// This function parses that output and returns a new command with
/// the alias expanded.
pub fn expand_git_alias(cmd: &Command) -> Command {
    if !cmd.output.contains("trace: alias expansion:") {
        return cmd.clone();
    }

    let re = Regex::new(r"trace: alias expansion: ([^ ]*) => ([^\n]*)").unwrap();
    if let Some(captures) = re.captures(&cmd.output) {
        let alias = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        let expansion_raw = captures.get(2).map(|m| m.as_str()).unwrap_or("");

        // Parse the expansion (git quotes arguments like 'commit' '--amend')
        let expansion = parse_git_quoted_expansion(expansion_raw);

        // Replace the alias in the script with the expansion
        let pattern = format!(r"\b{}\b", regex::escape(alias));
        if let Ok(alias_re) = Regex::new(&pattern) {
            let new_script = alias_re.replace(&cmd.script, expansion.as_str()).to_string();
            return cmd.with_script(new_script);
        }
    }

    cmd.clone()
}

/// Parse git's quoted expansion format (e.g., "'commit' '--amend'")
fn parse_git_quoted_expansion(s: &str) -> String {
    // Remove quotes and join with spaces
    s.split_whitespace()
        .map(|part| part.trim_matches('\''))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Wrapper that adds git support to a rule.
///
/// This wrapper:
/// 1. Checks if the command is a git command
/// 2. Expands git aliases if present
/// 3. Delegates to the inner rule
pub struct GitSupport<R>(pub R);

impl<R: Rule> Rule for GitSupport<R> {
    fn name(&self) -> &str {
        self.0.name()
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_git_command(cmd) {
            return false;
        }
        let expanded = expand_git_alias(cmd);
        self.0.is_match(&expanded)
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let expanded = expand_git_alias(cmd);
        self.0.get_new_command(&expanded)
    }

    fn priority(&self) -> i32 {
        self.0.priority()
    }

    fn enabled_by_default(&self) -> bool {
        self.0.enabled_by_default()
    }

    fn requires_output(&self) -> bool {
        self.0.requires_output()
    }
}

/// Replace an argument in a command script.
pub fn replace_argument(script: &str, from: &str, to: &str) -> String {
    // First try to replace at the end of the command
    let end_pattern = format!(r" {}$", regex::escape(from));
    if let Ok(re) = Regex::new(&end_pattern) {
        let replaced = re.replace(script, format!(" {}", to));
        if replaced != script {
            return replaced.to_string();
        }
    }

    // Otherwise replace the first occurrence in the middle
    script.replacen(&format!(" {} ", from), &format!(" {} ", to), 1)
}

/// Replace a command with a similar one from matched suggestions.
pub fn replace_command(script: &str, broken: &str, matched: &[String]) -> Vec<String> {
    let close_matches = get_close_matches(broken, matched, 3, 0.1);
    close_matches
        .into_iter()
        .map(|new_cmd| replace_argument(script, broken, new_cmd.trim()))
        .collect()
}

/// Get close matches using string similarity.
pub fn get_close_matches(word: &str, possibilities: &[String], n: usize, cutoff: f64) -> Vec<String> {
    let mut scored: Vec<(f64, &String)> = possibilities
        .iter()
        .map(|p| (strsim::jaro_winkler(word, p), p))
        .filter(|(score, _)| *score >= cutoff)
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(n).map(|(_, s)| s.clone()).collect()
}

/// Get the closest match from a list of possibilities.
pub fn get_closest(word: &str, possibilities: &[String], fallback_to_first: bool) -> Option<String> {
    let matches = get_close_matches(word, possibilities, 1, 0.6);
    if let Some(m) = matches.into_iter().next() {
        return Some(m);
    }
    if fallback_to_first && !possibilities.is_empty() {
        return Some(possibilities[0].clone());
    }
    None
}

/// Extract all matched commands from stderr output.
pub fn get_all_matched_commands(stderr: &str, separators: &[&str]) -> Vec<String> {
    let mut result = Vec::new();
    let mut should_yield = false;

    for line in stderr.lines() {
        let found_separator = separators.iter().any(|sep| line.contains(sep));
        if found_separator {
            should_yield = true;
        } else if should_yield && !line.trim().is_empty() {
            result.push(line.trim().to_string());
        }
    }

    result
}

/// Get list of git branches (local and remote).
pub fn get_branches() -> Vec<String> {
    let output = ProcessCommand::new("git")
        .args(["branch", "-a", "--no-color", "--no-column"])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    // Skip remote HEAD references
                    if line.contains("->") {
                        return None;
                    }

                    let line = line.trim();
                    let line = if line.starts_with('*') {
                        line[1..].trim()
                    } else {
                        line
                    };

                    // Strip 'remotes/origin/' prefix for remote branches
                    let line = if line.starts_with("remotes/") {
                        line.split('/').skip(2).collect::<Vec<_>>().join("/")
                    } else {
                        line.to_string()
                    };

                    if line.is_empty() {
                        None
                    } else {
                        Some(line)
                    }
                })
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

/// Get the current git branch name.
pub fn get_current_branch() -> Option<String> {
    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout);
        Some(branch.trim().to_string())
    } else {
        None
    }
}

/// Create a command that runs two commands in sequence (cmd1 && cmd2).
pub fn and_commands(cmd1: &str, cmd2: &str) -> String {
    format!("{} && {}", cmd1, cmd2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_command() {
        let cmd = Command::new("git status", "");
        assert!(is_git_command(&cmd));

        let cmd = Command::new("hub status", "");
        assert!(is_git_command(&cmd));

        let cmd = Command::new("ls -la", "");
        assert!(!is_git_command(&cmd));
    }

    #[test]
    fn test_is_app() {
        let cmd = Command::new("git status", "");
        assert!(is_app(&cmd, &["git"]));
        assert!(is_app(&cmd, &["git", "hub"]));
        assert!(!is_app(&cmd, &["hub"]));
    }

    #[test]
    fn test_expand_git_alias() {
        let cmd = Command::new(
            "git ci -m 'test'",
            "trace: alias expansion: ci => 'commit'\n",
        );
        let expanded = expand_git_alias(&cmd);
        assert_eq!(expanded.script, "git commit -m 'test'");
    }

    #[test]
    fn test_expand_git_alias_no_alias() {
        let cmd = Command::new("git commit -m 'test'", "");
        let expanded = expand_git_alias(&cmd);
        assert_eq!(expanded.script, "git commit -m 'test'");
    }

    #[test]
    fn test_replace_argument() {
        assert_eq!(
            replace_argument("git branch -d feature", "-d", "-D"),
            "git branch -D feature"
        );
        assert_eq!(
            replace_argument("git push origin main", "push", "pull"),
            "git push origin main".replacen(" push ", " pull ", 1)
        );
    }

    #[test]
    fn test_replace_argument_at_end() {
        assert_eq!(
            replace_argument("git checkout feature", "feature", "main"),
            "git checkout main"
        );
    }

    #[test]
    fn test_get_close_matches() {
        let possibilities = vec![
            "commit".to_string(),
            "checkout".to_string(),
            "cherry-pick".to_string(),
        ];
        let matches = get_close_matches("comit", &possibilities, 3, 0.6);
        assert!(!matches.is_empty());
        assert_eq!(matches[0], "commit");
    }

    #[test]
    fn test_get_all_matched_commands() {
        let output = "git: 'stats' is not a git command. See 'git --help'.\n\nThe most similar command is\n\tstatus\n\tstash";
        let matched = get_all_matched_commands(output, &["The most similar command"]);
        assert!(matched.contains(&"status".to_string()));
    }

    #[test]
    fn test_and_commands() {
        assert_eq!(
            and_commands("git stash", "git checkout main"),
            "git stash && git checkout main"
        );
    }

    #[test]
    fn test_command_script_parts() {
        let cmd = Command::new("git commit -m 'hello world'", "");
        let parts = cmd.script_parts();
        // Note: This depends on shlex behavior, which may vary
        assert!(parts.len() >= 3);
        assert_eq!(parts[0], "git");
    }

    struct TestRule;
    impl Rule for TestRule {
        fn name(&self) -> &str {
            "test_rule"
        }
        fn is_match(&self, cmd: &Command) -> bool {
            cmd.script.contains("test")
        }
        fn get_new_command(&self, cmd: &Command) -> Vec<String> {
            vec![cmd.script.replace("test", "fixed")]
        }
    }

    #[test]
    fn test_git_support_wrapper() {
        let rule = GitSupport(TestRule);

        // Should not match non-git commands
        let cmd = Command::new("test command", "");
        assert!(!rule.is_match(&cmd));

        // Should match git commands
        let cmd = Command::new("git test command", "");
        assert!(rule.is_match(&cmd));
    }
}
