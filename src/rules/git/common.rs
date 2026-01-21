//! Common git rules.
//!
//! This module contains rules for common git operations.

use regex::Regex;

use super::support::{
    and_commands, get_all_matched_commands, get_closest, replace_argument, replace_command,
    Command, GitSupport, Rule,
};

/// Rule for handling git pull when there's no upstream set.
///
/// Matches when git pull fails because there's no tracking information.
pub struct GitPull;

impl GitPull {
    /// Creates a new GitPull rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitPull)
    }
}

impl Default for GitPull {
    fn default() -> Self {
        GitPull
    }
}

impl Rule for GitPull {
    fn name(&self) -> &str {
        "git_pull"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("pull") && cmd.output.contains("set-upstream")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the suggested upstream command from git's output
        let lines: Vec<&str> = cmd.output.lines().collect();

        // Look for the line with the git branch --set-upstream-to command
        for (i, line) in lines.iter().enumerate() {
            if line.contains("git branch --set-upstream-to") {
                // Use this line's suggestion
                let set_upstream = line.trim().to_string();
                return vec![and_commands(&set_upstream, &cmd.script)];
            }
            if line.contains("git pull <remote> <branch>") {
                // Extract branch name from context
                if let Some(branch_line) = lines.get(i + 1) {
                    let branch = branch_line.split_whitespace().last().unwrap_or("main");
                    let set_upstream =
                        format!("git branch --set-upstream-to=origin/{} {}", branch, branch);
                    return vec![and_commands(&set_upstream, &cmd.script)];
                }
            }
        }

        // Fallback: suggest setting upstream to origin/main
        vec![and_commands(
            "git branch --set-upstream-to=origin/main main",
            &cmd.script,
        )]
    }
}

/// Rule for handling git pull with uncommitted changes.
///
/// Matches when git pull fails because of uncommitted changes.
pub struct GitPullUncommittedChanges;

impl GitPullUncommittedChanges {
    /// Creates a new GitPullUncommittedChanges rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitPullUncommittedChanges)
    }
}

impl Default for GitPullUncommittedChanges {
    fn default() -> Self {
        GitPullUncommittedChanges
    }
}

impl Rule for GitPullUncommittedChanges {
    fn name(&self) -> &str {
        "git_pull_uncommitted_changes"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("pull")
            && (cmd.output.contains("You have unstaged changes")
                || cmd
                    .output
                    .contains("Your local changes to the following files would be overwritten")
                || cmd
                    .output
                    .contains("Please commit your changes or stash them"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![and_commands(
            "git stash",
            &format!("{} && git stash pop", cmd.script),
        )]
    }
}

/// Rule for handling stash operations.
///
/// Matches when an operation needs stashing first.
pub struct GitStash;

impl GitStash {
    /// Creates a new GitStash rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitStash)
    }
}

impl Default for GitStash {
    fn default() -> Self {
        GitStash
    }
}

impl Rule for GitStash {
    fn name(&self) -> &str {
        "git_stash"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Matches "Please commit or stash them" and similar messages
        cmd.output.contains("or stash them")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![and_commands("git stash", &cmd.script)]
    }
}

/// Rule for handling stash pop conflicts.
///
/// Matches when git stash pop fails due to conflicts.
pub struct GitStashPop;

impl GitStashPop {
    /// Creates a new GitStashPop rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitStashPop)
    }
}

impl Default for GitStashPop {
    fn default() -> Self {
        GitStashPop
    }
}

impl Rule for GitStashPop {
    fn name(&self) -> &str {
        "git_stash_pop"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("stash")
            && cmd.script.contains("pop")
            && cmd.output.contains("CONFLICT")
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        // Suggest dropping the stash after resolving conflicts
        vec![
            "git add . && git stash drop".to_string(),
            "git checkout --theirs . && git add . && git stash drop".to_string(),
            "git checkout --ours . && git add . && git stash drop".to_string(),
        ]
    }
}

/// Rule for handling git commit amend.
///
/// Matches when user wants to amend the last commit.
pub struct GitCommitAmend;

impl GitCommitAmend {
    /// Creates a new GitCommitAmend rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitCommitAmend)
    }
}

impl Default for GitCommitAmend {
    fn default() -> Self {
        GitCommitAmend
    }
}

impl Rule for GitCommitAmend {
    fn name(&self) -> &str {
        "git_commit_amend"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("commit") && cmd.output.contains("empty commit message")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "commit", "commit --amend")]
    }
}

/// Rule for handling git commit reset.
///
/// Suggests resetting after a bad commit.
pub struct GitCommitReset;

impl GitCommitReset {
    /// Creates a new GitCommitReset rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitCommitReset)
    }
}

