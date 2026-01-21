//! Git checkout related rules.
//!
//! This module contains rules for fixing common git checkout issues.

use regex::Regex;

use super::support::{
    and_commands, get_branches, get_closest, replace_argument, Command, GitSupport, Rule,
};

/// Rule for handling wrong branch name in checkout.
///
/// Matches when git checkout fails because the branch/file doesn't exist,
/// and suggests similar branch names.
pub struct GitCheckout;

impl GitCheckout {
    /// Creates a new GitCheckout rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitCheckout)
    }
}

impl Default for GitCheckout {
    fn default() -> Self {
        GitCheckout
    }
}

impl Rule for GitCheckout {
    fn name(&self) -> &str {
        "git_checkout"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output
            .contains("did not match any file(s) known to git")
            && !cmd.output.contains("Did you forget to 'git add'?")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the missing file/branch name from the error
        let re = Regex::new(r"error: pathspec '([^']*)' did not match any file\(s\) known to git")
            .unwrap();
        let missing = match re.captures(&cmd.output) {
            Some(captures) => captures.get(1).map(|m| m.as_str()).unwrap_or(""),
            None => return vec![],
        };

        if missing.is_empty() {
            return vec![];
        }

        let mut new_commands = Vec::new();

        // Try to find a similar branch name
        let branches = get_branches();
        let branch_strings: Vec<String> = branches.clone();
        if let Some(closest_branch) = get_closest(missing, &branch_strings, false) {
            new_commands.push(replace_argument(&cmd.script, missing, &closest_branch));
        }

        // If the command is 'checkout', suggest creating a new branch
        let parts: Vec<&str> = cmd.script.split_whitespace().collect();
        if parts.len() > 1 && parts[1] == "checkout" {
            new_commands.push(replace_argument(&cmd.script, "checkout", "checkout -b"));
        }

        // If no suggestions found, offer to create the branch first
        if new_commands.is_empty() {
            let create_and_checkout = and_commands(&format!("git branch {}", missing), &cmd.script);
            new_commands.push(create_and_checkout);
        }

        new_commands
    }
}

/// Rule for handling checkout when there are uncommitted changes.
///
/// Matches when git checkout fails because of uncommitted changes.
pub struct GitCheckoutUncommittedChanges;

impl GitCheckoutUncommittedChanges {
    /// Creates a new GitCheckoutUncommittedChanges rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitCheckoutUncommittedChanges)
    }
}

impl Default for GitCheckoutUncommittedChanges {
    fn default() -> Self {
        GitCheckoutUncommittedChanges
    }
}

impl Rule for GitCheckoutUncommittedChanges {
    fn name(&self) -> &str {
        "git_checkout_uncommitted_changes"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        (cmd.script.contains("checkout") || cmd.script.contains("switch"))
            && (cmd
                .output
                .contains("Please commit your changes or stash them")
                || cmd
                    .output
                    .contains("Your local changes to the following files would be overwritten"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![
            and_commands("git stash", &cmd.script),
            and_commands(&format!("{} --force", cmd.script), ""),
        ]
        .into_iter()
        .filter(|s| !s.ends_with(" && "))
        .map(|s| s.trim_end_matches(" && ").to_string())
        .collect()
    }
}

/// Rule for detecting main/master branch confusion.
///
/// Matches when trying to checkout 'main' but only 'master' exists, or vice versa.
pub struct GitMainMaster;

impl GitMainMaster {
    /// Creates a new GitMainMaster rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitMainMaster)
    }
}

impl Default for GitMainMaster {
    fn default() -> Self {
        GitMainMaster
    }
}

impl Rule for GitMainMaster {
    fn name(&self) -> &str {
        "git_main_master"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !cmd
            .output
            .contains("did not match any file(s) known to git")
        {
            return false;
        }

        // Check if the command mentions 'main' or 'master'
        (cmd.script.contains(" main") || cmd.script.contains(" master"))
            && (cmd.output.contains("'main'") || cmd.output.contains("'master'"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if cmd.script.contains(" main") {
            vec![cmd.script.replace(" main", " master")]
        } else {
            vec![cmd.script.replace(" master", " main")]
        }
    }

    fn priority(&self) -> i32 {
        // Higher priority than generic git_checkout
        900
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_checkout_matches() {
        let rule = GitCheckout;
        let cmd = Command::new(
            "git checkout featur",
            "error: pathspec 'featur' did not match any file(s) known to git\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_checkout_no_match_git_add() {
        let rule = GitCheckout;
        let cmd = Command::new(
            "git checkout newfile.txt",
            "error: pathspec 'newfile.txt' did not match any file(s) known to git\n\
             Did you forget to 'git add'?\n",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_checkout_no_match_success() {
        let rule = GitCheckout;
        let cmd = Command::new("git checkout main", "Switched to branch 'main'\n");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_checkout_get_new_command_suggests_checkout_b() {
        let rule = GitCheckout;
        let cmd = Command::new(
            "git checkout newbranch",
            "error: pathspec 'newbranch' did not match any file(s) known to git\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        // Should suggest checkout -b for creating a new branch
        assert!(new_commands.iter().any(|c| c.contains("checkout -b")));
    }

    #[test]
    fn test_git_checkout_uncommitted_changes_matches() {
        let rule = GitCheckoutUncommittedChanges;
        let cmd = Command::new(
            "git checkout feature",
            "error: Your local changes to the following files would be overwritten by checkout:\n\
             \tfile.txt\n\
             Please commit your changes or stash them before you switch branches.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_checkout_uncommitted_changes_get_new_command() {
        let rule = GitCheckoutUncommittedChanges;
        let cmd = Command::new(
            "git checkout feature",
            "error: Your local changes to the following files would be overwritten by checkout:\n\
             \tfile.txt\n\
             Please commit your changes or stash them before you switch branches.\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("git stash"));
        assert!(new_commands[0].contains("git checkout feature"));
    }

    #[test]
    fn test_git_main_master_matches_main() {
        let rule = GitMainMaster;
        let cmd = Command::new(
            "git checkout main",
            "error: pathspec 'main' did not match any file(s) known to git\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_main_master_matches_master() {
        let rule = GitMainMaster;
        let cmd = Command::new(
            "git checkout master",
            "error: pathspec 'master' did not match any file(s) known to git\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_main_master_get_new_command_main() {
        let rule = GitMainMaster;
        let cmd = Command::new(
            "git checkout main",
            "error: pathspec 'main' did not match any file(s) known to git\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git checkout master"]);
    }

    #[test]
    fn test_git_main_master_get_new_command_master() {
        let rule = GitMainMaster;
        let cmd = Command::new(
            "git checkout master",
            "error: pathspec 'master' did not match any file(s) known to git\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git checkout main"]);
    }

    #[test]
    fn test_git_checkout_wrapped() {
        let rule = GitCheckout::new();

        // Should not match non-git commands
        let cmd = Command::new(
            "svn checkout featur",
            "error: pathspec 'featur' did not match any file(s) known to git\n",
        );
        assert!(!rule.is_match(&cmd));

        // Should match git commands
        let cmd = Command::new(
            "git checkout featur",
            "error: pathspec 'featur' did not match any file(s) known to git\n",
        );
        assert!(rule.is_match(&cmd));
    }
}
