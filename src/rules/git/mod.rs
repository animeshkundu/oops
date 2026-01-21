//! Git rules module.
//!
//! This module contains all git-related rules for oops.
//!
//! # Rules
//!
//! ## Push Rules (`push.rs`)
//!
//! - `GitPush` - Fixes "git push" when there's no upstream branch set
//! - `GitPushPull` - Suggests pulling before pushing when remote has new commits
//! - `GitPushForce` - Suggests force push when regular push is rejected
//! - `GitPushWithoutCommits` - Detects push without any commits
//! - `GitPushDifferentBranchNames` - Handles push with different local/remote branch names
//!
//! ## Checkout Rules (`checkout.rs`)
//!
//! - `GitCheckout` - Suggests similar branch names when checkout fails
//! - `GitCheckoutUncommittedChanges` - Suggests stashing when checkout fails due to changes
//! - `GitMainMaster` - Handles main/master branch confusion
//!
//! ## Add Rules (`add.rs`)
//!
//! - `GitAdd` - Adds untracked files that need to be added
//! - `GitAddForce` - Suggests -f flag for ignored files
//! - `GitCommitAdd` - Suggests -a flag when committing without staged changes
//! - `GitAddAll` - Suggests adding all files
//!
//! ## Branch Rules (`branch.rs`)
//!
//! - `GitBranchDelete` - Suggests -D when -d fails for unmerged branch
//! - `GitBranchDeleteCheckedOut` - Handles deleting the currently checked out branch
//! - `GitBranchExists` - Handles creating a branch that already exists
//! - `GitBranchNotFound` - Suggests similar branch names
//! - `GitBranchList` - Fixes branch listing syntax
//! - `GitBranchFlagPosition` - Fixes flag position in branch commands
//!
//! ## Not Command Rules (`not_command.rs`)
//!
//! - `GitNotCommand` - Fixes unknown git commands using git's suggestions
//! - `GitCommandTypo` - Fixes typos when git doesn't provide suggestions
//! - `GitTwoDashes` - Fixes single-dash long options
//!
//! ## Common Rules (`common.rs`)
//!
//! - `GitPull` - Fixes pull when there's no upstream
//! - `GitPullUncommittedChanges` - Suggests stashing for pull
//! - `GitStash` - Suggests stashing when needed
//! - `GitStashPop` - Handles stash pop conflicts
//! - `GitCommitAmend` - Suggests amend for empty commit message
//! - `GitCommitReset` - Suggests reset after nothing to commit
//! - `GitDiffStaged` - Suggests --staged when diff is empty
//! - `GitMerge` - Handles merge conflicts
//! - `GitMergeUnrelated` - Handles unrelated histories
//! - `GitRebase` - Handles rebase conflicts
//! - `GitRebaseNoChanges` - Suggests skip when rebase has no changes
//! - `GitRmLocalModifications` - Suggests -f for rm on modified files
//! - `GitRmRecursive` - Suggests -r for rm on directories
//! - `GitRemoteDelete` - Fixes remote delete syntax
//! - `GitTagForce` - Suggests -f for existing tags
//! - `GitHookBypass` - Suggests --no-verify to bypass hooks
//! - `GitPullClone` - Suggests clone when pull fails in non-repo
//! - `GitCloneGitClone` - Fixes "git clone git clone" typo

pub mod add;
pub mod branch;
pub mod checkout;
pub mod common;
pub mod not_command;
pub mod push;
pub mod support;

// Re-export support types and functions
pub use support::{
    and_commands, expand_git_alias, get_all_matched_commands, get_branches, get_close_matches,
    get_closest, get_current_branch, is_app, is_git_command, replace_argument, replace_command,
    GitSupport,
};

// Re-export core types (Command and Rule come from crate::core via support)
pub use crate::core::{Command, Rule};

// Re-export push rules
pub use push::{
    GitPush, GitPushDifferentBranchNames, GitPushForce, GitPushPull, GitPushWithoutCommits,
};

// Re-export checkout rules
pub use checkout::{GitCheckout, GitCheckoutUncommittedChanges, GitMainMaster};

// Re-export add rules
pub use add::{GitAdd, GitAddAll, GitAddForce, GitCommitAdd};

// Re-export branch rules
pub use branch::{
    GitBranchDelete, GitBranchDeleteCheckedOut, GitBranchExists, GitBranchFlagPosition,
    GitBranchList, GitBranchNotFound,
};

// Re-export not_command rules
pub use not_command::{GitCommandTypo, GitNotCommand, GitTwoDashes};

// Re-export common rules
pub use common::{
    // New rules
    GitBisectUsage,
    GitCloneGitClone,
    GitCloneMissing,
    GitCommitAmend,
    GitCommitReset,
    GitDiffNoIndex,
    GitDiffStaged,
    GitFixStash,
    GitFlagAfterFilename,
    GitHelpAliased,
    GitHookBypass,
    GitLfsMistype,
    GitMerge,
    GitMergeUnrelated,
    GitPull,
    GitPullClone,
    GitPullUncommittedChanges,
    GitRebase,
    GitRebaseMergeDir,
    GitRebaseNoChanges,
    GitRemoteDelete,
    GitRemoteSeturlAdd,
    GitRmLocalModifications,
    GitRmRecursive,
    GitRmStaged,
    GitStash,
    GitStashPop,
    GitTagForce,
};