impl Default for GitCommitReset {
    fn default() -> Self {
        GitCommitReset
    }
}

impl Rule for GitCommitReset {
    fn name(&self) -> &str {
        "git_commit_reset"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("commit") && cmd.output.contains("nothing to commit")
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        vec!["git reset HEAD~".to_string()]
    }
}

/// Rule for handling git diff staged.
///
/// Matches when git diff shows nothing but there are staged changes.
pub struct GitDiffStaged;

impl GitDiffStaged {
    /// Creates a new GitDiffStaged rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitDiffStaged)
    }
}

impl Default for GitDiffStaged {
    fn default() -> Self {
        GitDiffStaged
    }
}

impl Rule for GitDiffStaged {
    fn name(&self) -> &str {
        "git_diff_staged"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script == "git diff" && cmd.output.is_empty()
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        vec!["git diff --staged".to_string(), "git diff HEAD".to_string()]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

/// Rule for handling git merge conflicts.
///
/// Matches when a merge operation has conflicts.
pub struct GitMerge;

impl GitMerge {
    /// Creates a new GitMerge rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitMerge)
    }
}

impl Default for GitMerge {
    fn default() -> Self {
        GitMerge
    }
}

impl Rule for GitMerge {
    fn name(&self) -> &str {
        "git_merge"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("merge")
            && (cmd.output.contains("CONFLICT") || cmd.output.contains("Automatic merge failed"))
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        vec![
            "git merge --abort".to_string(),
            "git add . && git commit".to_string(),
        ]
    }
}

/// Rule for handling git merge unrelated histories.
///
/// Matches when git refuses to merge unrelated histories.
pub struct GitMergeUnrelated;

impl GitMergeUnrelated {
    /// Creates a new GitMergeUnrelated rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitMergeUnrelated)
    }
}

impl Default for GitMergeUnrelated {
    fn default() -> Self {
        GitMergeUnrelated
    }
}

impl Rule for GitMergeUnrelated {
    fn name(&self) -> &str {
        "git_merge_unrelated"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output.contains("refusing to merge unrelated histories")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("{} --allow-unrelated-histories", cmd.script)]
    }
}

/// Rule for handling git rebase conflicts.
///
/// Matches when a rebase operation has conflicts.
pub struct GitRebase;

impl GitRebase {
    /// Creates a new GitRebase rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitRebase)
    }
}

impl Default for GitRebase {
    fn default() -> Self {
        GitRebase
    }
}

impl Rule for GitRebase {
    fn name(&self) -> &str {
        "git_rebase"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        (cmd.script.contains("rebase") || cmd.output.contains("rebase in progress"))
            && cmd.output.contains("CONFLICT")
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        vec![
            "git rebase --abort".to_string(),
            "git add . && git rebase --continue".to_string(),
        ]
    }
}

/// Rule for handling git rebase with no changes.
///
/// Matches when a rebase produces no changes.
pub struct GitRebaseNoChanges;

impl GitRebaseNoChanges {
    /// Creates a new GitRebaseNoChanges rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitRebaseNoChanges)
    }
}

impl Default for GitRebaseNoChanges {
    fn default() -> Self {
        GitRebaseNoChanges
    }
}

impl Rule for GitRebaseNoChanges {
    fn name(&self) -> &str {
        "git_rebase_no_changes"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output
            .contains("No changes - did you forget to use 'git add'")
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        vec!["git rebase --skip".to_string()]
    }
}

/// Rule for handling git rm on modified files.
///
/// Matches when git rm fails because file has local modifications.
pub struct GitRmLocalModifications;

impl GitRmLocalModifications {
    /// Creates a new GitRmLocalModifications rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitRmLocalModifications)
    }
}

impl Default for GitRmLocalModifications {
    fn default() -> Self {
        GitRmLocalModifications
    }
}

impl Rule for GitRmLocalModifications {
    fn name(&self) -> &str {
        "git_rm_local_modifications"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("rm")
            && (cmd.output.contains("has local modifications")
                || cmd.output.contains("has changes staged in the index"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "rm", "rm -f")]
    }
}

/// Rule for handling git rm on directories.
///
/// Matches when git rm fails because target is a directory.
pub struct GitRmRecursive;

impl GitRmRecursive {
    /// Creates a new GitRmRecursive rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitRmRecursive)
    }
}

impl Default for GitRmRecursive {
    fn default() -> Self {
        GitRmRecursive
    }
}

impl Rule for GitRmRecursive {
    fn name(&self) -> &str {
        "git_rm_recursive"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("rm")
            && (cmd.output.contains("not removing '") && cmd.output.contains("' recursively"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "rm", "rm -r")]
    }
}

/// Rule for handling git remote delete.
///
/// Matches when trying to delete a remote with the wrong syntax.
pub struct GitRemoteDelete;

