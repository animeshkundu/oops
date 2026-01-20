//! Git unknown command rule.
//!
//! This module contains rules for fixing unknown git commands.

use regex::Regex;

use super::support::{get_all_matched_commands, replace_command, Command, GitSupport, Rule};

/// Rule for handling unknown git commands.
///
/// Matches when git reports "is not a git command" and suggests alternatives.
pub struct GitNotCommand;

impl GitNotCommand {
    /// Creates a new GitNotCommand rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitNotCommand)
    }
}

impl Default for GitNotCommand {
    fn default() -> Self {
        GitNotCommand
    }
}

impl Rule for GitNotCommand {
    fn name(&self) -> &str {
        "git_not_command"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output
            .contains(" is not a git command. See 'git --help'.")
            && (cmd.output.contains("The most similar command")
                || cmd.output.contains("Did you mean"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the broken command from the error message
        let re = Regex::new(r"git: '([^']*)' is not a git command").unwrap();
        let broken_cmd = match re.captures(&cmd.output) {
            Some(captures) => captures.get(1).map(|m| m.as_str()).unwrap_or(""),
            None => return vec![],
        };

        if broken_cmd.is_empty() {
            return vec![];
        }

        // Get suggested commands from git's output
        let matched =
            get_all_matched_commands(&cmd.output, &["The most similar command", "Did you mean"]);

        // Replace the broken command with similar ones
        replace_command(&cmd.script, broken_cmd, &matched)
    }
}

/// Rule for handling git commands with typos (simple misspellings).
///
/// This is a fallback when git doesn't provide suggestions.
pub struct GitCommandTypo;

impl GitCommandTypo {
    /// Creates a new GitCommandTypo rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitCommandTypo)
    }

    /// List of common git subcommands
    fn common_commands() -> Vec<String> {
        vec![
            "add",
            "bisect",
            "branch",
            "checkout",
            "cherry-pick",
            "clone",
            "commit",
            "config",
            "diff",
            "fetch",
            "grep",
            "init",
            "log",
            "merge",
            "mv",
            "pull",
            "push",
            "rebase",
            "remote",
            "reset",
            "restore",
            "revert",
            "rm",
            "show",
            "stash",
            "status",
            "switch",
            "tag",
            "worktree",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

impl Default for GitCommandTypo {
    fn default() -> Self {
        GitCommandTypo
    }
}

impl Rule for GitCommandTypo {
    fn name(&self) -> &str {
        "git_command_typo"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Only match if there's an error and git didn't provide suggestions
        cmd.output.contains(" is not a git command")
            && !cmd.output.contains("The most similar command")
            && !cmd.output.contains("Did you mean")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the broken command
        let re = Regex::new(r"git: '([^']*)' is not a git command").unwrap();
        let broken_cmd = match re.captures(&cmd.output) {
            Some(captures) => captures.get(1).map(|m| m.as_str()).unwrap_or(""),
            None => return vec![],
        };

        if broken_cmd.is_empty() {
            return vec![];
        }

        // Use our own list of common commands
        replace_command(&cmd.script, broken_cmd, &Self::common_commands())
    }

    fn priority(&self) -> i32 {
        // Lower priority than git_not_command which uses git's own suggestions
        1100
    }
}

/// Rule for handling git two dashes typo.
///
/// Matches when user types "-option" instead of "--option".
pub struct GitTwoDashes;

impl GitTwoDashes {
    /// Creates a new GitTwoDashes rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitTwoDashes)
    }
}

impl Default for GitTwoDashes {
    fn default() -> Self {
        GitTwoDashes
    }
}

impl Rule for GitTwoDashes {
    fn name(&self) -> &str {
        "git_two_dashes"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Look for single-dash long options that should be double-dash
        let re = Regex::new(r" -([a-z]{2,})").unwrap();
        re.is_match(&cmd.script)
            && (cmd.output.contains("error: unknown switch")
                || cmd.output.contains("error: did you mean"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace single-dash long options with double-dash
        let re = Regex::new(r" -([a-z]{2,})").unwrap();
        let fixed = re.replace_all(&cmd.script, " --$1");
        vec![fixed.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_not_command_matches() {
        let rule = GitNotCommand;
        let cmd = Command::new(
            "git stats",
            "git: 'stats' is not a git command. See 'git --help'.\n\n\
             The most similar command is\n\
             \tstatus\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_not_command_matches_did_you_mean() {
        let rule = GitNotCommand;
        let cmd = Command::new(
            "git pul",
            "git: 'pul' is not a git command. See 'git --help'.\n\n\
             Did you mean this?\n\
             \tpull\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_not_command_no_match_no_suggestions() {
        let rule = GitNotCommand;
        let cmd = Command::new(
            "git xyz",
            "git: 'xyz' is not a git command. See 'git --help'.\n",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_not_command_no_match_success() {
        let rule = GitNotCommand;
        let cmd = Command::new("git status", "On branch main\nnothing to commit\n");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_not_command_get_new_command() {
        let rule = GitNotCommand;
        let cmd = Command::new(
            "git stats",
            "git: 'stats' is not a git command. See 'git --help'.\n\n\
             The most similar command is\n\
             \tstatus\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands.iter().any(|c| c.contains("status")));
    }

    #[test]
    fn test_git_not_command_get_new_command_multiple() {
        let rule = GitNotCommand;
        let cmd = Command::new(
            "git chec",
            "git: 'chec' is not a git command. See 'git --help'.\n\n\
             The most similar commands are\n\
             \tcheckout\n\
             \tcherry-pick\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
    }

    #[test]
    fn test_git_command_typo_matches() {
        let rule = GitCommandTypo;
        let cmd = Command::new(
            "git comit",
            "git: 'comit' is not a git command. See 'git --help'.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_command_typo_no_match_has_suggestions() {
        let rule = GitCommandTypo;
        let cmd = Command::new(
            "git stats",
            "git: 'stats' is not a git command. See 'git --help'.\n\n\
             The most similar command is\n\
             \tstatus\n",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_command_typo_get_new_command() {
        let rule = GitCommandTypo;
        let cmd = Command::new(
            "git comit -m 'test'",
            "git: 'comit' is not a git command. See 'git --help'.\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        // Should suggest 'commit'
        assert!(new_commands.iter().any(|c| c.contains("commit")));
    }

    #[test]
    fn test_git_two_dashes_matches() {
        let rule = GitTwoDashes;
        let cmd = Command::new("git log -oneline", "error: unknown switch `o'\n");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_two_dashes_no_match() {
        let rule = GitTwoDashes;
        let cmd = Command::new("git log --oneline", "abc1234 Initial commit\n");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_two_dashes_get_new_command() {
        let rule = GitTwoDashes;
        let cmd = Command::new("git log -oneline", "error: unknown switch `o'\n");
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git log --oneline"]);
    }

    #[test]
    fn test_git_two_dashes_multiple_options() {
        let rule = GitTwoDashes;
        let cmd = Command::new("git log -oneline -graph", "error: unknown switch `o'\n");
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git log --oneline --graph"]);
    }

    #[test]
    fn test_git_not_command_wrapped() {
        let rule = GitNotCommand::new();

        // Should not match non-git commands
        let cmd = Command::new(
            "svn stats",
            "git: 'stats' is not a git command. See 'git --help'.\n\n\
             The most similar command is\n\
             \tstatus\n",
        );
        assert!(!rule.is_match(&cmd));

        // Should match git commands
        let cmd = Command::new(
            "git stats",
            "git: 'stats' is not a git command. See 'git --help'.\n\n\
             The most similar command is\n\
             \tstatus\n",
        );
        assert!(rule.is_match(&cmd));
    }
}