/// Returns all git rules.
///
/// This function creates instances of all git rules wrapped with GitSupport.
/// The rules are returned in order of priority (higher priority first).
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // Push rules
        Box::new(GitPush::new()),
        Box::new(GitPushPull::new()),
        Box::new(GitPushForce::new()),
        Box::new(GitPushWithoutCommits::new()),
        Box::new(GitPushDifferentBranchNames::new()),
        // Checkout rules
        Box::new(GitCheckout::new()),
        Box::new(GitCheckoutUncommittedChanges::new()),
        Box::new(GitMainMaster::new()),
        // Add rules
        Box::new(GitAdd::new()),
        Box::new(GitAddForce::new()),
        Box::new(GitCommitAdd::new()),
        Box::new(GitAddAll::new()),
        // Branch rules
        Box::new(GitBranchDelete::new()),
        Box::new(GitBranchDeleteCheckedOut::new()),
        Box::new(GitBranchExists::new()),
        Box::new(GitBranchNotFound::new()),
        Box::new(GitBranchList::new()),
        Box::new(GitBranchFlagPosition::new()),
        // Not command rules
        Box::new(GitNotCommand::new()),
        Box::new(GitCommandTypo::new()),
        Box::new(GitTwoDashes::new()),
        // Common rules
        Box::new(GitPull::new()),
        Box::new(GitPullUncommittedChanges::new()),
        Box::new(GitStash::new()),
        Box::new(GitStashPop::new()),
        Box::new(GitCommitAmend::new()),
        Box::new(GitCommitReset::new()),
        Box::new(GitDiffStaged::new()),
        Box::new(GitMerge::new()),
        Box::new(GitMergeUnrelated::new()),
        Box::new(GitRebase::new()),
        Box::new(GitRebaseNoChanges::new()),
        Box::new(GitRmLocalModifications::new()),
        Box::new(GitRmRecursive::new()),
        Box::new(GitRemoteDelete::new()),
        Box::new(GitTagForce::new()),
        Box::new(GitHookBypass::new()),
        Box::new(GitPullClone::new()),
        Box::new(GitCloneGitClone::new()),
        // New common rules
        Box::new(GitBisectUsage::new()),
        Box::new(GitCloneMissing::new()),
        Box::new(GitDiffNoIndex::new()),
        Box::new(GitFixStash::new()),
        Box::new(GitFlagAfterFilename::new()),
        Box::new(GitHelpAliased::new()),
        Box::new(GitLfsMistype::new()),
        Box::new(GitRebaseMergeDir::new()),
        Box::new(GitRemoteSeturlAdd::new()),
        Box::new(GitRmStaged::new()),
    ]
}

/// Returns the names of all git rules.
pub fn rule_names() -> Vec<&'static str> {
    vec![
        "git_push",
        "git_push_pull",
        "git_push_force",
        "git_push_without_commits",
        "git_push_different_branch_names",
        "git_checkout",
        "git_checkout_uncommitted_changes",
        "git_main_master",
        "git_add",
        "git_add_force",
        "git_commit_add",
        "git_add_all",
        "git_branch_delete",
        "git_branch_delete_checked_out",
        "git_branch_exists",
        "git_branch_not_found",
        "git_branch_list",
        "git_branch_0flag",
        "git_not_command",
        "git_command_typo",
        "git_two_dashes",
        "git_pull",
        "git_pull_uncommitted_changes",
        "git_stash",
        "git_stash_pop",
        "git_commit_amend",
        "git_commit_reset",
        "git_diff_staged",
        "git_merge",
        "git_merge_unrelated",
        "git_rebase",
        "git_rebase_no_changes",
        "git_rm_local_modifications",
        "git_rm_recursive",
        "git_remote_delete",
        "git_tag_force",
        "git_hook_bypass",
        "git_pull_clone",
        "git_clone_git_clone",
        // New rules
        "git_bisect_usage",
        "git_clone_missing",
        "git_diff_no_index",
        "git_fix_stash",
        "git_flag_after_filename",
        "git_help_aliased",
        "git_lfs_mistype",
        "git_rebase_merge_dir",
        "git_remote_seturl_add",
        "git_rm_staged",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_returns_rules() {
        let rules = all_rules();
        assert!(!rules.is_empty());
        // Verify we have all expected rules
        assert!(rules.len() >= 30);
    }

    #[test]
    fn test_rule_names_match_rules() {
        let rules = all_rules();
        let names = rule_names();
        assert_eq!(rules.len(), names.len());

        // Verify each rule has a matching name
        for (rule, name) in rules.iter().zip(names.iter()) {
            assert_eq!(rule.name(), *name);
        }
    }

    #[test]
    fn test_git_rules_only_match_git_commands() {
        let rules = all_rules();

        // Non-git command
        let non_git_cmd = Command::new("svn status", "some output");

        for rule in rules.iter() {
            // Rules that don't require output might still match
            if rule.requires_output() {
                assert!(
                    !rule.is_match(&non_git_cmd),
                    "Rule {} should not match non-git commands",
                    rule.name()
                );
            }
        }
    }

    #[test]
    fn test_rules_have_valid_priorities() {
        let rules = all_rules();

        for rule in rules.iter() {
            // Priority should be a reasonable value
            assert!(
                rule.priority() > 0,
                "Rule {} has invalid priority",
                rule.name()
            );
            assert!(
                rule.priority() <= 2000,
                "Rule {} has too high priority",
                rule.name()
            );
        }
    }
}
