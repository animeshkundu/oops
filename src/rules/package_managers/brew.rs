//! Homebrew package manager rules (macOS/Linux).
//!
//! Contains rules for:
//! - `brew_install` - Suggest similar formula when "No available formula" error
//! - `brew_update` - Suggest brew update when encountering "No such file or directory"
//! - `brew_update_formula` - Suggest brew upgrade when brew update is used with formula
//! - `brew_cask_dependency` - Handle cask dependency errors
//! - `brew_link` - Fix brew link issues
//! - `brew_reinstall` - Suggest reinstall when install fails for already installed formula
//! - `brew_uninstall` - Fix uninstall errors with --force flag
//! - `brew_unknown_command` - Fix typos in brew commands

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;

/// Common brew commands for fuzzy matching.
const BREW_COMMANDS: &[&str] = &[
    "info",
    "home",
    "options",
    "install",
    "uninstall",
    "search",
    "list",
    "update",
    "upgrade",
    "pin",
    "unpin",
    "doctor",
    "create",
    "edit",
    "cask",
    "tap",
    "untap",
    "link",
    "unlink",
    "reinstall",
    "outdated",
    "deps",
    "uses",
    "leaves",
    "cleanup",
    "services",
    "bundle",
    "analytics",
    "autoremove",
    "fetch",
    "formulae",
    "casks",
    "commands",
    "config",
    "desc",
    "generate-cask-api",
    "generate-formula-api",
    "generate-man-completions",
    "gist-logs",
    "homepage",
    "info",
    "irb",
    "leaves",
    "ln",
    "log",
    "migrate",
    "missing",
    "pr-automerge",
    "pr-publish",
    "pr-pull",
    "pr-upload",
    "prof",
    "readall",
    "reinstall",
    "ruby",
    "sh",
    "shellenv",
    "style",
    "tap-info",
    "tap-new",
    "tc",
    "test",
    "tests",
    "typecheck",
    "unbottled",
    "uninstall",
    "unlink",
    "unpack",
    "untap",
    "update-license-data",
    "update-maintainers",
    "update-python-resources",
    "update-sponsors",
    "upgrade",
    "vendor-gems",
    "which-formula",
];

/// Rule to suggest similar formula names when brew install fails.
///
/// Matches errors like:
/// - `Warning: No available formula with the name "foo". Did you mean bar, baz?`
///
/// Suggests installing the similar formula names.
#[derive(Debug, Clone, Copy, Default)]
pub struct BrewInstall;

impl BrewInstall {
    /// Parse suggestions from the "Did you mean" part of the error message.
    fn get_suggestions(output: &str) -> Vec<String> {
        // Look for the pattern: Did you mean <suggestions>?
        let re = Regex::new(
            r#"Warning: No available formula with the name "[^"]+"\. Did you mean (.+)\?"#,
        );

        if let Ok(re) = re {
            if let Some(caps) = re.captures(output) {
                if let Some(suggestions_match) = caps.get(1) {
                    let suggestions_str = suggestions_match.as_str();
                    // Parse "foo, bar or baz" or "foo or bar" format
                    return suggestions_str
                        .replace(" or ", ", ")
                        .split(", ")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }

        Vec::new()
    }
}

impl Rule for BrewInstall {
    fn name(&self) -> &str {
        "brew_install"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["brew"]) {
            return false;
        }

        let parts = command.script_parts();
        // Need at least "brew install <something>"
        if parts.len() < 2 {
            return false;
        }

        // Check if it's an install command
        let has_install = parts.iter().any(|p| p == "install");
        if !has_install {
            return false;
        }

        // Check for the specific error pattern
        command.output.contains("No available formula") && command.output.contains("Did you mean")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let suggestions = Self::get_suggestions(&command.output);

        suggestions
            .into_iter()
            .map(|formula| format!("brew install {}", formula))
            .collect()
    }
}

/// Rule to suggest brew update when encountering file errors.
///
/// When Homebrew's local cache is out of date, you might get errors like
/// "Error: No such file or directory" for formulas that have been renamed
/// or moved.
///
/// This rule suggests running `brew update` first.
#[derive(Debug, Clone, Copy, Default)]
pub struct BrewUpdate;

impl Rule for BrewUpdate {
    fn name(&self) -> &str {
        "brew_update"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["brew"]) {
            return false;
        }

