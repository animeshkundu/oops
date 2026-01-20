//! Git push related rules.
//!
//! This module contains rules for fixing common git push issues.

use regex::Regex;

use super::support::{
    and_commands, replace_argument, Command, GitSupport, Rule,
};

/// Rule for handling "git push" when there's no upstream branch set.
///
/// Matches when git suggests using `--set-upstream` to push.
pub struct GitPush;

impl GitPush {
    /// Creates a new GitPush rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitPush)
    }
}

impl Default for GitPush {
    fn default() -> Self {
        GitPush
    }
}

impl Rule for GitPush {
    fn name(&self) -> &str {
        "git_push"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("push") && cmd.output.contains("git push --set-upstream")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Parse the script into parts
        let parts: Vec<&str> = cmd.script.split_whitespace().collect();
        let mut command_parts: Vec<String> = parts.iter().map(|s| s.to_string()).collect();

        // Find and handle --set-upstream or -u option
        let upstream_index = command_parts
            .iter()
            .position(|p| p == "--set-upstream" || p == "-u");

        if let Some(idx) = upstream_index {
            // Remove --set-upstream/-u and its argument
            command_parts.remove(idx);
            if command_parts.len() > idx {
                command_parts.remove(idx);
            }
        } else {
            // Remove non-option arguments after 'push' to avoid duplication
            // since git's suggestion includes repository and refspec
            if let Some(push_idx) = command_parts.iter().position(|p| p == "push") {
                let mut i = command_parts.len();
                while i > push_idx + 1 {
                    i -= 1;
                    if !command_parts[i].starts_with('-') {
                        command_parts.remove(i);
                    }
                }
            }
        }

        // Extract the suggested push command from git's output
        let re = Regex::new(r"git push (.*)").unwrap();
        if let Some(captures) = re.captures_iter(&cmd.output).last() {
            let arguments = captures
                .get(1)
                .map(|m| m.as_str())
                .unwrap_or("")
                .replace("'", "\\'")
                .trim()
                .to_string();

            let base_cmd = command_parts.join(" ");
            return vec![replace_argument(&base_cmd, "push", &format!("push {}", arguments))];
        }

        vec![]
    }
}

/// Rule for handling push rejection when remote has new commits.
///
/// Matches when push is rejected because the remote contains work
/// that you don't have locally.
pub struct GitPushPull;

impl GitPushPull {
    /// Creates a new GitPushPull rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitPushPull)
    }
}

impl Default for GitPushPull {
    fn default() -> Self {
        GitPushPull
    }
}

impl Rule for GitPushPull {
    fn name(&self) -> &str {
        "git_push_pull"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("push")
            && cmd.output.contains("! [rejected]")
            && cmd.output.contains("failed to push some refs to")
            && (cmd
                .output
                .contains("Updates were rejected because the tip of your current branch is behind")
                || cmd
                    .output
                    .contains("Updates were rejected because the remote contains work that you do"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let pull_cmd = replace_argument(&cmd.script, "push", "pull");
        vec![and_commands(&pull_cmd, &cmd.script)]
    }
}

/// Rule for handling push with force when needed.
///
/// Matches when push is rejected and suggests force pushing.
pub struct GitPushForce;

impl GitPushForce {
    /// Creates a new GitPushForce rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitPushForce)
    }
}

impl Default for GitPushForce {
    fn default() -> Self {
        GitPushForce
    }
}

impl Rule for GitPushForce {
    fn name(&self) -> &str {
        "git_push_force"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("push")
            && cmd.output.contains("! [rejected]")
            && (cmd.output.contains("failed to push some refs to")
                || cmd.output.contains("Updates were rejected"))
            && !cmd.output.contains("the remote contains work that you do")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Prefer --force-with-lease over --force for safety
        vec![format!("{} --force-with-lease", cmd.script)]
    }

    fn priority(&self) -> i32 {
        // Lower priority than git_push_pull since force is more dangerous
        900
    }
}

/// Rule for handling push without any commits.
///
/// Matches when trying to push but there are no commits yet.
pub struct GitPushWithoutCommits;

impl GitPushWithoutCommits {
    /// Creates a new GitPushWithoutCommits rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitPushWithoutCommits)
    }
}

impl Default for GitPushWithoutCommits {
    fn default() -> Self {
        GitPushWithoutCommits
    }
}

impl Rule for GitPushWithoutCommits {
    fn name(&self) -> &str {
        "git_push_without_commits"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("push")
            && (cmd.output.contains("src refspec") && cmd.output.contains("does not match any"))
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        vec!["git commit".to_string()]
    }
}

