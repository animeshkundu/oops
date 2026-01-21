//! NPM package manager rules (Node.js).
//!
//! Contains rules for:
//! - `npm_missing_script` - Suggest correct script names when "missing script" error
//! - `npm_wrong_command` - Suggest similar npm commands when command not recognized

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;

/// Rule to suggest correct npm script names when "missing script" error occurs.
///
/// Matches errors like:
/// - `npm ERR! missing script: tset` (when user meant `test`)
///
/// Suggests the closest matching script from package.json.
#[derive(Debug, Clone, Copy, Default)]
pub struct NpmMissingScript;

impl NpmMissingScript {
    /// Extract the misspelled script name from npm error output.
    fn get_misspelled_script(output: &str) -> Option<String> {
        let re = Regex::new(r"npm ERR! missing script: (.+)").ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().trim().to_string())
    }

    /// Extract available scripts from npm error output.
    ///
    /// npm usually lists available scripts when a script is missing.
    fn get_available_scripts(output: &str) -> Vec<String> {
        // npm shows available scripts in the output like:
        // "Did you mean one of these?"
        // "  test"
        // "  start"
        // "  build"
        // Or sometimes: "Available scripts: test, start, build"

        let mut scripts = Vec::new();

        // Try parsing "Did you mean one of these?" format
        let mut in_scripts_section = false;
        for line in output.lines() {
            if line.contains("Did you mean") || line.contains("available scripts:") {
                in_scripts_section = true;
                continue;
            }
            if in_scripts_section {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("npm") {
                    break;
                }
                // Script names might have "- " prefix
                let script = trimmed.trim_start_matches("- ").trim();
                if !script.is_empty() && !script.contains(' ') {
                    scripts.push(script.to_string());
                }
            }
        }

        // Common npm scripts as fallback
        if scripts.is_empty() {
            scripts = vec![
                "test".to_string(),
                "start".to_string(),
                "build".to_string(),
                "dev".to_string(),
                "lint".to_string(),
                "serve".to_string(),
                "watch".to_string(),
                "clean".to_string(),
                "install".to_string(),
                "prepare".to_string(),
            ];
        }

        scripts
    }
}

impl Rule for NpmMissingScript {
    fn name(&self) -> &str {
        "npm_missing_script"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["npm"]) {
            return false;
        }

        // Check if command has "run" or similar script-running part
        let parts = command.script_parts();
        let has_run_like = parts
            .iter()
            .any(|p| p.starts_with("ru") || p == "run-script");

        has_run_like && command.output.contains("npm ERR! missing script:")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let misspelled = match Self::get_misspelled_script(&command.output) {
            Some(s) => s,
            None => return vec![],
        };

        let scripts = Self::get_available_scripts(&command.output);
        let matches = get_close_matches(&misspelled, &scripts, 3, 0.6);

        if matches.is_empty() {
            return vec![];
        }

        matches
            .into_iter()
            .map(|script| replace_argument(&command.script, &misspelled, &script))
            .collect()
    }
}

/// Rule to suggest similar npm commands when command is not recognized.
///
/// Matches errors like:
/// - `'instal' is not a npm command`
/// - `where <command> is one of: ...`
///
/// Suggests the closest matching npm command.
#[derive(Debug, Clone, Copy, Default)]
pub struct NpmWrongCommand;

impl NpmWrongCommand {
    /// Extract the wrong command from script parts.
    fn get_wrong_command(parts: &[String]) -> Option<String> {
        // Find the first non-flag argument after "npm"
        parts.iter().skip(1).find(|p| !p.starts_with('-')).cloned()
    }

    /// Extract available commands from npm error output.
    fn get_available_commands(output: &str) -> Vec<String> {
        let mut commands = Vec::new();
        let mut in_commands_section = false;

        for line in output.lines() {
            if line.contains("where <command> is one of:") {
                in_commands_section = true;
                continue;
            }
            if in_commands_section {
                if line.trim().is_empty() {
                    break;
                }
                // Commands are usually comma-separated on a line
                for cmd in line.split(',') {
                    let cmd = cmd.trim();
                    if !cmd.is_empty() {
                        commands.push(cmd.to_string());
                    }
                }
            }
        }

        // Fallback to common npm commands if parsing failed
        if commands.is_empty() {
            commands = vec![
                "install".to_string(),
                "uninstall".to_string(),
                "update".to_string(),
                "publish".to_string(),
                "run".to_string(),
                "start".to_string(),
                "test".to_string(),
                "init".to_string(),
                "search".to_string(),
                "list".to_string(),
                "outdated".to_string(),
                "audit".to_string(),
                "cache".to_string(),
                "config".to_string(),
                "help".to_string(),
                "version".to_string(),
                "view".to_string(),
                "pack".to_string(),
                "link".to_string(),
                "prune".to_string(),
                "rebuild".to_string(),
                "dedupe".to_string(),
            ];
        }

        commands
    }
}