impl GitRemoteDelete {
    /// Creates a new GitRemoteDelete rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitRemoteDelete)
    }
}

impl Default for GitRemoteDelete {
    fn default() -> Self {
        GitRemoteDelete
    }
}

impl Rule for GitRemoteDelete {
    fn name(&self) -> &str {
        "git_remote_delete"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("remote")
            && cmd.script.contains("delete")
            && cmd.output.contains("error:")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "delete", "remove")]
    }
}

/// Rule for handling git tag force.
///
/// Matches when trying to create a tag that already exists.
pub struct GitTagForce;

impl GitTagForce {
    /// Creates a new GitTagForce rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitTagForce)
    }
}

impl Default for GitTagForce {
    fn default() -> Self {
        GitTagForce
    }
}

impl Rule for GitTagForce {
    fn name(&self) -> &str {
        "git_tag_force"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("tag") && cmd.output.contains("already exists")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "tag", "tag -f")]
    }
}

/// Rule for handling git hook bypass.
///
/// Matches when a git hook blocks an operation.
pub struct GitHookBypass;

impl GitHookBypass {
    /// Creates a new GitHookBypass rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitHookBypass)
    }
}

impl Default for GitHookBypass {
    fn default() -> Self {
        GitHookBypass
    }
}

impl Rule for GitHookBypass {
    fn name(&self) -> &str {
        "git_hook_bypass"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        (cmd.script.contains("commit") || cmd.script.contains("push"))
            && (cmd.output.contains("hook") || cmd.output.contains("pre-commit"))
            && cmd.output.contains("failed")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("{} --no-verify", cmd.script)]
    }

    fn priority(&self) -> i32 {
        // Lower priority since bypassing hooks can be risky
        1100
    }
}

/// Rule for handling git clone when already in a repo.
///
/// Matches when trying to git pull in a non-repo but a .git exists nearby.
pub struct GitPullClone;

impl GitPullClone {
    /// Creates a new GitPullClone rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitPullClone)
    }
}

impl Default for GitPullClone {
    fn default() -> Self {
        GitPullClone
    }
}

impl Rule for GitPullClone {
    fn name(&self) -> &str {
        "git_pull_clone"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("pull") && cmd.output.contains("not a git repository")
    }

    fn get_new_command(&self, _cmd: &Command) -> Vec<String> {
        vec!["git clone".to_string()]
    }
}

/// Rule for handling clone git clone typo.
///
/// Matches when user types "git clone git clone <url>".
pub struct GitCloneGitClone;

impl GitCloneGitClone {
    /// Creates a new GitCloneGitClone rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitCloneGitClone)
    }
}

impl Default for GitCloneGitClone {
    fn default() -> Self {
        GitCloneGitClone
    }
}

impl Rule for GitCloneGitClone {
    fn name(&self) -> &str {
        "git_clone_git_clone"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let re = Regex::new(r"git clone git clone ").unwrap();
        re.is_match(&cmd.script)
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![cmd.script.replace("git clone git clone ", "git clone ")]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

/// Rule for handling git bisect usage errors.
///
/// Matches when git bisect command has incorrect subcommand.
pub struct GitBisectUsage;

impl GitBisectUsage {
    /// Creates a new GitBisectUsage rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitBisectUsage)
    }
}

impl Default for GitBisectUsage {
    fn default() -> Self {
        GitBisectUsage
    }
}

impl Rule for GitBisectUsage {
    fn name(&self) -> &str {
        "git_bisect_usage"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("bisect") && cmd.output.contains("usage: git bisect")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the broken subcommand from the script
        let broken_re = Regex::new(r"git bisect ([^ $]*)").unwrap();
        let usage_re = Regex::new(r"usage: git bisect \[([^\]]+)\]").unwrap();

        if let (Some(broken_cap), Some(usage_cap)) = (
            broken_re.captures(&cmd.script),
            usage_re.captures(&cmd.output),
        ) {
            let broken = broken_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let usage = usage_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let options: Vec<String> = usage.split('|').map(|s| s.trim().to_string()).collect();
            return replace_command(&cmd.script, broken, &options);
        }

        vec![]
    }
}

/// Rule for handling missing git clone command.
///
/// Matches when a git URL is pasted without the clone command.
pub struct GitCloneMissing;

impl GitCloneMissing {
    /// Creates a new GitCloneMissing rule wrapped with git support.
    pub fn new() -> Self {
        GitCloneMissing
    }
}

impl Default for GitCloneMissing {
    fn default() -> Self {
        GitCloneMissing
    }
}

impl Rule for GitCloneMissing {
    fn name(&self) -> &str {
        "git_clone_missing"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        // Must be a single URL
        if parts.len() != 1 {
            return false;
        }

        // Check for the expected error messages
        if !cmd.output.contains("No such file or directory")
            && !cmd.output.contains("not found")
            && !cmd.output.contains("is not recognised as")
            && !cmd.output.contains("is not recognized as")
        {
            return false;
        }

        let script = &cmd.script;

        // Check if it looks like a URL
        if script.starts_with("http://") || script.starts_with("https://") {
            return true;
        }

        // Check for SSH format (git@github.com:user/repo.git)
        if script.contains('@') && script.contains(':') {
            return true;
        }

        false
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("git clone {}", cmd.script)]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule for handling git diff on non-git files.
///
/// Adds --no-index flag when diffing files outside a git repository.
pub struct GitDiffNoIndex;

impl GitDiffNoIndex {
    /// Creates a new GitDiffNoIndex rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitDiffNoIndex)
    }
}

impl Default for GitDiffNoIndex {
    fn default() -> Self {
        GitDiffNoIndex
    }
}

impl Rule for GitDiffNoIndex {
    fn name(&self) -> &str {
        "git_diff_no_index"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        if !cmd.script.contains("diff") || cmd.script.contains("--no-index") {
            return false;
        }

        // Get files (non-flag arguments after "diff")
        let files: Vec<&String> = parts
            .iter()
            .skip(2) // Skip "git" and "diff"
            .filter(|arg| !arg.starts_with('-'))
            .collect();

        files.len() == 2
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "diff", "diff --no-index")]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

/// Rule for handling git stash command syntax errors.
///
/// Fixes invalid stash subcommands.
pub struct GitFixStash;

impl GitFixStash {
    /// Creates a new GitFixStash rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitFixStash)
    }
}