        // Match "Error: No such file or directory" errors
        // This often happens when Homebrew needs to be updated
        (command.output.contains("Error: No such file or directory")
            || command.output.contains("Error: No formulae or casks found"))
            && !command.script.contains("update")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Suggest updating first, then retrying the original command
        vec![format!("brew update && {}", command.script)]
    }

    fn priority(&self) -> i32 {
        // Lower priority since this is a more general fix
        1100
    }
}

/// Rule to suggest brew upgrade instead of brew update for formula updates.
///
/// When you run `brew update <formula>`, you actually want `brew upgrade <formula>`.
/// `brew update` without arguments updates Homebrew itself.
#[derive(Debug, Clone, Copy, Default)]
pub struct BrewUpdateFormula;

impl Rule for BrewUpdateFormula {
    fn name(&self) -> &str {
        "brew_update_formula"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["brew"]) {
            return false;
        }

        // Check if it's "brew update <something>"
        let parts = command.script_parts();
        if parts.len() < 3 {
            return false;
        }

        parts.get(1).map(|s| s.as_str()) == Some("update")
            && command.output.contains("This command updates brew itself")
            && command.output.contains("Use `brew upgrade")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        vec![command.script.replace("update", "upgrade")]
    }
}

/// Rule to handle cask dependency errors.
///
/// When installing a formula that requires a cask dependency, brew will suggest
/// installing the cask first.
///
/// Matches output containing:
/// - `brew cask install <dependency>`
///
/// # Example
///
/// ```text
/// $ brew install foo
/// Error: foo requires bar. You can install it with:
///   brew cask install bar
///
/// $ fuck
/// brew cask install bar && brew install foo
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct BrewCaskDependency;

impl BrewCaskDependency {
    /// Extract cask install commands from the output.
    fn get_cask_install_lines(output: &str) -> Vec<String> {
        output
            .lines()
            .map(|line| line.trim())
            .filter(|line| line.starts_with("brew cask install"))
            .map(|line| line.to_string())
            .collect()
    }
}

impl Rule for BrewCaskDependency {
    fn name(&self) -> &str {
        "brew_cask_dependency"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["brew"]) {
            return false;
        }

        let parts = command.script_parts();
        let has_install = parts.iter().any(|p| p == "install");

        has_install && command.output.contains("brew cask install")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let cask_lines = Self::get_cask_install_lines(&command.output);

        if cask_lines.is_empty() {
            return vec![];
        }

        // Join all cask install commands, then run the original command
        let cask_script = cask_lines.join(" && ");
        vec![format!("{} && {}", cask_script, command.script)]
    }
}

/// Rule to fix brew link issues.
///
/// When brew link fails due to existing files, it suggests using --overwrite --dry-run
/// to preview the changes.
///
/// Matches output containing:
/// - `brew link --overwrite --dry-run`
///
/// # Example
///
/// ```text
/// $ brew link vim
/// Error: Could not symlink bin/vim
/// Target /usr/local/bin/vim already exists. You may want to remove it:
///   rm '/usr/local/bin/vim'
///
/// To force the link and overwrite all conflicting files:
///   brew link --overwrite --dry-run vim
///
/// $ fuck
/// brew link --overwrite --dry-run vim
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct BrewLink;

impl Rule for BrewLink {
    fn name(&self) -> &str {
        "brew_link"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["brew"]) {
            return false;
        }

        let parts = command.script_parts();
        if parts.len() < 2 {
            return false;
        }