impl Rule for NpmWrongCommand {
    fn name(&self) -> &str {
        "npm_wrong_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["npm"]) {
            return false;
        }

        // Check for "is not a npm command" or the list of available commands
        (command.output.contains("is not a npm command")
            || command.output.contains("where <command> is one of:"))
            && Self::get_wrong_command(command.script_parts()).is_some()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let wrong_cmd = match Self::get_wrong_command(command.script_parts()) {
            Some(c) => c,
            None => return vec![],
        };

        let available = Self::get_available_commands(&command.output);
        let matches = get_close_matches(&wrong_cmd, &available, 1, 0.6);

        if let Some(fixed) = matches.into_iter().next() {
            return vec![replace_argument(&command.script, &wrong_cmd, &fixed)];
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod npm_missing_script_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(NpmMissingScript.name(), "npm_missing_script");
        }

        #[test]
        fn test_matches_missing_script() {
            let cmd = Command::new(
                "npm run tset",
                "npm ERR! missing script: tset\n\nnpm ERR! Did you mean one of these?\n  test",
            );
            assert!(NpmMissingScript.is_match(&cmd));
        }

        #[test]
        fn test_matches_run_script() {
            let cmd = Command::new("npm run-script tset", "npm ERR! missing script: tset");
            assert!(NpmMissingScript.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_error() {
            let cmd = Command::new("npm run test", "npm ERR! Test failed");
            assert!(!NpmMissingScript.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("yarn run tset", "npm ERR! missing script: tset");
            assert!(!NpmMissingScript.is_match(&cmd));
        }

        #[test]
        fn test_get_misspelled_script() {
            let output = "npm ERR! missing script: tset\nnpm ERR! Did you mean";
            let script = NpmMissingScript::get_misspelled_script(output);
            assert_eq!(script, Some("tset".to_string()));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "npm run tset",
                "npm ERR! missing script: tset\n\nnpm ERR! Did you mean one of these?\n  test",
            );
            let fixes = NpmMissingScript.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            // "test" should be a suggestion for "tset"
            assert!(fixes.iter().any(|f| f.contains("test")));
        }

        #[test]
        fn test_get_new_command_with_fallback_scripts() {
            let cmd = Command::new("npm run tset", "npm ERR! missing script: tset");
            let fixes = NpmMissingScript.get_new_command(&cmd);
            // Should use fallback scripts and find "test"
            assert!(fixes.iter().any(|f| f.contains("test")));
        }
    }

    mod npm_wrong_command_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(NpmWrongCommand.name(), "npm_wrong_command");
        }

        #[test]
        fn test_matches_not_a_command() {
            let cmd = Command::new(
                "npm instal lodash",
                "'instal' is not a npm command. See 'npm help'.\n\nwhere <command> is one of:\n    install, uninstall, update",
            );
            assert!(NpmWrongCommand.is_match(&cmd));
        }

        #[test]
        fn test_matches_command_list() {
            let cmd = Command::new(
                "npm pubish",
                "Unknown command: pubish\n\nwhere <command> is one of:\n    publish, install",
            );
            assert!(NpmWrongCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_valid_command() {
            let cmd = Command::new("npm install lodash", "added 1 package");
            assert!(!NpmWrongCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_tool() {
            let cmd = Command::new("yarn instal", "'instal' is not a npm command");
            assert!(!NpmWrongCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_wrong_command() {
            let parts = vec![
                "npm".to_string(),
                "instal".to_string(),
                "lodash".to_string(),
            ];
            let wrong = NpmWrongCommand::get_wrong_command(&parts);
            assert_eq!(wrong, Some("instal".to_string()));
        }

        #[test]
        fn test_get_wrong_command_with_flag() {
            let parts = vec![
                "npm".to_string(),
                "-g".to_string(),
                "instal".to_string(),
                "lodash".to_string(),
            ];
            let wrong = NpmWrongCommand::get_wrong_command(&parts);
            assert_eq!(wrong, Some("instal".to_string()));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "npm instal lodash",
                "'instal' is not a npm command.\n\nwhere <command> is one of:\n    install, uninstall, update",
            );
            let fixes = NpmWrongCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["npm install lodash"]);
        }

        #[test]
        fn test_get_new_command_with_fallback() {
            let cmd = Command::new("npm pubish package", "'pubish' is not a npm command.");
            let fixes = NpmWrongCommand.get_new_command(&cmd);
            assert_eq!(fixes, vec!["npm publish package"]);
        }
    }
}
