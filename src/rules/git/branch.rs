//! Git branch related rules.
//!
//! This module contains rules for fixing common git branch issues.

use regex::Regex;

use super::support::{
    and_commands, get_branches, get_closest, replace_argument, Command, GitSupport, Rule,
};

/// Rule for handling branch deletion when not fully merged.
///
/// Matches when git refuses to delete a branch with -d because it's not fully merged.
pub struct GitBranchDelete;

impl GitBranchDelete {
    /// Creates a new GitBranchDelete rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitBranchDelete)
    }
}

impl Default for GitBranchDelete {
    fn default() -> Self {
        GitBranchDelete
    }
}

impl Rule for GitBranchDelete {
    fn name(&self) -> &str {
        "git_branch_delete"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("branch -d")
            && cmd.output.contains("If you are sure you want to delete it")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "-d", "-D")]
    }
}

/// Rule for handling branch deletion when branch is checked out.
///
/// Matches when git refuses to delete a branch because it's currently checked out.
pub struct GitBranchDeleteCheckedOut;

impl GitBranchDeleteCheckedOut {
    /// Creates a new GitBranchDeleteCheckedOut rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitBranchDeleteCheckedOut)
    }
}

impl Default for GitBranchDeleteCheckedOut {
    fn default() -> Self {
        GitBranchDeleteCheckedOut
    }
}

impl Rule for GitBranchDeleteCheckedOut {
    fn name(&self) -> &str {
        "git_branch_delete_checked_out"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        (cmd.script.contains("branch -d") || cmd.script.contains("branch -D"))
            && cmd.output.contains("error: Cannot delete branch '")
            && cmd.output.contains("' checked out at '")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // First checkout master/main, then delete the branch
        let delete_cmd = replace_argument(&cmd.script, "-d", "-D");
        vec![and_commands("git checkout master", &delete_cmd)]
    }
}

/// Rule for handling branch that already exists.
///
/// Matches when trying to create a branch that already exists.
pub struct GitBranchExists;

impl GitBranchExists {
    /// Creates a new GitBranchExists rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitBranchExists)
    }
}

impl Default for GitBranchExists {
    fn default() -> Self {
        GitBranchExists
    }
}

impl Rule for GitBranchExists {
    fn name(&self) -> &str {
        "git_branch_exists"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output.contains("already exists")
            && (cmd.script.contains("checkout -b") || cmd.script.contains("branch "))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the branch name
        let re = Regex::new(r"fatal: [Aa] branch named '([^']*)' already exists").unwrap();
        let branch = match re.captures(&cmd.output) {
            Some(captures) => captures.get(1).map(|m| m.as_str()).unwrap_or(""),
            None => return vec![],
        };

        if branch.is_empty() {
            return vec![];
        }

        let mut new_commands = Vec::new();

        // If using checkout -b, suggest just checkout
        if cmd.script.contains("checkout -b") {
            new_commands.push(replace_argument(&cmd.script, "checkout -b", "checkout"));
        }

        // Also suggest resetting the existing branch
        if cmd.script.contains("branch ") {
            new_commands.push(format!("git checkout {}", branch));
        }

        new_commands
    }
}

/// Rule for handling wrong branch name.
///
/// Matches when git can't find a branch and suggests similar ones.
pub struct GitBranchNotFound;

impl GitBranchNotFound {
    /// Creates a new GitBranchNotFound rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitBranchNotFound)
    }
}

impl Default for GitBranchNotFound {
    fn default() -> Self {
        GitBranchNotFound
    }
}

