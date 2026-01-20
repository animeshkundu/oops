//! Git add related rules.
//!
//! This module contains rules for fixing common git add issues.

use regex::Regex;
use std::path::Path;

use super::support::{and_commands, replace_argument, Command, GitSupport, Rule};

/// Rule for adding untracked/modified files.
///
/// Matches when a git command fails because a file needs to be added first.
pub struct GitAdd;

impl GitAdd {
    /// Creates a new GitAdd rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitAdd)
    }
}

impl Default for GitAdd {
    fn default() -> Self {
        GitAdd
    }
}

impl Rule for GitAdd {
    fn name(&self) -> &str {
        "git_add"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !cmd.output.contains("did not match any file(s) known to git.") {
            return false;
        }

        // Check if the missing file actually exists (needs to be added)
        if let Some(missing_file) = get_missing_file(&cmd.output) {
            return Path::new(&missing_file).exists();
        }

        false
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(missing_file) = get_missing_file(&cmd.output) {
            // Suggest: git add -- <file> && <original_command>
            return vec![and_commands(
                &format!("git add -- {}", missing_file),
                &cmd.script,
            )];
        }
        vec![]
    }
}

/// Extract the missing file from git's error output.
fn get_missing_file(output: &str) -> Option<String> {
    let re =
        Regex::new(r"error: pathspec '([^']*)' did not match any file\(s\) known to git.").unwrap();
    re.captures(output)
        .and_then(|captures| captures.get(1))
        .map(|m| m.as_str().to_string())
}

/// Rule for handling "git add" when files are .gitignored.
///
/// Matches when trying to add files that are ignored by .gitignore.
pub struct GitAddForce;

impl GitAddForce {
    /// Creates a new GitAddForce rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitAddForce)
    }
}

impl Default for GitAddForce {
    fn default() -> Self {
        GitAddForce
    }
}

impl Rule for GitAddForce {
    fn name(&self) -> &str {
        "git_add_force"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("add")
            && (cmd.output.contains("Use -f if you really want to add")
                || cmd.output.contains("ignored by one of your .gitignore files"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "add", "add -f")]
    }
}

/// Rule for handling "git commit" when there are no staged changes.
///
/// Matches when trying to commit but there are no changes staged.
pub struct GitCommitAdd;

impl GitCommitAdd {
    /// Creates a new GitCommitAdd rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitCommitAdd)
    }
}

impl Default for GitCommitAdd {
    fn default() -> Self {
        GitCommitAdd
    }
}

impl Rule for GitCommitAdd {
    fn name(&self) -> &str {
        "git_commit_add"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("commit") && cmd.output.contains("no changes added to commit")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Suggest both -a (add tracked files) and -p (interactive patch)
        vec![
            replace_argument(&cmd.script, "commit", "commit -a"),
            replace_argument(&cmd.script, "commit", "commit -p"),
        ]
    }
}

/// Rule for adding all modified/untracked files.
///
/// Matches when git status shows untracked or modified files.
pub struct GitAddAll;

impl GitAddAll {
    /// Creates a new GitAddAll rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitAddAll)
    }
}

impl Default for GitAddAll {
    fn default() -> Self {
        GitAddAll
    }
}

impl Rule for GitAddAll {
    fn name(&self) -> &str {
        "git_add_all"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Match when running git status and there are changes to add
        cmd.script.contains("status")
            && (cmd.output.contains("Changes not staged for commit")
                || cmd.output.contains("Untracked files"))
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        vec!["git add .".to_string(), "git add -A".to_string()]
    }

    fn priority(&self) -> i32 {
        // Lower priority since this is a convenience, not a fix
        1100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_missing_file() {
        let output = "error: pathspec 'newfile.txt' did not match any file(s) known to git.\n";
        assert_eq!(get_missing_file(output), Some("newfile.txt".to_string()));
    }

    #[test]
    fn test_get_missing_file_no_match() {
        let output = "Everything up-to-date\n";
        assert_eq!(get_missing_file(output), None);
    }

    #[test]
    fn test_git_add_force_matches() {
        let rule = GitAddForce;
        let cmd = Command::new(
            "git add secret.key",
            "The following paths are ignored by one of your .gitignore files:\n\
             secret.key\n\
             Use -f if you really want to add them.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_add_force_no_match() {
        let rule = GitAddForce;
        let cmd = Command::new("git add file.txt", "");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_add_force_get_new_command() {
        let rule = GitAddForce;
        let cmd = Command::new(
            "git add secret.key",
            "The following paths are ignored by one of your .gitignore files:\n\
             secret.key\n\
             Use -f if you really want to add them.\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git add -f secret.key"]);
    }

    #[test]
    fn test_git_commit_add_matches() {
        let rule = GitCommitAdd;
        let cmd = Command::new(
            "git commit -m 'test'",
            "On branch main\n\
             Changes not staged for commit:\n\
             \tmodified:   file.txt\n\n\
             no changes added to commit\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_commit_add_no_match() {
        let rule = GitCommitAdd;
        let cmd = Command::new(
            "git commit -m 'test'",
            "[main abc1234] test\n 1 file changed, 1 insertion(+)\n",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_commit_add_get_new_command() {
        let rule = GitCommitAdd;
        let cmd = Command::new(
            "git commit -m 'test'",
            "no changes added to commit\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(new_commands.iter().any(|c| c.contains("commit -a")));
        assert!(new_commands.iter().any(|c| c.contains("commit -p")));
    }

    #[test]
    fn test_git_add_all_matches_untracked() {
        let rule = GitAddAll;
        let cmd = Command::new(
            "git status",
            "On branch main\n\n\
             Untracked files:\n\
             \t(use \"git add <file>...\" to include in what will be committed)\n\
             \n\
             \tnewfile.txt\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_add_all_matches_modified() {
        let rule = GitAddAll;
        let cmd = Command::new(
            "git status",
            "On branch main\n\n\
             Changes not staged for commit:\n\
             \t(use \"git add <file>...\" to update what will be committed)\n\
             \n\
             \tmodified:   file.txt\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_add_all_no_match() {
        let rule = GitAddAll;
        let cmd = Command::new(
            "git status",
            "On branch main\n\
             nothing to commit, working tree clean\n",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_add_all_get_new_command() {
        let rule = GitAddAll;
        let cmd = Command::new(
            "git status",
            "Untracked files:\n\tnewfile.txt\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(new_commands.contains(&"git add .".to_string()));
        assert!(new_commands.contains(&"git add -A".to_string()));
    }

    #[test]
    fn test_git_add_wrapped() {
        let rule = GitAddForce::new();

        // Should not match non-git commands
        let cmd = Command::new(
            "svn add secret.key",
            "Use -f if you really want to add them.\n",
        );
        assert!(!rule.is_match(&cmd));

        // Should match git commands
        let cmd = Command::new(
            "git add secret.key",
            "Use -f if you really want to add them.\n",
        );
        assert!(rule.is_match(&cmd));
    }
}