        let is_link = parts
            .get(1)
            .map(|s| s == "ln" || s == "link")
            .unwrap_or(false);

        is_link && command.output.contains("brew link --overwrite --dry-run")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let mut parts = command.script_parts().to_vec();

        if parts.len() < 2 {
            return vec![];
        }

        // Replace ln/link with "link" and add flags
        parts[1] = "link".to_string();

        // Insert --overwrite and --dry-run after "link"
        parts.insert(2, "--overwrite".to_string());
        parts.insert(3, "--dry-run".to_string());

        vec![parts.join(" ")]
    }
}

/// Rule to suggest reinstall when install fails for already installed formula.
///
/// When trying to install a formula that is already installed and up-to-date,
/// brew suggests using `brew reinstall`.
///
/// Matches output containing:
/// - `Warning: <formula> is already installed and up-to-date`
/// - `To reinstall..., run `brew reinstall`
///
/// # Example
///
/// ```text
/// $ brew install vim
/// Warning: vim 8.2.0 is already installed and up-to-date
/// To reinstall 8.2.0, run `brew reinstall vim`
///
/// $ fuck
/// brew reinstall vim
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct BrewReinstall;

impl BrewReinstall {
    /// Check if the output indicates the formula is already installed.
    fn is_already_installed(output: &str) -> bool {
        let warning_re = Regex::new(r"Warning: .+ is already installed and up-to-date").ok();
        let message_re = Regex::new(r"To reinstall .+, run `brew reinstall").ok();

        let has_warning = warning_re.map(|re| re.is_match(output)).unwrap_or(false);
        let has_message = message_re.map(|re| re.is_match(output)).unwrap_or(false);

        has_warning && has_message
    }
}

impl Rule for BrewReinstall {
    fn name(&self) -> &str {
        "brew_reinstall"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["brew"]) {
            return false;
        }

        command.script.contains("install") && Self::is_already_installed(&command.output)
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        vec![command.script.replace("install", "reinstall")]
    }
}

/// Rule to fix brew uninstall errors.
///
/// When uninstalling a formula with multiple versions, brew requires --force.
///
/// Matches output containing:
/// - `brew uninstall --force`
///
/// # Example
///
/// ```text
/// $ brew uninstall vim
/// Error: Refusing to uninstall vim
/// because it is required by ...
/// You can override this and force removal with:
///   brew uninstall --force vim
///
/// $ fuck
/// brew uninstall --force vim
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct BrewUninstall;

impl Rule for BrewUninstall {
    fn name(&self) -> &str {
        "brew_uninstall"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["brew"]) {
            return false;
        }

        let parts = command.script_parts();
        if parts.len() < 2 {
            return false;
        }

        let is_uninstall = parts
            .get(1)
            .map(|s| s == "uninstall" || s == "rm" || s == "remove")
            .unwrap_or(false);

        is_uninstall && command.output.contains("brew uninstall --force")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let mut parts = command.script_parts().to_vec();

        if parts.len() < 2 {
            return vec![];
        }

        // Normalize the command to "uninstall"
        parts[1] = "uninstall".to_string();

        // Insert --force after "uninstall"
        parts.insert(2, "--force".to_string());

        vec![parts.join(" ")]
    }
}

/// Rule to fix typos in brew commands.
///
/// When an unknown brew command is used, this rule suggests the closest valid command.
///
/// Matches output containing:
/// - `Error: Unknown command: <cmd>`
///
/// # Example
///
/// ```text
/// $ brew instal vim
/// Error: Unknown command: instal
///
/// $ fuck
/// brew install vim
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct BrewUnknownCommand;

impl BrewUnknownCommand {
    /// Extract the unknown command from the error output.
    fn get_unknown_command(output: &str) -> Option<String> {
        let re = Regex::new(r"Error: Unknown command: ([a-z-]+)").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Get brew commands as owned strings.
    fn get_commands() -> Vec<String> {
        BREW_COMMANDS.iter().map(|s| s.to_string()).collect()
    }
}

impl Rule for BrewUnknownCommand {
    fn name(&self) -> &str {
        "brew_unknown_command"
    }

