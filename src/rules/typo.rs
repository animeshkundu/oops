//! Common typo correction rules.
//!
//! This module contains rules for fixing common command typos:
//!
//! - [`SlLs`] - Fixes "sl" to "ls"
//! - [`PythonCommand`] - Fixes python version issues (python vs python3)
//! - [`Systemctl`] - Fixes common systemctl typos

use crate::core::{is_app, Command, Rule};
use crate::utils::get_close_matches;

/// Rule that fixes "sl" typo to "ls".
///
/// This is one of the most common typos - typing "sl" instead of "ls".
/// Note: Some systems have an actual `sl` command (steam locomotive joke),
/// but this rule handles the case where `sl` is not installed.
///
/// # Example
///
/// ```
/// use oops::rules::typo::SlLs;
/// use oops::core::{Command, Rule};
///
/// let rule = SlLs;
/// let cmd = Command::new("sl", "sl: command not found");
/// assert!(rule.is_match(&cmd));
/// assert_eq!(rule.get_new_command(&cmd), vec!["ls"]);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SlLs;

impl Rule for SlLs {
    fn name(&self) -> &str {
        "sl_ls"
    }

    fn priority(&self) -> i32 {
        // High priority - very common typo
        100
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let script = cmd.script.trim();

        // Must start with "sl" (possibly with arguments)
        if !script.starts_with("sl") {
            return false;
        }

        // Check that it's "sl" followed by nothing, space, or tab
        let after_sl = &script[2..];
        if !after_sl.is_empty() && !after_sl.starts_with(' ') && !after_sl.starts_with('\t') {
            return false;
        }

        // Must have an error indicating command not found
        let output_lower = cmd.output.to_lowercase();
        output_lower.contains("command not found")
            || output_lower.contains("not recognized")
            || output_lower.contains("not found")
            || output_lower.contains("unknown command")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let script = cmd.script.trim();

        // Replace "sl" with "ls" at the beginning
        if let Some(rest) = script.strip_prefix("sl ") {
            vec![format!("ls {}", rest)]
        } else if script == "sl" {
            vec!["ls".to_string()]
        } else if let Some(rest) = script.strip_prefix("sl") {
            vec![format!("ls{}", rest)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that fixes python version command issues.
///
/// On many modern systems, Python 2 has been removed and `python` either
/// doesn't exist or is Python 3. This rule helps users transition by
/// suggesting the correct python command.
///
/// Common fixes:
/// - `python` -> `python3` (when python is not available)
/// - `python3` -> `python` (when python3 is not available but python is)
/// - `pip` -> `pip3` (same as above)
///
/// # Example
///
/// ```
/// use oops::rules::typo::PythonCommand;
/// use oops::core::{Command, Rule};
///
/// let rule = PythonCommand;
/// let cmd = Command::new("python script.py", "python: command not found");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PythonCommand;

impl PythonCommand {
    /// Check if a command exists in PATH
    fn command_exists(cmd: &str) -> bool {
        which::which(cmd).is_ok()
    }
}

impl Rule for PythonCommand {
    fn name(&self) -> &str {
        "python_command"
    }

    fn priority(&self) -> i32 {
        150
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return false;
        }

        let first = &parts[0];

        // Check if it's a python-related command
        let is_python_cmd = first == "python"
            || first == "python3"
            || first == "python2"
            || first == "pip"
            || first == "pip3"
            || first == "pip2";

        if !is_python_cmd {
            return false;
        }

        // Check for command not found error
        let output_lower = cmd.output.to_lowercase();
        output_lower.contains("command not found")
            || output_lower.contains("not recognized")
            || output_lower.contains("not found")
            || output_lower.contains("no such file or directory")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return vec![];
        }

        let first = &parts[0];
        let rest: String = if parts.len() > 1 {
            format!(" {}", parts[1..].join(" "))
        } else {
            String::new()
        };

        let mut suggestions = Vec::new();

        match first.as_str() {
            "python" => {
                // Try python3 first, then python2
                if Self::command_exists("python3") {
                    suggestions.push(format!("python3{}", rest));
                }
                if Self::command_exists("python2") {
                    suggestions.push(format!("python2{}", rest));
                }
            }
            "python3" => {
                // Try python
                if Self::command_exists("python") {
                    suggestions.push(format!("python{}", rest));
                }
            }
            "python2" => {
                // Python 2 is deprecated, suggest python3
                if Self::command_exists("python3") {
                    suggestions.push(format!("python3{}", rest));
                }
                if Self::command_exists("python") {
                    suggestions.push(format!("python{}", rest));
                }
            }
            "pip" => {
                if Self::command_exists("pip3") {
                    suggestions.push(format!("pip3{}", rest));
                }
                if Self::command_exists("pip2") {
                    suggestions.push(format!("pip2{}", rest));
                }
            }
            "pip3" => {
                if Self::command_exists("pip") {
                    suggestions.push(format!("pip{}", rest));
                }
            }
            "pip2" => {
                if Self::command_exists("pip3") {
                    suggestions.push(format!("pip3{}", rest));
                }
                if Self::command_exists("pip") {
                    suggestions.push(format!("pip{}", rest));
                }
            }
            _ => {}
        }

        suggestions
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Systemctl commands that can be suggested for typos.
const SYSTEMCTL_COMMANDS: &[&str] = &[
    "start",
    "stop",
    "restart",
    "reload",
    "status",
    "enable",
    "disable",
    "is-active",
    "is-enabled",
    "is-failed",
    "list-units",
    "list-unit-files",
    "list-sockets",
    "list-timers",
    "list-dependencies",
    "daemon-reload",
    "daemon-reexec",
    "show",
    "cat",
    "edit",
    "mask",
    "unmask",
    "link",
    "revert",
    "preset",
    "preset-all",
    "isolate",
    "kill",
    "clean",
    "freeze",
    "thaw",
    "set-property",
    "reset-failed",
    "poweroff",
    "reboot",
    "suspend",
    "hibernate",
    "hybrid-sleep",
    "suspend-then-hibernate",
];

/// Rule that fixes common systemctl typos.
///
/// This rule handles:
/// - Typos in systemctl subcommands (e.g., "systemctl statsu" -> "systemctl status")
/// - Swapped arguments (e.g., "systemctl nginx restart" -> "systemctl restart nginx")
///
/// # Example
///
/// ```
/// use oops::rules::typo::Systemctl;
/// use oops::core::{Command, Rule};
///
/// let rule = Systemctl;
/// let cmd = Command::new("systemctl statsu nginx", "Unknown operation 'statsu'");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Systemctl;

impl Rule for Systemctl {
    fn name(&self) -> &str {
        "systemctl"
    }

    fn priority(&self) -> i32 {
        200
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Must be a systemctl command
        if !is_app(cmd, &["systemctl"]) {
            let script = cmd.script.trim();
            if script != "systemctl" && !script.starts_with("systemctl ") {
                return false;
            }
        }

        // Check for error messages
        let output_lower = cmd.output.to_lowercase();
        output_lower.contains("unknown operation")
            || output_lower.contains("unknown command")
            || output_lower.contains("invalid")
            || output_lower.contains("not a valid")
            || output_lower.contains("unrecognized option")
            || output_lower.contains("failed to")
            || output_lower.contains("too few arguments")
            || output_lower.contains("requires at least")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();

        if parts.len() < 2 {
            return vec![];
        }

        // Skip the "systemctl" part
        let subcommand = &parts[1];

        // Check if arguments might be swapped FIRST (e.g., "systemctl nginx restart")
        // This takes priority over typo correction
        if parts.len() >= 3 {
            let potential_cmd = &parts[2];
            if SYSTEMCTL_COMMANDS.contains(&potential_cmd.as_str()) {
                // Likely swapped: "systemctl <service> <command>" -> "systemctl <command> <service>"
                let mut fixed_parts = vec!["systemctl".to_string()];
                fixed_parts.push(parts[2].clone()); // The command
                fixed_parts.push(parts[1].clone()); // The service
                fixed_parts.extend_from_slice(&parts[3..]);
                return vec![fixed_parts.join(" ")];
            }
        }

        // Check if it's a typo in the subcommand
        let commands: Vec<String> = SYSTEMCTL_COMMANDS.iter().map(|s| s.to_string()).collect();
        let matches = get_close_matches(subcommand, &commands, 3, 0.6);

        if !matches.is_empty() {
            // Found close matches - suggest corrections
            return matches
                .into_iter()
                .map(|correct_cmd| {
                    let mut new_parts = vec!["systemctl".to_string(), correct_cmd];
                    new_parts.extend_from_slice(&parts[2..]);
                    new_parts.join(" ")
                })
                .collect();
        }

        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SlLs tests
    mod sl_ls {
        use super::*;

        #[test]
        fn test_name() {
            let rule = SlLs;
            assert_eq!(rule.name(), "sl_ls");
        }

        #[test]
        fn test_matches_sl_not_found() {
            let rule = SlLs;
            let cmd = Command::new("sl", "sl: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_sl_with_args() {
            let rule = SlLs;
            let cmd = Command::new("sl -la", "sl: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_not_recognized() {
            let rule = SlLs;
            let cmd = Command::new(
                "sl",
                "'sl' is not recognized as an internal or external command",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_ls() {
            let rule = SlLs;
            let cmd = Command::new("ls", "file1 file2");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_sleep() {
            let rule = SlLs;
            // "sleep" starts with "sl" but is a different command
            let cmd = Command::new("sleep 5", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_sl_successful() {
            let rule = SlLs;
            // If sl command succeeds (steam locomotive), don't match
            let cmd = Command::new("sl", "     ====        ________                ___________");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_simple() {
            let rule = SlLs;
            let cmd = Command::new("sl", "command not found");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ls"]);
        }

        #[test]
        fn test_get_new_command_with_args() {
            let rule = SlLs;
            let cmd = Command::new("sl -la", "command not found");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ls -la"]);
        }

        #[test]
        fn test_get_new_command_with_path() {
            let rule = SlLs;
            let cmd = Command::new("sl /home", "command not found");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ls /home"]);
        }
    }

    // PythonCommand tests
    mod python_command {
        use super::*;

        #[test]
        fn test_name() {
            let rule = PythonCommand;
            assert_eq!(rule.name(), "python_command");
        }

        #[test]
        fn test_matches_python_not_found() {
            let rule = PythonCommand;
            let cmd = Command::new("python script.py", "python: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_python3_not_found() {
            let rule = PythonCommand;
            let cmd = Command::new("python3 script.py", "python3: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_pip_not_found() {
            let rule = PythonCommand;
            let cmd = Command::new("pip install package", "pip: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_python_success() {
            let rule = PythonCommand;
            let cmd = Command::new("python script.py", "Hello, World!");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = PythonCommand;
            let cmd = Command::new("ruby script.rb", "ruby: command not found");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_requires_output() {
            let rule = PythonCommand;
            assert!(rule.requires_output());
        }

        // Note: get_new_command tests depend on what's installed on the system
        // so we just verify it doesn't panic
        #[test]
        fn test_get_new_command_doesnt_panic() {
            let rule = PythonCommand;
            let cmd = Command::new("python script.py", "command not found");
            let _fixes = rule.get_new_command(&cmd);
            // Just verify it completes without panic
        }
    }

    // Systemctl tests
    mod systemctl {
        use super::*;

        #[test]
        fn test_name() {
            let rule = Systemctl;
            assert_eq!(rule.name(), "systemctl");
        }

        #[test]
        fn test_matches_unknown_operation() {
            let rule = Systemctl;
            let cmd = Command::new("systemctl statsu nginx", "Unknown operation 'statsu'");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_unknown_command() {
            let rule = Systemctl;
            let cmd = Command::new("systemctl stopp nginx", "Unknown command verb 'stopp'");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_failed() {
            let rule = Systemctl;
            let cmd = Command::new(
                "systemctl restar nginx",
                "Failed to restart nginx.service: Unknown unit",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_too_few_arguments() {
            let rule = Systemctl;
            let cmd = Command::new("systemctl start", "Too few arguments");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let rule = Systemctl;
            let cmd = Command::new(
                "systemctl status nginx",
                "nginx.service - A high performance web server",
            );
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let rule = Systemctl;
            let cmd = Command::new("service nginx status", "nginx is running");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_typo() {
            let rule = Systemctl;
            let cmd = Command::new("systemctl statsu nginx", "Unknown operation 'statsu'");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("status"));
            assert!(fixes[0].contains("nginx"));
        }

        #[test]
        fn test_get_new_command_restart_typo() {
            let rule = Systemctl;
            let cmd = Command::new("systemctl restar nginx", "Unknown operation 'restar'");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("restart"));
        }

        #[test]
        fn test_get_new_command_swapped_args() {
            let rule = Systemctl;
            let cmd = Command::new("systemctl nginx restart", "Unknown operation 'nginx'");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert_eq!(fixes[0], "systemctl restart nginx");
        }

        #[test]
        fn test_get_new_command_swapped_args_stop() {
            let rule = Systemctl;
            let cmd = Command::new(
                "systemctl nginx.service stop",
                "Unknown operation 'nginx.service'",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert_eq!(fixes[0], "systemctl stop nginx.service");
        }

        #[test]
        fn test_requires_output() {
            let rule = Systemctl;
            assert!(rule.requires_output());
        }
    }

    // Integration tests
    mod integration {
        use super::*;

        #[test]
        fn test_sl_ls_priority() {
            let sl_ls = SlLs;
            let python = PythonCommand;
            let systemctl = Systemctl;

            // SlLs should have highest priority (lowest number)
            assert!(sl_ls.priority() < python.priority());
            assert!(python.priority() < systemctl.priority());
        }

        #[test]
        fn test_all_rules_enabled_by_default() {
            assert!(SlLs.enabled_by_default());
            assert!(PythonCommand.enabled_by_default());
            assert!(Systemctl.enabled_by_default());
        }
    }
}