impl Default for GitFixStash {
    fn default() -> Self {
        GitFixStash
    }
}

const STASH_COMMANDS: &[&str] = &[
    "apply", "branch", "clear", "drop", "list", "pop", "save", "show",
];

impl Rule for GitFixStash {
    fn name(&self) -> &str {
        "git_fix_stash"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        parts.len() > 1
            && parts.get(1).map(|s| s.as_str()) == Some("stash")
            && cmd.output.contains("usage:")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();

        if parts.len() < 3 {
            return vec![];
        }

        let stash_cmd = &parts[2];
        let stash_commands: Vec<String> = STASH_COMMANDS.iter().map(|s| s.to_string()).collect();

        if let Some(fixed) = get_closest(stash_cmd, &stash_commands, false) {
            vec![replace_argument(&cmd.script, stash_cmd, &fixed)]
        } else {
            // Insert "save" after "stash"
            let mut new_parts = parts.to_vec();
            new_parts.insert(2, "save".to_string());
            vec![new_parts.join(" ")]
        }
    }
}

/// Rule for handling flags placed after filenames.
///
/// Moves flags to appear before filenames in git commands.
pub struct GitFlagAfterFilename;

impl GitFlagAfterFilename {
    /// Creates a new GitFlagAfterFilename rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitFlagAfterFilename)
    }
}

impl Default for GitFlagAfterFilename {
    fn default() -> Self {
        GitFlagAfterFilename
    }
}

impl Rule for GitFlagAfterFilename {
    fn name(&self) -> &str {
        "git_flag_after_filename"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output.contains("fatal: bad flag '") && cmd.output.contains("' used after filename")
            || cmd.output.contains("fatal: option '")
                && cmd
                    .output
                    .contains("' must come before non-option arguments")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let mut parts = cmd.script_parts().to_vec();

        // Extract the bad flag from the error message
        let bad_flag = if let Some(cap) =
            Regex::new(r"fatal: bad flag '([^']+)' used after filename")
                .ok()
                .and_then(|re| re.captures(&cmd.output))
        {
            cap.get(1).map(|m| m.as_str().to_string())
        } else if let Some(cap) =
            Regex::new(r"fatal: option '([^']+)' must come before non-option arguments")
                .ok()
                .and_then(|re| re.captures(&cmd.output))
        {
            cap.get(1).map(|m| m.as_str().to_string())
        } else {
            None
        };

        if let Some(bad_flag) = bad_flag {
            if let Some(bad_flag_index) = parts.iter().position(|p| p == &bad_flag) {
                // Find the first non-flag argument before the bad flag
                let mut filename_index = None;
                for index in (0..bad_flag_index).rev() {
                    if !parts[index].starts_with('-') {
                        filename_index = Some(index);
                        break;
                    }
                }

                if let Some(filename_index) = filename_index {
                    // Swap the flag and filename
                    parts.swap(bad_flag_index, filename_index);
                    return vec![parts.join(" ")];
                }
            }
        }

        vec![]
    }
}