    fn is_match(&self, command: &Command) -> bool {
        if !is_app(command, &["brew"]) {
            return false;
        }

        if !command.output.contains("Unknown command") {
            return false;
        }

        // Check that we can find a close match
        if let Some(unknown_cmd) = Self::get_unknown_command(&command.output) {
            let commands = Self::get_commands();
            let matches = get_close_matches(&unknown_cmd, &commands, 1, 0.6);
            return !matches.is_empty();
        }

        false
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let unknown_cmd = match Self::get_unknown_command(&command.output) {
            Some(cmd) => cmd,
            None => return vec![],
        };

        let commands = Self::get_commands();
        let suggestions = get_close_matches(&unknown_cmd, &commands, 3, 0.6);

        suggestions
            .into_iter()
            .map(|cmd| replace_argument(&command.script, &unknown_cmd, &cmd))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod brew_install_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(BrewInstall.name(), "brew_install");
        }

        #[test]
        fn test_matches_no_available_formula() {
            let cmd = Command::new(
                "brew install vim-foo",
                r#"Warning: No available formula with the name "vim-foo". Did you mean vim, neovim or macvim?"#,
            );
            assert!(BrewInstall.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful_install() {
            let cmd = Command::new("brew install vim", "==> Downloading vim...");
            assert!(!BrewInstall.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("apt install vim", "No available formula");
            assert!(!BrewInstall.is_match(&cmd));
        }

        #[test]
        fn test_no_match_without_install() {
            let cmd = Command::new("brew search vim", "No available formula");
            assert!(!BrewInstall.is_match(&cmd));
        }

        #[test]
        fn test_get_suggestions_multiple() {
            let output = r#"Warning: No available formula with the name "vim-foo". Did you mean vim, neovim or macvim?"#;
            let suggestions = BrewInstall::get_suggestions(output);
            assert_eq!(suggestions, vec!["vim", "neovim", "macvim"]);
        }

        #[test]
        fn test_get_suggestions_single_or() {
            let output =
                r#"Warning: No available formula with the name "htps". Did you mean htop or http?"#;
            let suggestions = BrewInstall::get_suggestions(output);
            assert_eq!(suggestions, vec!["htop", "http"]);
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "brew install vim-foo",
                r#"Warning: No available formula with the name "vim-foo". Did you mean vim, neovim or macvim?"#,
            );
            let fixes = BrewInstall.get_new_command(&cmd);
            assert_eq!(
                fixes,
                vec![
                    "brew install vim",
                    "brew install neovim",
                    "brew install macvim"
                ]
            );
        }

        #[test]
        fn test_get_new_command_empty_on_no_suggestions() {
            let cmd = Command::new(
                "brew install xyz",
                "Error: No available formula with the name xyz",
            );
            let fixes = BrewInstall.get_new_command(&cmd);
            assert!(fixes.is_empty());
        }
    }

    mod brew_update_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(BrewUpdate.name(), "brew_update");
        }

        #[test]
        fn test_matches_no_such_file() {
            let cmd = Command::new(
                "brew install some-old-formula",
                "Error: No such file or directory @ rb_sysopen - /usr/local/Homebrew/Library/...",
            );
            assert!(BrewUpdate.is_match(&cmd));
        }

        #[test]
        fn test_matches_no_formulae_found() {
            let cmd = Command::new(
                "brew install oldpackage",
                "Error: No formulae or casks found for oldpackage.",
            );
            assert!(BrewUpdate.is_match(&cmd));
        }

