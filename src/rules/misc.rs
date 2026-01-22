//! Miscellaneous correction rules.
//!
//! This module contains various correction rules that don't fit neatly into other categories:
//!
//! - [`RemoveShellPromptLiteral`] - Removes `$` or `#` from copied commands
//! - [`Unsudo`] - Removes unnecessary sudo
//! - [`FixAltSpace`] - Fixes alt+space character issues (non-breaking space)
//! - [`MissingSpaceBeforeSubcommand`] - Adds missing spaces in commands
//! - [`NoSuchFile`] - Handles "No such file" errors for mv/cp
//! - [`PathFromHistory`] - Suggests paths from command history
//! - [`QuotationMarks`] - Fixes mismatched quotation marks
//! - [`RemoveTrailingCedilla`] - Removes trailing special characters
//! - [`SudoCommandFromUserPath`] - Uses full path with sudo
//! - [`WrongHyphenBeforeSubcommand`] - Fixes hyphen typos in commands
//! - [`AptUpgrade`] - Suggests apt upgrade after listing upgradable packages
//! - [`FixFile`] - Opens editor at error location

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_all_executables, replace_argument, which};
use regex::Regex;
use std::env;
use std::path::Path;

// ============================================================================
// RemoveShellPromptLiteral
// ============================================================================

/// Rule that removes shell prompt literals (`$` or `#`) from copied commands.
///
/// This usually happens when commands are copied from documentation that includes
/// the shell prompt in code blocks.
///
/// # Example
///
/// ```
/// use oops::rules::misc::RemoveShellPromptLiteral;
/// use oops::core::{Command, Rule};
///
/// let rule = RemoveShellPromptLiteral;
/// let cmd = Command::new("$ git clone https://github.com/nvbn/thefuck.git", "$: command not found");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct RemoveShellPromptLiteral;

impl RemoveShellPromptLiteral {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for RemoveShellPromptLiteral {
    fn name(&self) -> &str {
        "remove_shell_prompt_literal"
    }