/// Rule for handling help on aliased git commands.
///
/// Redirects help to the actual command when asking for help on an alias.
pub struct GitHelpAliased;

impl GitHelpAliased {
    /// Creates a new GitHelpAliased rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitHelpAliased)
    }
}

impl Default for GitHelpAliased {
    fn default() -> Self {
        GitHelpAliased
    }
}

impl Rule for GitHelpAliased {
    fn name(&self) -> &str {
        "git_help_aliased"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("help") && cmd.output.contains(" is aliased to ")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the aliased command from output like "`ci` is aliased to `commit`"
        // The output format is: `alias` is aliased to `actual command`
        if let Some(pos) = cmd.output.find(" is aliased to ") {
            let after = &cmd.output[pos + " is aliased to ".len()..];
            // Find the command between backticks
            if let Some(start) = after.find('`') {
                let rest = &after[start + 1..];
                if let Some(end) = rest.find('\'').or_else(|| rest.find('`')) {
                    let aliased = rest[..end].split_whitespace().next().unwrap_or("");
                    if !aliased.is_empty() {
                        return vec![format!("git help {}", aliased)];
                    }
                }
            }
        }
        vec![]
    }
}

/// Rule for handling git lfs command typos.
///
/// Fixes mistyped git lfs subcommands.
pub struct GitLfsMistype;

impl GitLfsMistype {
    /// Creates a new GitLfsMistype rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitLfsMistype)
    }
}

impl Default for GitLfsMistype {
    fn default() -> Self {
        GitLfsMistype
    }
}

impl Rule for GitLfsMistype {
    fn name(&self) -> &str {
        "git_lfs_mistype"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("lfs") && cmd.output.contains("Did you mean this?")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the broken command from the error
        let broken_re = Regex::new(r#"Error: unknown command "([^"]*)" for "git-lfs""#).unwrap();

        if let Some(cap) = broken_re.captures(&cmd.output) {
            let broken_cmd = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let matched = get_all_matched_commands(&cmd.output, &["Did you mean", " for usage."]);
            return replace_command(&cmd.script, broken_cmd, &matched);
        }

        vec![]
    }
}

/// Rule for handling rebase-merge directory conflicts.
///
/// Suggests rebase continue, abort, or skip when rebase-merge directory exists.
pub struct GitRebaseMergeDir;

impl GitRebaseMergeDir {
    /// Creates a new GitRebaseMergeDir rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitRebaseMergeDir)
    }
}

impl Default for GitRebaseMergeDir {
    fn default() -> Self {
        GitRebaseMergeDir
    }
}

impl Rule for GitRebaseMergeDir {
    fn name(&self) -> &str {
        "git_rebase_merge_dir"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("rebase")
            && cmd
                .output
                .contains("It seems that there is already a rebase-merge directory")
            && cmd
                .output
                .contains("I wonder if you are in the middle of another rebase")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let mut commands = vec![
            "git rebase --continue".to_string(),
            "git rebase --abort".to_string(),
            "git rebase --skip".to_string(),
        ];

        // Extract the rm command from the output (usually 4th line from the end)
        let lines: Vec<&str> = cmd.output.lines().collect();
        if lines.len() >= 4 {
            let rm_cmd = lines[lines.len() - 4].trim();
            if rm_cmd.contains("rm -fr") || rm_cmd.contains("rm -rf") {
                commands.push(rm_cmd.to_string());
            }
        }

        // Sort by similarity to original command
        let original = &cmd.script;
        let mut scored: Vec<(f64, String)> = commands
            .into_iter()
            .map(|c| (strsim::jaro_winkler(original, &c), c))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored.into_iter().map(|(_, c)| c).collect()
    }
}

/// Rule for handling git remote set-url vs add confusion.
///
/// Suggests using 'add' instead of 'set-url' when remote doesn't exist.
pub struct GitRemoteSeturlAdd;

impl GitRemoteSeturlAdd {
    /// Creates a new GitRemoteSeturlAdd rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitRemoteSeturlAdd)
    }
}

impl Default for GitRemoteSeturlAdd {
    fn default() -> Self {
        GitRemoteSeturlAdd
    }
}

impl Rule for GitRemoteSeturlAdd {
    fn name(&self) -> &str {
        "git_remote_seturl_add"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("set-url") && cmd.output.contains("fatal: No such remote")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "set-url", "add")]
    }
}

/// Rule for handling git rm on staged files.
///
/// Suggests --cached or -f flags when removing files with staged changes.
pub struct GitRmStaged;

impl GitRmStaged {
    /// Creates a new GitRmStaged rule wrapped with git support.
    pub fn new() -> GitSupport<Self> {
        GitSupport(GitRmStaged)
    }
}

impl Default for GitRmStaged {
    fn default() -> Self {
        GitRmStaged
    }
}