/// Rule for handling push with different local and remote branch names.
///
/// Matches when pushing to a remote branch with a different name.
pub struct GitPushDifferentBranchNames;

impl GitPushDifferentBranchNames {
    /// Creates a new GitPushDifferentBranchNames rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitPushDifferentBranchNames)
    }
}

impl Default for GitPushDifferentBranchNames {
    fn default() -> Self {
        GitPushDifferentBranchNames
    }
}

impl Rule for GitPushDifferentBranchNames {
    fn name(&self) -> &str {
        "git_push_different_branch_names"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("push")
            && cmd.output.contains("push your current branch")
            && cmd.output.contains("with the same name on the remote")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the branch name from output
        let re = Regex::new(r"To push to the upstream branch on the remote, use\n\s+git push ([^\n]+)").unwrap();
        if let Some(captures) = re.captures(&cmd.output) {
            let suggestion = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            return vec![format!("git push {}", suggestion.trim())];
        }

        // Fallback: add HEAD to the push command
        vec![format!("{} HEAD", cmd.script)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_push_matches() {
        let rule = GitPush;
        let cmd = Command::new(
            "git push",
            "fatal: The current branch feature has no upstream branch.\n\
             To push the current branch and set the remote as upstream, use\n\n\
             git push --set-upstream origin feature\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_push_no_match() {
        let rule = GitPush;
        let cmd = Command::new("git push", "Everything up-to-date");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_push_get_new_command() {
        let rule = GitPush;
        let cmd = Command::new(
            "git push",
            "fatal: The current branch feature has no upstream branch.\n\
             To push the current branch and set the remote as upstream, use\n\n\
             git push --set-upstream origin feature\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("--set-upstream origin feature"));
    }

    #[test]
    fn test_git_push_with_existing_upstream() {
        let rule = GitPush;
        let cmd = Command::new(
            "git push -u origin",
            "fatal: The current branch feature has no upstream branch.\n\
             To push the current branch and set the remote as upstream, use\n\n\
             git push --set-upstream origin feature\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("--set-upstream"));
    }

    #[test]
    fn test_git_push_pull_matches() {
        let rule = GitPushPull;
        let cmd = Command::new(
            "git push origin main",
            "To github.com:user/repo.git\n\
             ! [rejected]        main -> main (non-fast-forward)\n\
             error: failed to push some refs to 'github.com:user/repo.git'\n\
             hint: Updates were rejected because the tip of your current branch is behind\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_push_pull_get_new_command() {
        let rule = GitPushPull;
        let cmd = Command::new(
            "git push origin main",
            "To github.com:user/repo.git\n\
             ! [rejected]        main -> main (non-fast-forward)\n\
             error: failed to push some refs to 'github.com:user/repo.git'\n\
             hint: Updates were rejected because the tip of your current branch is behind\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("git pull origin main"));
        assert!(new_commands[0].contains("&&"));
        assert!(new_commands[0].contains("git push origin main"));
    }

    #[test]
    fn test_git_push_force_matches() {
        let rule = GitPushForce;
        let cmd = Command::new(
            "git push origin feature",
            "To github.com:user/repo.git\n\
             ! [rejected]        feature -> feature (non-fast-forward)\n\
             error: failed to push some refs to 'github.com:user/repo.git'\n\
             hint: Updates were rejected because the tip of your current branch is behind\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_push_force_get_new_command() {
        let rule = GitPushForce;
        let cmd = Command::new(
            "git push origin feature",
            "To github.com:user/repo.git\n\
             ! [rejected]        feature -> feature\n\
             error: failed to push some refs to 'github.com:user/repo.git'\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("--force-with-lease"));
    }

    #[test]
    fn test_git_push_without_commits_matches() {
        let rule = GitPushWithoutCommits;
        let cmd = Command::new(
            "git push origin main",
            "error: src refspec main does not match any\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_push_without_commits_get_new_command() {
        let rule = GitPushWithoutCommits;
        let cmd = Command::new(
            "git push origin main",
            "error: src refspec main does not match any\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git commit"]);
    }

    #[test]
    fn test_git_push_wrapped() {
        let rule = GitPush::new();

        // Should not match non-git commands
        let cmd = Command::new("ls push", "git push --set-upstream origin feature");
        assert!(!rule.is_match(&cmd));

        // Should match git commands
        let cmd = Command::new("git push", "git push --set-upstream origin feature");
        assert!(rule.is_match(&cmd));
    }
}