    fn priority(&self) -> i32 {
        900
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Check for "$: command not found" in output
        if !cmd.output.contains("$: command not found") {
            return false;
        }

        // Check if script starts with $ or # followed by a space and a command
        let re = Regex::new(r"^[\s]*\$ [\S]+").unwrap();
        re.is_match(&cmd.script)
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Remove leading whitespace and "$ " from the command
        let trimmed = cmd.script.trim_start();
        if let Some(rest) = trimmed.strip_prefix("$ ") {
            vec![rest.to_string()]
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            vec![rest.to_string()]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Unsudo
// ============================================================================

/// Rule that removes unnecessary sudo.
///
/// Some operations cannot be performed as root, such as certain package manager
/// operations that require a normal user context.
///
/// # Example
///
/// ```
/// use oops::rules::misc::Unsudo;
/// use oops::core::{Command, Rule};
///
/// let rule = Unsudo;
/// let cmd = Command::new("sudo npm install", "you cannot perform this operation as root");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Unsudo;

impl Unsudo {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for Unsudo {
    fn name(&self) -> &str {
        "unsudo"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        if parts.is_empty() || parts[0] != "sudo" {
            return false;
        }

        let output_lower = cmd.output.to_lowercase();
        output_lower.contains("you cannot perform this operation as root")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.len() > 1 {
            vec![parts[1..].join(" ")]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// FixAltSpace
// ============================================================================

/// Rule that fixes alt+space character issues (non-breaking space).
///
/// On some keyboards, pressing Alt+Space inserts a non-breaking space character
/// instead of a regular space, which can cause "command not found" errors.
///
/// # Example
///
/// ```
/// use oops::rules::misc::FixAltSpace;
/// use oops::core::{Command, Rule};
///
/// let rule = FixAltSpace;
/// let cmd = Command::new("git\u{00A0}status", "command not found"); // non-breaking space
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct FixAltSpace;

impl FixAltSpace {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for FixAltSpace {
    fn name(&self) -> &str {
        "fix_alt_space"
    }

    fn priority(&self) -> i32 {
        900
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Check for command not found error
        let output_lower = cmd.output.to_lowercase();
        if !output_lower.contains("command not found") {
            return false;
        }

        // Check if script contains non-breaking space (U+00A0)
        cmd.script.contains('\u{00A0}')
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace non-breaking spaces with regular spaces
        vec![cmd.script.replace('\u{00A0}', " ")]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// MissingSpaceBeforeSubcommand
// ============================================================================

/// Rule that adds missing spaces before subcommands.
///
/// Handles cases like "gitclone" -> "git clone" where the user forgot to add
/// a space between the command and subcommand.
///
/// # Example
///
/// ```
/// use oops::rules::misc::MissingSpaceBeforeSubcommand;
/// use oops::core::{Command, Rule};
///
/// let rule = MissingSpaceBeforeSubcommand;
/// let cmd = Command::new("gitclone https://...", "gitclone: command not found");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MissingSpaceBeforeSubcommand;

impl MissingSpaceBeforeSubcommand {
    pub fn new() -> Self {
        Self
    }

    /// Try to find an executable that is a prefix of the given script part
    fn find_executable_prefix(script_part: &str) -> Option<String> {
        let executables = get_all_executables();

        for executable in executables.iter() {
            if executable.len() > 1
                && script_part.starts_with(executable.as_str())
                && script_part.len() > executable.len()
            {
                return Some(executable.clone());
            }
        }
        None
    }
}

impl Rule for MissingSpaceBeforeSubcommand {
    fn name(&self) -> &str {
        "missing_space_before_subcommand"
    }

    fn priority(&self) -> i32 {
        4000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return false;
        }

        let first_part = &parts[0];
        let executables = get_all_executables();

        // Check if the first part is NOT an executable
        if executables.contains(first_part) {
            return false;
        }

        // Check if the first part starts with a known executable
        Self::find_executable_prefix(first_part).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return vec![];
        }

        if let Some(executable) = Self::find_executable_prefix(&parts[0]) {
            // Insert a space after the executable
            let fixed = cmd
                .script
                .replacen(&executable, &format!("{} ", executable), 1);
            vec![fixed]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// ============================================================================
// NoSuchFile
// ============================================================================

/// Rule that handles "No such file or directory" errors for mv/cp commands.
///
/// When mv or cp fails because the destination directory doesn't exist,
/// this rule suggests creating the directory first.
///
/// # Example
///
/// ```no_run
/// use oops::rules::misc::NoSuchFile;
/// use oops::core::{Command, Rule};
///
/// let rule = NoSuchFile;
/// let cmd = Command::new("mv file.txt /path/to/dest/", "cannot stat 'file.txt': No such file or directory");
/// // May match depending on the error output pattern
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoSuchFile;

impl NoSuchFile {
    pub fn new() -> Self {
        Self
    }

    /// Extract destination path from mv/cp error output
    fn extract_destination(output: &str) -> Option<String> {
        let patterns = [
            r"mv: cannot move '[^']*' to '([^']*)': No such file or directory",
            r"mv: cannot move '[^']*' to '([^']*)': Not a directory",
            r"cp: cannot create regular file '([^']*)': No such file or directory",
            r"cp: cannot create regular file '([^']*)': Not a directory",
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(output) {
                    if let Some(m) = caps.get(1) {
                        return Some(m.as_str().to_string());
                    }
                }
            }
        }
        None
    }
}

impl Rule for NoSuchFile {
    fn name(&self) -> &str {
        "no_such_file"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        Self::extract_destination(&cmd.output).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(file) = Self::extract_destination(&cmd.output) {
            // Get the directory part (everything before the last /)
            if let Some(last_slash) = file.rfind('/') {
                let dir = &file[..last_slash];
                if !dir.is_empty() {
                    // Create mkdir command followed by original command
                    return vec![format!("mkdir -p {} && {}", dir, cmd.script)];
                }
            }
        }
        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// PathFromHistory
// ============================================================================

/// Rule that suggests paths from command history.
///
/// When a command fails because a file/directory doesn't exist, this rule
/// searches the command history for similar paths that might be what the
/// user intended.
///
/// # Example
///
/// ```
/// use oops::rules::misc::PathFromHistory;
/// use oops::core::{Command, Rule};
///
/// let rule = PathFromHistory;
/// let cmd = Command::new("cd projects", "no such file or directory: projects");
/// // Would suggest full paths from history that end with "projects"
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PathFromHistory;

impl PathFromHistory {
    pub fn new() -> Self {
        Self
    }

    /// Extract the destination/path that wasn't found from error output
    fn get_destination(cmd: &Command) -> Option<String> {
        let patterns = [
            r"no such file or directory: (.*)$",
            r"cannot access '(.*)': No such file or directory",
            r": (.*): No such file or directory",
            r"can't cd to (.*)$",
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                if let Some(caps) = re.captures(&cmd.output) {
                    if let Some(m) = caps.get(1) {
                        let found = m.as_str().trim();
                        // Check if this is part of the command
                        let parts = cmd.script_parts();
                        if parts.iter().any(|p| p == found) {
                            return Some(found.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Get command history from environment
    fn get_history() -> Vec<String> {
        // Try TF_HISTORY first, then THEFUCK_HISTORY
        let history_str = env::var("TF_HISTORY")
            .or_else(|_| env::var("THEFUCK_HISTORY"))
            .unwrap_or_default();

        history_str.lines().map(|s| s.to_string()).collect()
    }

    /// Extract absolute paths from history
    fn get_absolute_paths_from_history(current_cmd: &str) -> Vec<String> {
        let history = Self::get_history();
        let mut paths: Vec<String> = Vec::new();

        for line in history {
            // Skip the current command
            if line == current_cmd {
                continue;
            }

            // Split command into parts
            if let Some(parts) = shlex::split(&line) {
                for param in parts.iter().skip(1) {
                    if param.starts_with('/') || param.starts_with('~') {
                        let mut path = param.clone();
                        if path.ends_with('/') {
                            path.pop();
                        }
                        if !paths.contains(&path) {
                            paths.push(path);
                        }
                    }
                }
            }
        }

        paths
    }
}

impl Rule for PathFromHistory {
    fn name(&self) -> &str {
        "path_from_history"
    }

    fn priority(&self) -> i32 {
        800
    }

    fn is_match(&self, cmd: &Command) -> bool {
        Self::get_destination(cmd).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let destination = match Self::get_destination(cmd) {
            Some(d) => d,
            None => return vec![],
        };

        let paths = Self::get_absolute_paths_from_history(&cmd.script);

        paths
            .into_iter()
            .filter(|path| {
                // Path must end with the destination
                if !path.ends_with(&destination) {
                    return false;
                }

                // Path must exist
                let expanded = if path.starts_with('~') {
                    if let Some(home) = dirs::home_dir() {
                        path.replacen('~', &home.to_string_lossy(), 1)
                    } else {
                        path.clone()
                    }
                } else {
                    path.clone()
                };

                Path::new(&expanded).exists()
            })
            .map(|path| replace_argument(&cmd.script, &destination, &path))
            .collect()
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// QuotationMarks
// ============================================================================

/// Rule that fixes mismatched quotation marks.
///
/// When a command contains both single and double quotes (often by mistake),
/// this rule replaces all single quotes with double quotes for consistency.
///
/// # Example
///
/// ```
/// use oops::rules::misc::QuotationMarks;
/// use oops::core::{Command, Rule};
///
/// let rule = QuotationMarks;
/// let cmd = Command::new("git commit -m 'My Message\"", "");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct QuotationMarks;

impl QuotationMarks {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for QuotationMarks {
    fn name(&self) -> &str {
        "quotation_marks"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Match if command contains both single and double quotes
        cmd.script.contains('\'') && cmd.script.contains('"')
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace all single quotes with double quotes
        vec![cmd.script.replace('\'', "\"")]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// ============================================================================
// RemoveTrailingCedilla
// ============================================================================

/// Rule that removes trailing cedilla character.
///
/// This can happen on some keyboard layouts where accidentally pressing
/// a key adds a trailing special character.
///
/// # Example
///
/// ```
/// use oops::rules::misc::RemoveTrailingCedilla;
/// use oops::core::{Command, Rule};
///
/// let rule = RemoveTrailingCedilla;
/// let cmd = Command::new("git status\u{00E7}", ""); // ends with cedilla
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct RemoveTrailingCedilla;

impl RemoveTrailingCedilla {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for RemoveTrailingCedilla {
    fn name(&self) -> &str {
        "remove_trailing_cedilla"
    }

    fn priority(&self) -> i32 {
        900
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Check if script ends with cedilla character
        cmd.script.ends_with('\u{00E7}') // 'ç' cedilla
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Remove the trailing cedilla
        let mut fixed = cmd.script.clone();
        fixed.pop();
        vec![fixed]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// ============================================================================
// SudoCommandFromUserPath
// ============================================================================

/// Rule that uses full path with sudo when command is not in sudo's PATH.
///
/// When running sudo, the PATH might be different from the user's PATH,
/// causing "command not found" errors. This rule suggests using the
/// full path to the command.
///
/// # Example
///
/// ```
/// use oops::rules::misc::SudoCommandFromUserPath;
/// use oops::core::{Command, Rule};
///
/// let rule = SudoCommandFromUserPath;
/// let cmd = Command::new("sudo my_script", "sudo: my_script: command not found");
/// // Would suggest: sudo env "PATH=$PATH" my_script
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SudoCommandFromUserPath;

impl SudoCommandFromUserPath {
    pub fn new() -> Self {
        Self
    }

    /// Extract the command name from the sudo error
    fn get_command_name(output: &str) -> Option<String> {
        let re = Regex::new(r"sudo: (.*): command not found").ok()?;
        let caps = re.captures(output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }
}

impl Rule for SudoCommandFromUserPath {
    fn name(&self) -> &str {
        "sudo_command_from_user_path"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Must be a sudo command
        if !is_app(cmd, &["sudo"]) {
            return false;
        }

        // Must have "command not found" in output
        if !cmd.output.contains("command not found") {
            return false;
        }

        // Check if the command exists in user's PATH
        if let Some(command_name) = Self::get_command_name(&cmd.output) {
            return which(command_name.to_string()).is_some();
        }

        false
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(command_name) = Self::get_command_name(&cmd.output) {
            let replacement = format!("env \"PATH=$PATH\" {}", command_name);
            vec![replace_argument(&cmd.script, &command_name, &replacement)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// WrongHyphenBeforeSubcommand
// ============================================================================

/// Rule that fixes wrong hyphen before subcommand.
///
/// Handles cases where a user typed "git-clone" instead of "git clone".
///
/// # Example
///
/// ```
/// use oops::rules::misc::WrongHyphenBeforeSubcommand;
/// use oops::core::{Command, Rule};
///
/// let rule = WrongHyphenBeforeSubcommand;
/// let cmd = Command::new("git-clone https://...", "git-clone: command not found");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct WrongHyphenBeforeSubcommand;

impl WrongHyphenBeforeSubcommand {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for WrongHyphenBeforeSubcommand {
    fn name(&self) -> &str {
        "wrong_hyphen_before_subcommand"
    }

    fn priority(&self) -> i32 {
        4500
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return false;
        }

        let first_part = &parts[0];

        // Must contain a hyphen
        if !first_part.contains('-') {
            return false;
        }

        let executables = get_all_executables();

        // If the whole thing is a valid executable, don't match
        if executables.contains(first_part) {
            return false;
        }

        // Check if the part before the first hyphen is a valid executable
        if let Some((cmd_part, _)) = first_part.split_once('-') {
            return executables.contains(cmd_part);
        }

        false
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace the first hyphen with a space
        vec![cmd.script.replacen('-', " ", 1)]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// ============================================================================
// AptUpgrade
// ============================================================================

/// Rule that suggests apt upgrade after listing upgradable packages.
///
/// When the user runs `apt list --upgradable` and there are packages
/// available for upgrade, this rule suggests running `apt upgrade`.
///
/// # Example
///
/// ```
/// use oops::rules::misc::AptUpgrade;
/// use oops::core::{Command, Rule};
///
/// let rule = AptUpgrade;
/// let cmd = Command::new(
///     "apt list --upgradable",
///     "Listing...\npackage1/stable 1.0\npackage2/stable 2.0"
/// );
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct AptUpgrade;

impl AptUpgrade {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for AptUpgrade {
    fn name(&self) -> &str {
        "apt_upgrade"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn enabled_by_default(&self) -> bool {
        // Only enable on systems where apt is available
        cfg!(target_os = "linux")
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Must be "apt list --upgradable"
        if cmd.script.trim() != "apt list --upgradable"
            && cmd.script.trim() != "sudo apt list --upgradable"
        {
            return false;
        }

        // Check if there are packages to upgrade (more than just the header line)
        let lines: Vec<&str> = cmd.output.trim().lines().collect();
        lines.len() > 1
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if cmd.script.starts_with("sudo ") {
            vec!["sudo apt upgrade".to_string()]
        } else {
            vec!["apt upgrade".to_string()]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// FixFile
// ============================================================================

/// Rule that opens an editor at the error location.
///
/// When a command outputs an error with file and line information,
/// this rule suggests opening the file in an editor at that location.
///
/// # Example
///
/// ```
/// use oops::rules::misc::FixFile;
/// use oops::core::{Command, Rule};
///
/// let rule = FixFile;
/// let cmd = Command::new("python script.py", "  File \"script.py\", line 10");
/// // Would suggest: $EDITOR script.py +10 && python script.py
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct FixFile;

impl FixFile {
    pub fn new() -> Self {
        Self
    }

    /// Search for file:line patterns in output
    fn search_error_location(output: &str) -> Option<(String, String, Option<String>)> {
        // Order is important: only the first match is considered
        let patterns = [
            // js, node:
            r"^    at ([^:\n]+):([0-9]+):([0-9]+)",
            // cargo:
            r"^   ([^:\n]+):([0-9]+):([0-9]+)",
            // python:
            r#"^  File "([^"]+)", line ([0-9]+)"#,
            // awk:
            r"^awk: ([^:\n]+):([0-9]+):",
            // git:
            r"^fatal: bad config file line ([0-9]+) in ([^:\n]+)",
            // llc:
            r"^llc: ([^:\n]+):([0-9]+):([0-9]+):",
            // lua:
            r"^lua: ([^:\n]+):([0-9]+):",
            // fish:
            r"^([^:\n]+) \(line ([0-9]+)\):",
            // bash, sh, ssh:
            r"^([^:\n]+): line ([0-9]+): ",
            // cargo, clang, gcc, go, pep8, rustc:
            r"^([^:\n]+):([0-9]+):([0-9]+)",
            // ghc, make, ruby, zsh:
            r"^([^:\n]+):([0-9]+):",
            // perl:
            r"at ([^:\n]+) line ([0-9]+)",
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(&format!("(?m){}", pattern)) {
                if let Some(caps) = re.captures(output) {
                    // Check for git's reversed pattern (line number first)
                    if pattern.contains("fatal: bad config") {
                        let line = caps.get(1).map(|m| m.as_str().to_string())?;
                        let file = caps.get(2).map(|m| m.as_str().to_string())?;

                        // Verify file exists
                        if Path::new(&file).is_file() {
                            return Some((file, line, None));
                        }
                    } else {
                        let file = caps.get(1).map(|m| m.as_str().to_string())?;
                        let line = caps.get(2).map(|m| m.as_str().to_string())?;
                        let col = caps.get(3).map(|m| m.as_str().to_string());

                        // Verify file exists
                        if Path::new(&file).is_file() {
                            return Some((file, line, col));
                        }
                    }
                }
            }
        }
        None
    }
}

impl Rule for FixFile {
    fn name(&self) -> &str {
        "fix_file_error"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Must have EDITOR environment variable set
        if env::var("EDITOR").is_err() {
            return false;
        }

        Self::search_error_location(&cmd.output).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let editor = match env::var("EDITOR") {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        if let Some((file, line, _col)) = Self::search_error_location(&cmd.output) {
            // Format: editor file +line && original_command
            let editor_call = format!("{} {} +{}", editor, file, line);
            vec![format!("{} && {}", editor_call, cmd.script)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Module exports
// ============================================================================

/// Returns all miscellaneous rules as boxed trait objects.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(RemoveShellPromptLiteral::new()),
        Box::new(Unsudo::new()),
        Box::new(FixAltSpace::new()),
        Box::new(MissingSpaceBeforeSubcommand::new()),
        Box::new(NoSuchFile::new()),
        Box::new(PathFromHistory::new()),
        Box::new(QuotationMarks::new()),
        Box::new(RemoveTrailingCedilla::new()),
        Box::new(SudoCommandFromUserPath::new()),
        Box::new(WrongHyphenBeforeSubcommand::new()),
        Box::new(AptUpgrade::new()),
        // Note: FixFile is already implemented in system.rs
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // RemoveShellPromptLiteral tests
    mod remove_shell_prompt_literal {
        use super::*;

        #[test]
        fn test_name() {
            let rule = RemoveShellPromptLiteral;
            assert_eq!(rule.name(), "remove_shell_prompt_literal");
        }

        #[test]
        fn test_matches_dollar_prompt() {
            let rule = RemoveShellPromptLiteral;
            let cmd = Command::new(
                "$ git clone https://github.com/nvbn/thefuck.git",
                "$: command not found",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_with_leading_spaces() {
            let rule = RemoveShellPromptLiteral;
            let cmd = Command::new(
                "  $ git clone https://github.com/nvbn/thefuck.git",
                "$: command not found",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_normal_command() {
            let rule = RemoveShellPromptLiteral;
            let cmd = Command::new("git clone https://github.com/nvbn/thefuck.git", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_without_error() {
            let rule = RemoveShellPromptLiteral;
            let cmd = Command::new("$ echo hello", "hello");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = RemoveShellPromptLiteral;
            let cmd = Command::new(
                "$ git clone https://github.com/nvbn/thefuck.git",
                "$: command not found",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["git clone https://github.com/nvbn/thefuck.git"]);
        }
    }

    // Unsudo tests
    mod unsudo {
        use super::*;

        #[test]
        fn test_name() {
            let rule = Unsudo;
            assert_eq!(rule.name(), "unsudo");
        }

        #[test]
        fn test_matches_cannot_as_root() {
            let rule = Unsudo;
            let cmd = Command::new(
                "sudo npm install",
                "you cannot perform this operation as root",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_without_sudo() {
            let rule = Unsudo;
            let cmd = Command::new("npm install", "you cannot perform this operation as root");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful() {
            let rule = Unsudo;
            let cmd = Command::new("sudo apt update", "Reading package lists...");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = Unsudo;
            let cmd = Command::new(
                "sudo npm install",
                "you cannot perform this operation as root",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["npm install"]);
        }

        #[test]
        fn test_get_new_command_with_args() {
            let rule = Unsudo;
            let cmd = Command::new(
                "sudo npm install -g typescript",
                "you cannot perform this operation as root",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["npm install -g typescript"]);
        }
    }

    // FixAltSpace tests
    mod fix_alt_space {
        use super::*;

        #[test]
        fn test_name() {
            let rule = FixAltSpace;
            assert_eq!(rule.name(), "fix_alt_space");
        }

        #[test]
        fn test_matches_non_breaking_space() {
            let rule = FixAltSpace;
            let cmd = Command::new("git\u{00A0}status", "git\u{00A0}status: command not found");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_normal_space() {
            let rule = FixAltSpace;
            let cmd = Command::new("git status", "command not found");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = FixAltSpace;
            let cmd = Command::new("git\u{00A0}status", "command not found");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["git status"]);
        }
    }

    // QuotationMarks tests
    mod quotation_marks {
        use super::*;

        #[test]
        fn test_name() {
            let rule = QuotationMarks;
            assert_eq!(rule.name(), "quotation_marks");
        }

        #[test]
        fn test_matches_mixed_quotes() {
            let rule = QuotationMarks;
            let cmd = Command::new("git commit -m 'My Message\"", "");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_single_quote_only() {
            let rule = QuotationMarks;
            let cmd = Command::new("git commit -m 'My Message'", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_double_quote_only() {
            let rule = QuotationMarks;
            let cmd = Command::new("git commit -m \"My Message\"", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = QuotationMarks;
            let cmd = Command::new("git commit -m 'My Message\"", "");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["git commit -m \"My Message\""]);
        }

        #[test]
        fn test_does_not_require_output() {
            let rule = QuotationMarks;
            assert!(!rule.requires_output());
        }
    }

    // RemoveTrailingCedilla tests
    mod remove_trailing_cedilla {
        use super::*;

        #[test]
        fn test_name() {
            let rule = RemoveTrailingCedilla;
            assert_eq!(rule.name(), "remove_trailing_cedilla");
        }

        #[test]
        fn test_matches_trailing_cedilla() {
            let rule = RemoveTrailingCedilla;
            let cmd = Command::new("git status\u{00E7}", "");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_normal() {
            let rule = RemoveTrailingCedilla;
            let cmd = Command::new("git status", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = RemoveTrailingCedilla;
            let cmd = Command::new("git status\u{00E7}", "");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["git status"]);
        }

        #[test]
        fn test_does_not_require_output() {
            let rule = RemoveTrailingCedilla;
            assert!(!rule.requires_output());
        }
    }

    // WrongHyphenBeforeSubcommand tests
    mod wrong_hyphen_before_subcommand {
        use super::*;

        #[test]
        fn test_name() {
            let rule = WrongHyphenBeforeSubcommand;
            assert_eq!(rule.name(), "wrong_hyphen_before_subcommand");
        }

        #[test]
        fn test_get_new_command() {
            let rule = WrongHyphenBeforeSubcommand;
            let cmd = Command::new("git-clone https://example.com/repo.git", "");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["git clone https://example.com/repo.git"]);
        }

        #[test]
        fn test_does_not_require_output() {
            let rule = WrongHyphenBeforeSubcommand;
            assert!(!rule.requires_output());
        }
    }

    // AptUpgrade tests
    mod apt_upgrade {
        use super::*;

        #[test]
        fn test_name() {
            let rule = AptUpgrade;
            assert_eq!(rule.name(), "apt_upgrade");
        }

        #[test]
        fn test_matches_with_upgrades() {
            let rule = AptUpgrade;
            let cmd = Command::new(
                "apt list --upgradable",
                "Listing...\npackage1/stable 1.0\npackage2/stable 2.0",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_no_upgrades() {
            let rule = AptUpgrade;
            let cmd = Command::new("apt list --upgradable", "Listing...");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_different_command() {
            let rule = AptUpgrade;
            let cmd = Command::new("apt list", "package1/stable 1.0\npackage2/stable 2.0");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = AptUpgrade;
            let cmd = Command::new("apt list --upgradable", "Listing...\npackage1/stable 1.0");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["apt upgrade"]);
        }

        #[test]
        fn test_get_new_command_with_sudo() {
            let rule = AptUpgrade;
            let cmd = Command::new(
                "sudo apt list --upgradable",
                "Listing...\npackage1/stable 1.0",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["sudo apt upgrade"]);
        }
    }

    // NoSuchFile tests
    mod no_such_file {
        use super::*;

        #[test]
        fn test_name() {
            let rule = NoSuchFile;
            assert_eq!(rule.name(), "no_such_file");
        }

        #[test]
        fn test_matches_mv_error() {
            let rule = NoSuchFile;
            let cmd = Command::new(
                "mv file.txt /path/to/dest/file.txt",
                "mv: cannot move 'file.txt' to '/path/to/dest/file.txt': No such file or directory",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_cp_error() {
            let rule = NoSuchFile;
            let cmd = Command::new(
                "cp file.txt /path/to/dest/file.txt",
                "cp: cannot create regular file '/path/to/dest/file.txt': No such file or directory",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let rule = NoSuchFile;
            let cmd = Command::new("mv file.txt dest/", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_mv() {
            let rule = NoSuchFile;
            let cmd = Command::new(
                "mv file.txt /path/to/dest/file.txt",
                "mv: cannot move 'file.txt' to '/path/to/dest/file.txt': No such file or directory",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(
                fixes,
                vec!["mkdir -p /path/to/dest && mv file.txt /path/to/dest/file.txt"]
            );
        }
    }

    // FixFile tests
    mod fix_file {
        use super::*;

        #[test]
        fn test_name() {
            let rule = FixFile;
            assert_eq!(rule.name(), "fix_file_error");
        }

        // Note: Many FixFile tests depend on file existence and EDITOR being set,
        // so we test what we can without side effects

        #[test]
        fn test_search_patterns() {
            // Test that our patterns can find file:line info
            let outputs = [
                ("  File \"test.py\", line 10", "test.py", "10"),
                ("test.rs:42:5", "test.rs", "42"),
                ("awk: test.awk:15:", "test.awk", "15"),
            ];

            for (output, expected_file, expected_line) in &outputs {
                // This tests the regex patterns work, even if the file doesn't exist
                // (which will cause search_error_location to return None)
                let re_python = Regex::new(r#"File "([^"]+)", line ([0-9]+)"#).unwrap();
                let re_generic = Regex::new(r"^([^:\n]+):([0-9]+):([0-9]+)").unwrap();
                let re_awk = Regex::new(r"^awk: ([^:\n]+):([0-9]+):").unwrap();

                if output.contains("File") {
                    let caps = re_python.captures(output).unwrap();
                    assert_eq!(caps.get(1).unwrap().as_str(), *expected_file);
                    assert_eq!(caps.get(2).unwrap().as_str(), *expected_line);
                } else if output.contains("awk") {
                    let caps = re_awk.captures(output).unwrap();
                    assert_eq!(caps.get(1).unwrap().as_str(), *expected_file);
                    assert_eq!(caps.get(2).unwrap().as_str(), *expected_line);
                } else {
                    let caps = re_generic.captures(output).unwrap();
                    assert_eq!(caps.get(1).unwrap().as_str(), *expected_file);
                    assert_eq!(caps.get(2).unwrap().as_str(), *expected_line);
                }
            }
        }
    }

    // Integration tests
    mod integration {
        use super::*;

        #[test]
        fn test_all_rules_returns_rules() {
            let rules = all_rules();
            assert!(!rules.is_empty());
            assert_eq!(rules.len(), 11); // FixFile is in system.rs
        }

        #[test]
        fn test_all_rules_have_unique_names() {
            let rules = all_rules();
            let mut names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
            let original_len = names.len();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), original_len, "Rule names should be unique");
        }

        #[test]
        fn test_all_rules_have_names() {
            let rules = all_rules();
            for rule in rules {
                assert!(!rule.name().is_empty());
            }
        }
    }
}