impl Rule for GitBranchNotFound {
    fn name(&self) -> &str {
        "git_branch_not_found"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("branch")
            && (cmd.output.contains("error: branch '") && cmd.output.contains("' not found"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the branch name that wasn't found
        let re = Regex::new(r"error: branch '([^']*)' not found").unwrap();
        let missing = match re.captures(&cmd.output) {
            Some(captures) => captures.get(1).map(|m| m.as_str()).unwrap_or(""),
            None => return vec![],
        };

        if missing.is_empty() {
            return vec![];
        }

        // Find similar branches
        let branches = get_branches();
        if let Some(closest) = get_closest(missing, &branches, false) {
            return vec![replace_argument(&cmd.script, missing, &closest)];
        }

        vec![]
    }
}

/// Rule for listing branches when using wrong flag.
///
/// Matches when trying to list branches with wrong syntax.
pub struct GitBranchList;

impl GitBranchList {
    /// Creates a new GitBranchList rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitBranchList)
    }
}

impl Default for GitBranchList {
    fn default() -> Self {
        GitBranchList
    }
}

impl Rule for GitBranchList {
    fn name(&self) -> &str {
        "git_branch_list"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("branch")
            && (cmd.output.contains("error: unknown switch")
                || cmd.output.contains("error: did you mean"))
            && (cmd.script.contains("-l ") || cmd.script.ends_with("-l"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // The -l flag with argument is for listing branches matching a pattern
        // But if used incorrectly, suggest --list
        vec![replace_argument(&cmd.script, "-l", "--list")]
    }
}

/// Rule for handling branch flag position error.
///
/// Matches when flag is placed after the branch name.
pub struct GitBranchFlagPosition;

impl GitBranchFlagPosition {
    /// Creates a new GitBranchFlagPosition rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitBranchFlagPosition)
    }
}

impl Default for GitBranchFlagPosition {
    fn default() -> Self {
        GitBranchFlagPosition
    }
}

impl Rule for GitBranchFlagPosition {
    fn name(&self) -> &str {
        "git_branch_0flag"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("branch ")
            && cmd.output.contains("fatal: ")
            && cmd.output.contains("is not a valid branch name")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // The issue is usually that a flag was interpreted as a branch name
        // Try to reorder the arguments
        let parts: Vec<&str> = cmd.script.split_whitespace().collect();

        // Find the branch subcommand index
        let branch_idx = parts.iter().position(|&p| p == "branch").unwrap_or(0);

        // Find flags after branch names
        let mut new_parts: Vec<&str> = Vec::new();
        let mut flags: Vec<&str> = Vec::new();
        let mut args: Vec<&str> = Vec::new();

        for (i, &part) in parts.iter().enumerate() {
            if i <= branch_idx {
                new_parts.push(part);
            } else if part.starts_with('-') {
                flags.push(part);
            } else {
                args.push(part);
            }
        }

        // Reorder: git branch [flags] [args]
        new_parts.extend(flags);
        new_parts.extend(args);

        vec![new_parts.join(" ")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_branch_delete_matches() {
        let rule = GitBranchDelete;
        let cmd = Command::new(
            "git branch -d feature",
            "error: The branch 'feature' is not fully merged.\n\
             If you are sure you want to delete it, run 'git branch -D feature'.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_delete_no_match() {
        let rule = GitBranchDelete;
        let cmd = Command::new("git branch -d feature", "Deleted branch feature\n");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_delete_get_new_command() {
        let rule = GitBranchDelete;
        let cmd = Command::new(
            "git branch -d feature",
            "error: The branch 'feature' is not fully merged.\n\
             If you are sure you want to delete it, run 'git branch -D feature'.\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git branch -D feature"]);
    }

    #[test]
    fn test_git_branch_delete_checked_out_matches() {
        let rule = GitBranchDeleteCheckedOut;
        let cmd = Command::new(
            "git branch -d feature",
            "error: Cannot delete branch 'feature' checked out at '/path/to/repo'\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_delete_checked_out_matches_D() {
        let rule = GitBranchDeleteCheckedOut;
        let cmd = Command::new(
            "git branch -D feature",
            "error: Cannot delete branch 'feature' checked out at '/path/to/repo'\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_delete_checked_out_get_new_command() {
        let rule = GitBranchDeleteCheckedOut;
        let cmd = Command::new(
            "git branch -d feature",
            "error: Cannot delete branch 'feature' checked out at '/path/to/repo'\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("git checkout master"));
        assert!(new_commands[0].contains("git branch -D feature"));
    }

    #[test]
    fn test_git_branch_exists_matches() {
        let rule = GitBranchExists;
        let cmd = Command::new(
            "git checkout -b feature",
            "fatal: A branch named 'feature' already exists.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_exists_no_match() {
        let rule = GitBranchExists;
        let cmd = Command::new(
            "git checkout -b newbranch",
            "Switched to a new branch 'newbranch'\n",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_exists_get_new_command() {
        let rule = GitBranchExists;
        let cmd = Command::new(
            "git checkout -b feature",
            "fatal: A branch named 'feature' already exists.\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands.iter().any(|c| c == "git checkout feature"));
    }

    #[test]
    fn test_git_branch_not_found_matches() {
        let rule = GitBranchNotFound;
        let cmd = Command::new(
            "git branch -d featur",
            "error: branch 'featur' not found.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_not_found_no_match() {
        let rule = GitBranchNotFound;
        let cmd = Command::new("git branch -d feature", "Deleted branch feature\n");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_list_matches() {
        let rule = GitBranchList;
        let cmd = Command::new(
            "git branch -l",
            "error: unknown switch `l'\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_list_get_new_command() {
        let rule = GitBranchList;
        let cmd = Command::new(
            "git branch -l feature*",
            "error: unknown switch `l'\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(new_commands.iter().any(|c| c.contains("--list")));
    }

    #[test]
    fn test_git_branch_flag_position_matches() {
        let rule = GitBranchFlagPosition;
        let cmd = Command::new(
            "git branch feature -d",
            "fatal: '-d' is not a valid branch name.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_branch_flag_position_get_new_command() {
        let rule = GitBranchFlagPosition;
        let cmd = Command::new(
            "git branch feature -d",
            "fatal: '-d' is not a valid branch name.\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert_eq!(new_commands[0], "git branch -d feature");
    }

    #[test]
    fn test_git_branch_delete_wrapped() {
        let rule = GitBranchDelete::new();

        // Should not match non-git commands
        let cmd = Command::new(
            "svn branch -d feature",
            "If you are sure you want to delete it",
        );
        assert!(!rule.is_match(&cmd));

        // Should match git commands
        let cmd = Command::new(
            "git branch -d feature",
            "If you are sure you want to delete it",
        );
        assert!(rule.is_match(&cmd));
    }
}
