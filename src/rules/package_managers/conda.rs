//! Conda package manager rules.
//!
//! Contains rules for:
//! - `conda_mistype` - Fix mistyped conda commands

use crate::core::{is_app, Command, Rule};
use crate::utils::replace_argument;
use regex::Regex;

/// Rule to fix mistyped conda commands.
///
/// Matches errors like:
/// - `Did you mean 'conda activate'?`
///
/// Uses the suggestion from conda's error message.
///
/// # Example
///
/// ```text
/// $ conda actiavte myenv
///
/// CommandNotFoundError: No command 'conda actiavte'.
/// Did you mean 'conda activate'?
///
/// $ fuck
/// conda activate myenv
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CondaMistype;

impl CondaMistype {
    /// Extract the broken and correct command from conda's error output.
    ///
    /// Returns (broken_cmd, correct_cmd) if found.
    fn get_commands(output: &str) -> Option<(String, String)> {
        // Match pattern like: 'conda broken' ... 'conda correct'
        let re = Regex::new(r"'conda ([^']*)'").ok()?;

        let matches: Vec<_> = re.captures_iter(output)
            .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if matches.len() >= 2 {
            Some((matches[0].clone(), matches[1].clone()))
        } else {
            None
        }
    }
}

impl Rule for CondaMistype {
    fn name(&self) -> &str {
        "conda_mistype"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["conda"]) {
            return false;
        }

        command.output.contains("Did you mean 'conda")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let (broken_cmd, correct_cmd) = match Self::get_commands(&command.output) {
            Some(cmds) => cmds,
            None => return vec![],
        };

        vec![replace_argument(&command.script, &broken_cmd, &correct_cmd)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(CondaMistype.name(), "conda_mistype");
    }

    #[test]
    fn test_matches_did_you_mean() {
        let cmd = Command::new(
            "conda actiavte myenv",
            "CommandNotFoundError: No command 'conda actiavte'.\nDid you mean 'conda activate'?",
        );
        assert!(CondaMistype.is_match(&cmd));
    }

    #[test]
    fn test_no_match_successful() {
        let cmd = Command::new("conda activate myenv", "");
        assert!(!CondaMistype.is_match(&cmd));
    }

    #[test]
    fn test_no_match_other_command() {
        let cmd = Command::new(
            "pip actiavte myenv",
            "Did you mean 'conda activate'?",
        );
        assert!(!CondaMistype.is_match(&cmd));
    }

    #[test]
    fn test_get_commands() {
        let output = "CommandNotFoundError: No command 'conda actiavte'.\nDid you mean 'conda activate'?";
        let cmds = CondaMistype::get_commands(output);
        assert_eq!(cmds, Some(("actiavte".to_string(), "activate".to_string())));
    }

    #[test]
    fn test_get_commands_install() {
        let output = "CommandNotFoundError: No command 'conda instal'.\nDid you mean 'conda install'?";
        let cmds = CondaMistype::get_commands(output);
        assert_eq!(cmds, Some(("instal".to_string(), "install".to_string())));
    }

    #[test]
    fn test_get_new_command() {
        let cmd = Command::new(
            "conda actiavte myenv",
            "CommandNotFoundError: No command 'conda actiavte'.\nDid you mean 'conda activate'?",
        );
        let fixes = CondaMistype.get_new_command(&cmd);
        assert_eq!(fixes, vec!["conda activate myenv"]);
    }

    #[test]
    fn test_get_new_command_install() {
        let cmd = Command::new(
            "conda instal numpy",
            "CommandNotFoundError: No command 'conda instal'.\nDid you mean 'conda install'?",
        );
        let fixes = CondaMistype.get_new_command(&cmd);
        assert_eq!(fixes, vec!["conda install numpy"]);
    }

    #[test]
    fn test_get_new_command_deactivate() {
        let cmd = Command::new(
            "conda deactiavte",
            "CommandNotFoundError: No command 'conda deactiavte'.\nDid you mean 'conda deactivate'?",
        );
        let fixes = CondaMistype.get_new_command(&cmd);
        assert_eq!(fixes, vec!["conda deactivate"]);
    }
}