impl Rule for GitRmStaged {
    fn name(&self) -> &str {
        "git_rm_staged"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains(" rm ")
            && cmd
                .output
                .contains("error: the following file has changes staged in the index")
            && cmd
                .output
                .contains("use --cached to keep the file, or -f to force removal")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();

        // Find the position of "rm" in the command
        if let Some(rm_index) = parts.iter().position(|p| p == "rm") {
            let mut cached_parts = parts.to_vec();
            cached_parts.insert(rm_index + 1, "--cached".to_string());

            let mut force_parts = parts.to_vec();
            force_parts.insert(rm_index + 1, "-f".to_string());

            vec![cached_parts.join(" "), force_parts.join(" ")]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_pull_matches() {
        let rule = GitPull;
        let cmd = Command::new(
            "git pull",
            "There is no tracking information for the current branch.\n\
             Please specify which branch you want to merge with.\n\
             See git-pull(1) for details.\n\n\
             git pull <remote> <branch>\n\n\
             If you wish to set tracking information for this branch you can do so with:\n\n\
             git branch --set-upstream-to=origin/<branch> main\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_pull_no_match() {
        let rule = GitPull;
        let cmd = Command::new("git pull", "Already up to date.\n");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_stash_matches() {
        let rule = GitStash;
        let cmd = Command::new(
            "git checkout main",
            "error: Your local changes to the following files would be overwritten by checkout:\n\
             \tfile.txt\n\
             Please commit your changes or stash them before you switch branches.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_stash_get_new_command() {
        let rule = GitStash;
        let cmd = Command::new(
            "git checkout main",
            "Please commit your changes or stash them before you switch branches.\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("git stash"));
    }

    #[test]
    fn test_git_merge_matches() {
        let rule = GitMerge;
        let cmd = Command::new(
            "git merge feature",
            "CONFLICT (content): Merge conflict in file.txt\n\
             Automatic merge failed; fix conflicts and then commit the result.\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_merge_get_new_command() {
        let rule = GitMerge;
        let cmd = Command::new(
            "git merge feature",
            "CONFLICT (content): Merge conflict in file.txt\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(new_commands.contains(&"git merge --abort".to_string()));
    }

    #[test]
    fn test_git_merge_unrelated_matches() {
        let rule = GitMergeUnrelated;
        let cmd = Command::new(
            "git merge origin/main",
            "fatal: refusing to merge unrelated histories\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_merge_unrelated_get_new_command() {
        let rule = GitMergeUnrelated;
        let cmd = Command::new(
            "git merge origin/main",
            "fatal: refusing to merge unrelated histories\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(
            new_commands,
            vec!["git merge origin/main --allow-unrelated-histories"]
        );
    }

    #[test]
    fn test_git_rm_local_modifications_matches() {
        let rule = GitRmLocalModifications;
        let cmd = Command::new(
            "git rm file.txt",
            "error: the following file has local modifications:\n    file.txt\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_rm_local_modifications_get_new_command() {
        let rule = GitRmLocalModifications;
        let cmd = Command::new(
            "git rm file.txt",
            "error: the following file has local modifications:\n    file.txt\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git rm -f file.txt"]);
    }

    #[test]
    fn test_git_rm_recursive_matches() {
        let rule = GitRmRecursive;
        let cmd = Command::new(
            "git rm directory",
            "fatal: not removing 'directory' recursively without -r\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_rm_recursive_get_new_command() {
        let rule = GitRmRecursive;
        let cmd = Command::new(
            "git rm directory",
            "fatal: not removing 'directory' recursively without -r\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git rm -r directory"]);
    }

    #[test]
    fn test_git_tag_force_matches() {
        let rule = GitTagForce;
        let cmd = Command::new("git tag v1.0", "fatal: tag 'v1.0' already exists\n");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_tag_force_get_new_command() {
        let rule = GitTagForce;
        let cmd = Command::new("git tag v1.0", "fatal: tag 'v1.0' already exists\n");
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git tag -f v1.0"]);
    }

    #[test]
    fn test_git_clone_git_clone_matches() {
        let rule = GitCloneGitClone;
        let cmd = Command::new("git clone git clone https://github.com/user/repo.git", "");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_clone_git_clone_get_new_command() {
        let rule = GitCloneGitClone;
        let cmd = Command::new("git clone git clone https://github.com/user/repo.git", "");
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(
            new_commands,
            vec!["git clone https://github.com/user/repo.git"]
        );
    }

    #[test]
    fn test_git_hook_bypass_matches() {
        let rule = GitHookBypass;
        let cmd = Command::new(
            "git commit -m 'test'",
            "Running pre-commit hook...\npre-commit hook failed\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_hook_bypass_get_new_command() {
        let rule = GitHookBypass;
        let cmd = Command::new("git commit -m 'test'", "pre-commit hook failed\n");
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git commit -m 'test' --no-verify"]);
    }

    #[test]
    fn test_git_diff_staged_matches() {
        let rule = GitDiffStaged;
        let cmd = Command::new("git diff", "");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_diff_staged_no_match() {
        let rule = GitDiffStaged;
        let cmd = Command::new("git diff", "diff --git a/file.txt b/file.txt\n");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_diff_staged_get_new_command() {
        let rule = GitDiffStaged;
        let cmd = Command::new("git diff", "");
        let new_commands = rule.get_new_command(&cmd);
        assert!(new_commands.contains(&"git diff --staged".to_string()));
        assert!(new_commands.contains(&"git diff HEAD".to_string()));
    }

    #[test]
    fn test_git_pull_wrapped() {
        let rule = GitPull::new();

        // Should not match non-git commands
        let cmd = Command::new("svn pull", "set-upstream");
        assert!(!rule.is_match(&cmd));

        // Should match git commands
        let cmd = Command::new("git pull", "set-upstream");
        assert!(rule.is_match(&cmd));
    }

    // Tests for new rules

    #[test]
    fn test_git_bisect_usage_matches() {
        let rule = GitBisectUsage;
        let cmd = Command::new(
            "git bisect strt",
            "usage: git bisect [help|start|bad|good|new|old|terms|skip|next|reset|visualize|view|replay|log|run]\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_bisect_usage_no_match() {
        let rule = GitBisectUsage;
        let cmd = Command::new("git bisect start", "");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_bisect_usage_get_new_command() {
        let rule = GitBisectUsage;
        let cmd = Command::new(
            "git bisect strt",
            "usage: git bisect [help|start|bad|good|new|old|terms|skip|next|reset|visualize|view|replay|log|run]\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("start"));
    }

    #[test]
    fn test_git_clone_missing_matches_https() {
        let rule = GitCloneMissing;
        let cmd = Command::new(
            "https://github.com/nvbn/thefuck.git",
            "https://github.com/nvbn/thefuck.git: No such file or directory",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_clone_missing_matches_ssh() {
        let rule = GitCloneMissing;
        let cmd = Command::new(
            "git@github.com:nvbn/thefuck.git",
            "git@github.com:nvbn/thefuck.git: command not found",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_clone_missing_no_match() {
        let rule = GitCloneMissing;
        let cmd = Command::new("git clone https://github.com/nvbn/thefuck.git", "");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_clone_missing_get_new_command() {
        let rule = GitCloneMissing;
        let cmd = Command::new(
            "https://github.com/nvbn/thefuck.git",
            "No such file or directory",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(
            new_commands,
            vec!["git clone https://github.com/nvbn/thefuck.git"]
        );
    }

    #[test]
    fn test_git_diff_no_index_matches() {
        let rule = GitDiffNoIndex;
        let cmd = Command::new("git diff file1.txt file2.txt", "");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_diff_no_index_no_match_has_flag() {
        let rule = GitDiffNoIndex;
        let cmd = Command::new("git diff --no-index file1.txt file2.txt", "");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_diff_no_index_get_new_command() {
        let rule = GitDiffNoIndex;
        let cmd = Command::new("git diff file1.txt file2.txt", "");
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(
            new_commands,
            vec!["git diff --no-index file1.txt file2.txt"]
        );
    }

    #[test]
    fn test_git_fix_stash_matches() {
        let rule = GitFixStash;
        let cmd = Command::new(
            "git stash lst",
            "usage: git stash list [<options>]\n   or: git stash show [<options>] [<stash>]\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_fix_stash_no_match() {
        let rule = GitFixStash;
        let cmd = Command::new("git stash list", "");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_fix_stash_get_new_command() {
        let rule = GitFixStash;
        let cmd = Command::new("git stash lst", "usage: git stash list [<options>]\n");
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("list"));
    }

    #[test]
    fn test_git_fix_stash_insert_save() {
        let rule = GitFixStash;
        let cmd = Command::new(
            "git stash my-changes",
            "usage: git stash list [<options>]\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("save"));
    }

    #[test]
    fn test_git_flag_after_filename_matches() {
        let rule = GitFlagAfterFilename;
        let cmd = Command::new(
            "git log README.md -p",
            "fatal: bad flag '-p' used after filename",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_flag_after_filename_matches_option() {
        let rule = GitFlagAfterFilename;
        let cmd = Command::new(
            "git log README.md --oneline",
            "fatal: option '--oneline' must come before non-option arguments",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_flag_after_filename_get_new_command() {
        let rule = GitFlagAfterFilename;
        let cmd = Command::new(
            "git log README.md -p",
            "fatal: bad flag '-p' used after filename",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert_eq!(new_commands[0], "git log -p README.md");
    }

    #[test]
    fn test_git_help_aliased_matches() {
        let rule = GitHelpAliased;
        let cmd = Command::new("git help ci", "`ci` is aliased to `commit`");
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_help_aliased_no_match() {
        let rule = GitHelpAliased;
        let cmd = Command::new(
            "git help commit",
            "NAME\n       git-commit - Record changes to the repository",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_help_aliased_get_new_command() {
        let rule = GitHelpAliased;
        let cmd = Command::new("git help ci", "`ci` is aliased to `commit`");
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git help commit"]);
    }

    #[test]
    fn test_git_lfs_mistype_matches() {
        let rule = GitLfsMistype;
        let cmd = Command::new(
            "git lfs trak",
            "Error: unknown command \"trak\" for \"git-lfs\"\n\nDid you mean this?\n\ttrack\n\nRun 'git lfs --help' for usage.",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_lfs_mistype_no_match() {
        let rule = GitLfsMistype;
        let cmd = Command::new("git lfs track", "");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_lfs_mistype_get_new_command() {
        let rule = GitLfsMistype;
        let cmd = Command::new(
            "git lfs trak",
            "Error: unknown command \"trak\" for \"git-lfs\"\n\nDid you mean this?\n\ttrack\n\nRun 'git lfs --help' for usage.",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands[0].contains("track"));
    }

    #[test]
    fn test_git_rebase_merge_dir_matches() {
        let rule = GitRebaseMergeDir;
        let cmd = Command::new(
            "git rebase master",
            "It seems that there is already a rebase-merge directory, and\n\
             I wonder if you are in the middle of another rebase.  If that is the\n\
             case, please try\n\tgit rebase (--continue | --abort | --skip)\n\
             If that is not the case, please\n\trm -fr \".git/rebase-merge\"\n\
             and run me again.  I am stopping in case you still have something\n\
             valuable there.",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_rebase_merge_dir_no_match() {
        let rule = GitRebaseMergeDir;
        let cmd = Command::new(
            "git rebase master",
            "Successfully rebased and updated refs/heads/feature.\n",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_rebase_merge_dir_get_new_command() {
        let rule = GitRebaseMergeDir;
        let cmd = Command::new(
            "git rebase master",
            "It seems that there is already a rebase-merge directory, and\n\
             I wonder if you are in the middle of another rebase.  If that is the\n\
             case, please try\n\tgit rebase (--continue | --abort | --skip)\n\
             If that is not the case, please\n\trm -fr \".git/rebase-merge\"\n\
             and run me again.",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert!(!new_commands.is_empty());
        assert!(new_commands.iter().any(|c| c.contains("--continue")));
        assert!(new_commands.iter().any(|c| c.contains("--abort")));
        assert!(new_commands.iter().any(|c| c.contains("--skip")));
    }

    #[test]
    fn test_git_remote_seturl_add_matches() {
        let rule = GitRemoteSeturlAdd;
        let cmd = Command::new(
            "git remote set-url origin https://github.com/user/repo.git",
            "fatal: No such remote 'origin'\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_remote_seturl_add_no_match() {
        let rule = GitRemoteSeturlAdd;
        let cmd = Command::new(
            "git remote set-url origin https://github.com/user/repo.git",
            "",
        );
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_remote_seturl_add_get_new_command() {
        let rule = GitRemoteSeturlAdd;
        let cmd = Command::new(
            "git remote set-url origin https://github.com/user/repo.git",
            "fatal: No such remote 'origin'\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(
            new_commands,
            vec!["git remote add origin https://github.com/user/repo.git"]
        );
    }

    #[test]
    fn test_git_rm_staged_matches() {
        let rule = GitRmStaged;
        let cmd = Command::new(
            "git rm file.txt",
            "error: the following file has changes staged in the index:\n    file.txt\n\
             (use --cached to keep the file, or -f to force removal)\n",
        );
        assert!(rule.is_match(&cmd));
    }

    #[test]
    fn test_git_rm_staged_no_match() {
        let rule = GitRmStaged;
        let cmd = Command::new("git rm file.txt", "rm 'file.txt'\n");
        assert!(!rule.is_match(&cmd));
    }

    #[test]
    fn test_git_rm_staged_get_new_command() {
        let rule = GitRmStaged;
        let cmd = Command::new(
            "git rm file.txt",
            "error: the following file has changes staged in the index:\n    file.txt\n\
             (use --cached to keep the file, or -f to force removal)\n",
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands.len(), 2);
        assert!(new_commands[0].contains("--cached"));
        assert!(new_commands[1].contains("-f"));
    }
}