        #[test]
        fn test_no_match_if_already_updating() {
            let cmd = Command::new("brew update", "Error: No such file or directory");
            assert!(!BrewUpdate.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("npm install", "Error: No such file or directory");
            assert!(!BrewUpdate.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "brew install old-formula",
                "Error: No such file or directory",
            );
            let fixes = BrewUpdate.get_new_command(&cmd);
            assert_eq!(fixes, vec!["brew update && brew install old-formula"]);
        }

        #[test]
        fn test_priority() {
            assert_eq!(BrewUpdate.priority(), 1100);
        }
    }

    mod brew_update_formula_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(BrewUpdateFormula.name(), "brew_update_formula");
        }

        #[test]
        fn test_matches_update_formula() {
            let cmd = Command::new(
                "brew update vim",
                "Error: This command updates brew itself, and does not take formula names.\nUse `brew upgrade vim` instead.",
            );
            assert!(BrewUpdateFormula.is_match(&cmd));
        }

        #[test]
        fn test_no_match_just_update() {
            let cmd = Command::new("brew update", "Updated Homebrew");
            assert!(!BrewUpdateFormula.is_match(&cmd));
        }

        #[test]
        fn test_no_match_upgrade() {
            let cmd = Command::new("brew upgrade vim", "Upgrading vim...");
            assert!(!BrewUpdateFormula.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "brew update vim",
                "Error: This command updates brew itself\nUse `brew upgrade vim` instead.",
            );
            let fixes = BrewUpdateFormula.get_new_command(&cmd);
            assert_eq!(fixes, vec!["brew upgrade vim"]);
        }
    }

    mod brew_cask_dependency_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(BrewCaskDependency.name(), "brew_cask_dependency");
        }

        #[test]
        fn test_matches_cask_dependency() {
            let cmd = Command::new(
                "brew install foo",
                "Error: foo requires bar. You can install it with:\n  brew cask install bar",
            );
            assert!(BrewCaskDependency.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let cmd = Command::new("brew install vim", "==> Downloading vim...");
            assert!(!BrewCaskDependency.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("apt install vim", "brew cask install");
            assert!(!BrewCaskDependency.is_match(&cmd));
        }

        #[test]
        fn test_get_cask_install_lines() {
            let output =
                "Error: foo requires bar.\n  brew cask install bar\n  brew cask install baz";
            let lines = BrewCaskDependency::get_cask_install_lines(output);
            assert_eq!(
                lines,
                vec!["brew cask install bar", "brew cask install baz"]
            );
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "brew install foo",
                "Error: foo requires bar.\n  brew cask install bar",
            );
            let fixes = BrewCaskDependency.get_new_command(&cmd);
            assert_eq!(fixes, vec!["brew cask install bar && brew install foo"]);
        }

        #[test]
        fn test_get_new_command_multiple() {
            let cmd = Command::new(
                "brew install foo",
                "Error:\n  brew cask install bar\n  brew cask install baz",
            );
            let fixes = BrewCaskDependency.get_new_command(&cmd);
            assert_eq!(
                fixes,
                vec!["brew cask install bar && brew cask install baz && brew install foo"]
            );
        }
    }

    mod brew_link_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(BrewLink.name(), "brew_link");
        }

        #[test]
        fn test_matches_link_error() {
            let cmd = Command::new(
                "brew link vim",
                "Error: Could not symlink\n  brew link --overwrite --dry-run vim",
            );
            assert!(BrewLink.is_match(&cmd));
        }

        #[test]
        fn test_matches_ln() {
            let cmd = Command::new(
                "brew ln vim",
                "Error: Could not symlink\n  brew link --overwrite --dry-run vim",
            );
            assert!(BrewLink.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let cmd = Command::new("brew link vim", "Linking vim...");
            assert!(!BrewLink.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "brew link vim",
                "Error:\n  brew link --overwrite --dry-run vim",
            );
            let fixes = BrewLink.get_new_command(&cmd);
            assert_eq!(fixes, vec!["brew link --overwrite --dry-run vim"]);
        }

        #[test]
        fn test_get_new_command_ln() {
            let cmd = Command::new(
                "brew ln vim",
                "Error:\n  brew link --overwrite --dry-run vim",
            );
            let fixes = BrewLink.get_new_command(&cmd);
            assert_eq!(fixes, vec!["brew link --overwrite --dry-run vim"]);
        }
    }

    mod brew_reinstall_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(BrewReinstall.name(), "brew_reinstall");
        }

        #[test]
        fn test_matches_already_installed() {
            let cmd = Command::new(
                "brew install vim",
                "Warning: vim 8.2.0 is already installed and up-to-date\nTo reinstall 8.2.0, run `brew reinstall vim`",
            );
            assert!(BrewReinstall.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let cmd = Command::new("brew install vim", "==> Downloading vim...");
            assert!(!BrewReinstall.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new(
                "apt install vim",
                "Warning: vim is already installed and up-to-date",
            );
            assert!(!BrewReinstall.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new(
                "brew install vim",
                "Warning: vim 8.2.0 is already installed and up-to-date\nTo reinstall 8.2.0, run `brew reinstall vim`",
            );
            let fixes = BrewReinstall.get_new_command(&cmd);
            assert_eq!(fixes, vec!["brew reinstall vim"]);
        }
    }

    mod brew_uninstall_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(BrewUninstall.name(), "brew_uninstall");
        }

        #[test]
        fn test_matches_uninstall_error() {
            let cmd = Command::new(
                "brew uninstall vim",
                "Error: Refusing to uninstall\n  brew uninstall --force vim",
            );
            assert!(BrewUninstall.is_match(&cmd));
        }

        #[test]
        fn test_matches_rm() {
            let cmd = Command::new("brew rm vim", "Error:\n  brew uninstall --force vim");
            assert!(BrewUninstall.is_match(&cmd));
        }

        #[test]
        fn test_matches_remove() {
            let cmd = Command::new("brew remove vim", "Error:\n  brew uninstall --force vim");
            assert!(BrewUninstall.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let cmd = Command::new("brew uninstall vim", "Uninstalling vim...");
            assert!(!BrewUninstall.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("brew uninstall vim", "Error:\n  brew uninstall --force vim");
            let fixes = BrewUninstall.get_new_command(&cmd);
            assert_eq!(fixes, vec!["brew uninstall --force vim"]);
        }

        #[test]
        fn test_get_new_command_rm() {
            let cmd = Command::new("brew rm vim", "Error:\n  brew uninstall --force vim");
            let fixes = BrewUninstall.get_new_command(&cmd);
            assert_eq!(fixes, vec!["brew uninstall --force vim"]);
        }
    }

    mod brew_unknown_command_tests {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(BrewUnknownCommand.name(), "brew_unknown_command");
        }

        #[test]
        fn test_matches_unknown_command() {
            let cmd = Command::new("brew instal vim", "Error: Unknown command: instal");
            assert!(BrewUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let cmd = Command::new("brew install vim", "==> Downloading vim...");
            assert!(!BrewUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("apt instal vim", "Error: Unknown command: instal");
            assert!(!BrewUnknownCommand.is_match(&cmd));
        }

        #[test]
        fn test_get_unknown_command() {
            let output = "Error: Unknown command: instal";
            let unknown = BrewUnknownCommand::get_unknown_command(output);
            assert_eq!(unknown, Some("instal".to_string()));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("brew instal vim", "Error: Unknown command: instal");
            let fixes = BrewUnknownCommand.get_new_command(&cmd);
            assert!(fixes.contains(&"brew install vim".to_string()));
        }

        #[test]
        fn test_get_new_command_upgade() {
            let cmd = Command::new("brew upgade vim", "Error: Unknown command: upgade");
            let fixes = BrewUnknownCommand.get_new_command(&cmd);
            assert!(fixes.contains(&"brew upgrade vim".to_string()));
        }
    }
}
