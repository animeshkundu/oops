//! Shell utility rules for common command-line errors.
//!
//! This module contains rules for various shell utilities:
//!
//! - [`AdbUnknownCommand`] - Android debug bridge fixes
//! - [`AgLiteral`] - Silver searcher literal search
//! - [`Dry`] - Suggests removing dry-run flag (duplicate word)
//! - [`GrepArgumentsOrder`] - Fix grep argument order
//! - [`GrepRecursive`] - Add -r for directory grep
//! - [`HasExistsScript`] - Handle script existence checks
//! - [`History`] - Shell history command fixes
//! - [`IfconfigDeviceNotFound`] - Network interface fixes
//! - [`LongFormHelp`] - Suggests --help instead of -help
//! - [`ProveRecursively`] - Perl prove -r flag
//! - [`SedUnterminatedS`] - Fix sed command syntax
//! - [`SwitchLang`] - Handle keyboard layout issues
//! - [`Mercurial`] - Mercurial/hg command fixes
//! - [`ScmCorrection`] - Source control typo fixes
//! - [`UnknownCommand`] - Generic unknown command handling

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, get_closest, replace_argument};
use regex::Regex;
use std::path::Path;

// ============================================================================
// ADB Unknown Command
// ============================================================================

/// ADB (Android Debug Bridge) commands that can be suggested for typos.
const ADB_COMMANDS: &[&str] = &[
    "backup",
    "bugreport",
    "connect",
    "devices",
    "disable-verity",
    "disconnect",
    "enable-verity",
    "emu",
    "forward",
    "get-devpath",
    "get-serialno",
    "get-state",
    "install",
    "install-multiple",
    "jdwp",
    "keygen",
    "kill-server",
    "logcat",
    "pull",
    "push",
    "reboot",
    "reconnect",
    "restore",
    "reverse",
    "root",
    "run-as",
    "shell",
    "sideload",
    "start-server",
    "sync",
    "tcpip",
    "uninstall",
    "unroot",
    "usb",
    "wait-for",
];

/// Rule that fixes unknown ADB commands.
///
/// When an ADB command is mistyped, this rule suggests the closest valid ADB command.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::AdbUnknownCommand;
/// use oops::core::{Command, Rule};
///
/// let rule = AdbUnknownCommand;
/// let cmd = Command::new("adb devics", "Android Debug Bridge version 1.0.41");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct AdbUnknownCommand;

impl AdbUnknownCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for AdbUnknownCommand {
    fn name(&self) -> &str {
        "adb_unknown_command"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["adb"]) && cmd.output.starts_with("Android Debug Bridge version")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();

        // Find the first non-option argument that's not a value for -s/-H/-P/-L
        for (idx, arg) in parts.iter().enumerate().skip(1) {
            // Skip options and their values
            if arg.starts_with('-') {
                continue;
            }
            // Check if this arg is a value for -s, -H, -P, or -L
            if idx > 0 {
                let prev = &parts[idx - 1];
                if prev == "-s" || prev == "-H" || prev == "-P" || prev == "-L" {
                    continue;
                }
            }

            // This should be the ADB command - find closest match
            let adb_cmds: Vec<String> = ADB_COMMANDS.iter().map(|s| s.to_string()).collect();
            if let Some(closest) = get_closest(arg, &adb_cmds, 0.6, true) {
                return vec![replace_argument(&cmd.script, arg, &closest)];
            }
        }

        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Ag Literal
// ============================================================================

/// Rule that adds -Q flag to ag (Silver Searcher) for literal searches.
///
/// When ag suggests using -Q for a literal search, this rule adds the flag.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::AgLiteral;
/// use oops::core::{Command, Rule};
///
/// let rule = AgLiteral;
/// let cmd = Command::new("ag pattern", "run ag with -Q\n");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct AgLiteral;

impl AgLiteral {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for AgLiteral {
    fn name(&self) -> &str {
        "ag_literal"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["ag"]) && cmd.output.ends_with("run ag with -Q\n")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace first "ag" with "ag -Q"
        if let Some(pos) = cmd.script.find("ag") {
            let mut result = cmd.script.clone();
            result.replace_range(pos..pos + 2, "ag -Q");
            return vec![result];
        }
        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Dry (duplicate first word - e.g., "git git status")
// ============================================================================

/// Rule that removes duplicate first word in commands.
///
/// Sometimes users accidentally type a command twice (e.g., "git git status").
/// This rule removes the duplicate.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::Dry;
/// use oops::core::{Command, Rule};
///
/// let rule = Dry;
/// let cmd = Command::new("git git status", "");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Dry;

impl Dry {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for Dry {
    fn name(&self) -> &str {
        "dry"
    }

    fn priority(&self) -> i32 {
        // Higher priority - this is rare but should be caught early
        900
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        parts.len() >= 2 && parts[0] == parts[1]
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.len() >= 2 {
            // Skip the first duplicate word
            vec![parts[1..].join(" ")]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// ============================================================================
// Grep Arguments Order
// ============================================================================

/// Rule that fixes grep argument order.
///
/// When grep fails because a file appears before the pattern, this rule
/// reorders the arguments to put the file at the end.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::GrepArgumentsOrder;
/// use oops::core::{Command, Rule};
///
/// let rule = GrepArgumentsOrder;
/// let cmd = Command::new("grep file.txt pattern", "file.txt: No such file or directory");
/// // Would match if file.txt exists
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GrepArgumentsOrder;

impl GrepArgumentsOrder {
    pub fn new() -> Self {
        Self
    }

    fn get_actual_file(parts: &[String]) -> Option<String> {
        for part in parts.iter().skip(1) {
            let path = Path::new(part);
            if path.is_file() || path.is_dir() {
                return Some(part.clone());
            }
        }
        None
    }
}

impl Rule for GrepArgumentsOrder {
    fn name(&self) -> &str {
        "grep_arguments_order"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["grep", "egrep"])
            && cmd.output.contains(": No such file or directory")
            && Self::get_actual_file(cmd.script_parts()).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if let Some(actual_file) = Self::get_actual_file(&parts) {
            // Move file to the end
            let mut new_parts: Vec<String> = parts
                .iter()
                .filter(|p| *p != &actual_file)
                .cloned()
                .collect();
            new_parts.push(actual_file);
            return vec![new_parts.join(" ")];
        }
        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Grep Recursive
// ============================================================================

/// Rule that adds -r flag to grep when searching in a directory.
///
/// When grep fails because the target is a directory, this rule adds the
/// recursive flag.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::GrepRecursive;
/// use oops::core::{Command, Rule};
///
/// let rule = GrepRecursive;
/// let cmd = Command::new("grep pattern .", ".: Is a directory");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct GrepRecursive;

impl GrepRecursive {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for GrepRecursive {
    fn name(&self) -> &str {
        "grep_recursive"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["grep"]) && cmd.output.to_lowercase().contains("is a directory")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Add -r after "grep"
        if cmd.script.starts_with("grep ") {
            vec![format!("grep -r {}", &cmd.script[5..])]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Has Exists Script
// ============================================================================

/// Rule that adds ./ prefix to run scripts in current directory.
///
/// When a script exists in the current directory but "command not found"
/// error occurs, this rule suggests running it with ./ prefix.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::HasExistsScript;
/// use oops::core::{Command, Rule};
///
/// let rule = HasExistsScript;
/// // Matches when the script file exists
/// let cmd = Command::new("myscript.sh", "myscript.sh: command not found");
/// // Would match if myscript.sh exists in current directory
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct HasExistsScript;

impl HasExistsScript {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for HasExistsScript {
    fn name(&self) -> &str {
        "has_exists_script"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return false;
        }

        // Check if the script exists and command not found
        let script_name = &parts[0];
        Path::new(script_name).exists() && cmd.output.contains("command not found")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("./{}", cmd.script)]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// History
// ============================================================================

/// Rule that suggests similar commands from shell history.
///
/// When a command is mistyped, this rule looks through the command history
/// for similar commands and suggests the closest match.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::History;
/// use oops::core::{Command, Rule};
///
/// let rule = History;
/// // This rule requires history context from environment
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct History;

impl History {
    pub fn new() -> Self {
        Self
    }

    fn get_history() -> Vec<String> {
        // Get history from TF_HISTORY environment variable
        std::env::var("TF_HISTORY")
            .unwrap_or_default()
            .lines()
            .map(|s| s.to_string())
            .collect()
    }
}

impl Rule for History {
    fn name(&self) -> &str {
        "shell_history"
    }

    fn priority(&self) -> i32 {
        // Very low priority - only use as a last resort
        9999
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let history = Self::get_history();
        let history_without_current: Vec<String> = history
            .iter()
            .filter(|h| *h != &cmd.script)
            .cloned()
            .collect();

        !get_close_matches(&cmd.script, &history_without_current, 1, 0.6).is_empty()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let history = Self::get_history();
        let history_without_current: Vec<String> = history
            .iter()
            .filter(|h| *h != &cmd.script)
            .cloned()
            .collect();

        if let Some(closest) = get_closest(&cmd.script, &history_without_current, 0.6, false) {
            vec![closest]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// ============================================================================
// Ifconfig Device Not Found
// ============================================================================

/// Rule that fixes ifconfig interface name typos.
///
/// When ifconfig fails to find a device, this rule suggests the closest
/// matching interface name.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::IfconfigDeviceNotFound;
/// use oops::core::{Command, Rule};
///
/// let rule = IfconfigDeviceNotFound;
/// let cmd = Command::new("ifconfig eth", "eth: error fetching interface information: Device not found");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct IfconfigDeviceNotFound;

impl IfconfigDeviceNotFound {
    pub fn new() -> Self {
        Self
    }

    fn get_interfaces() -> Vec<String> {
        // Try to get interfaces from ifconfig -a
        // This is a simplified version - real implementation would parse ifconfig output
        #[cfg(unix)]
        {
            use std::process::Command as ProcessCommand;

            if let Ok(output) = ProcessCommand::new("ifconfig").arg("-a").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout
                    .lines()
                    .filter(|line| !line.is_empty() && !line.starts_with(' '))
                    .filter_map(|line| line.split_whitespace().next())
                    .map(|s| s.trim_end_matches(':').to_string())
                    .collect();
            }
        }
        vec![]
    }
}

impl Rule for IfconfigDeviceNotFound {
    fn name(&self) -> &str {
        "ifconfig_device_not_found"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["ifconfig"])
            && cmd
                .output
                .contains("error fetching interface information: Device not found")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the interface name from output (format: "iface: error fetching...")
        let interface = cmd.output.split(':').next().unwrap_or("").trim();
        let interfaces = Self::get_interfaces();

        if let Some(closest) = get_closest(interface, &interfaces, 0.6, false) {
            return vec![replace_argument(&cmd.script, interface, &closest)];
        }
        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Long Form Help
// ============================================================================

/// Rule that suggests --help instead of -help or -h.
///
/// When a program suggests using --help, this rule provides the correct command.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::LongFormHelp;
/// use oops::core::{Command, Rule};
///
/// let rule = LongFormHelp;
/// let cmd = Command::new("command -h", "Try 'command --help' for more information.");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct LongFormHelp;

impl LongFormHelp {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for LongFormHelp {
    fn name(&self) -> &str {
        "long_form_help"
    }

    fn priority(&self) -> i32 {
        5000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Check for suggested help command pattern
        let help_regex = Regex::new(
            r"(?i)(?:Run|Try) '([^']+)'(?: or '[^']+')? for (?:details|more information)\.?",
        )
        .ok();

        if let Some(re) = help_regex {
            if re.is_match(&cmd.output) {
                return true;
            }
        }

        // Also match if output contains --help
        cmd.output.contains("--help")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Try to extract suggested command from output
        let help_regex = Regex::new(
            r"(?i)(?:Run|Try) '([^']+)'(?: or '[^']+')? for (?:details|more information)\.?",
        )
        .ok();

        if let Some(re) = help_regex {
            if let Some(captures) = re.captures(&cmd.output) {
                if let Some(suggested) = captures.get(1) {
                    return vec![suggested.as_str().to_string()];
                }
            }
        }

        // Fall back to replacing -h with --help
        vec![replace_argument(&cmd.script, "-h", "--help")]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Prove Recursively
// ============================================================================

/// Rule that adds -r flag to prove for recursive test running.
///
/// When prove fails with NOTESTS and a directory argument is provided,
/// this rule adds the recursive flag.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::ProveRecursively;
/// use oops::core::{Command, Rule};
///
/// let rule = ProveRecursively;
/// let cmd = Command::new("prove t/", "NOTESTS");
/// // Would match if t/ is a directory
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ProveRecursively;

impl ProveRecursively {
    pub fn new() -> Self {
        Self
    }

    fn is_recursive(part: &str) -> bool {
        if part == "--recurse" {
            return true;
        }
        if !part.starts_with("--") && part.starts_with('-') && part.contains('r') {
            return true;
        }
        false
    }

    fn has_directory_arg(parts: &[String]) -> bool {
        parts
            .iter()
            .skip(1)
            .any(|p| !p.starts_with('-') && Path::new(p).is_dir())
    }
}

impl Rule for ProveRecursively {
    fn name(&self) -> &str {
        "prove_recursively"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["prove"]) {
            return false;
        }

        let parts = cmd.script_parts();

        cmd.output.contains("NOTESTS")
            && !parts.iter().skip(1).any(|p| Self::is_recursive(p))
            && Self::has_directory_arg(&parts)
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        let mut new_parts = vec![parts[0].clone(), "-r".to_string()];
        new_parts.extend(parts[1..].iter().cloned());
        vec![new_parts.join(" ")]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Sed Unterminated S
// ============================================================================

/// Rule that fixes unterminated sed substitution commands.
///
/// When sed fails because an s command is missing a trailing slash,
/// this rule adds the missing delimiter.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::SedUnterminatedS;
/// use oops::core::{Command, Rule};
///
/// let rule = SedUnterminatedS;
/// let cmd = Command::new("sed 's/foo/bar' file.txt", "sed: -e expression #1, char 10: unterminated `s' command");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SedUnterminatedS;

impl SedUnterminatedS {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for SedUnterminatedS {
    fn name(&self) -> &str {
        "sed_unterminated_s"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["sed"]) && cmd.output.contains("unterminated `s' command")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Parse the command and fix unterminated s commands
        let parts = cmd.script_parts();
        let mut new_parts: Vec<String> = Vec::new();

        for part in parts {
            let mut new_part = part.clone();

            // Check if this is an s command that's missing a trailing delimiter
            if (part.starts_with("s/") || part.starts_with("-es/")) && !part.ends_with('/') {
                new_part.push('/');
            }

            new_parts.push(new_part);
        }

        vec![new_parts.join(" ")]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Switch Lang
// ============================================================================

/// Target keyboard layout (QWERTY).
const TARGET_LAYOUT: &str = "qwertyuiop[]asdfghjkl;'zxcvbnm,./QWERTYUIOP{}ASDFGHJKL:\"ZXCVBNM<>?";

/// Russian keyboard layout.
const RUSSIAN_LAYOUT: &str = "йцукенгшщзхъфывапролджэячсмитьбю.ЙЦУКЕНГШЩЗХЪФЫВАПРОЛДЖЭЯЧСМИТЬБЮ,";

/// Ukrainian keyboard layout.
const UKRAINIAN_LAYOUT: &str = "йцукенгшщзхїфівапролджєячсмитьбю.ЙЦУКЕНГШЩЗХЇФІВАПРОЛДЖЄЯЧСМИТЬБЮ,";

/// Rule that fixes commands typed with wrong keyboard layout.
///
/// When a user types a command with a non-English keyboard layout active,
/// this rule translates it to the correct layout.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::SwitchLang;
/// use oops::core::{Command, Rule};
///
/// let rule = SwitchLang;
/// // Russian "git status" typed in wrong layout
/// let cmd = Command::new("пше ыефегы", "command not found");
/// // Would match and translate to correct layout
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SwitchLang;

impl SwitchLang {
    pub fn new() -> Self {
        Self
    }

    fn get_matched_layout(script: &str) -> Option<&'static str> {
        let layouts = [RUSSIAN_LAYOUT, UKRAINIAN_LAYOUT];

        for layout in layouts {
            let all_match = script.split_whitespace().all(|word| {
                word.chars()
                    .all(|ch| layout.contains(ch) || ch == '-' || ch == '_')
            });

            if all_match {
                return Some(layout);
            }
        }
        None
    }

    fn switch_command(script: &str, source_layout: &str) -> String {
        let source_chars: Vec<char> = source_layout.chars().collect();
        let target_chars: Vec<char> = TARGET_LAYOUT.chars().collect();

        script
            .chars()
            .map(|ch| {
                if let Some(pos) = source_chars.iter().position(|&c| c == ch) {
                    if pos < target_chars.len() {
                        target_chars[pos]
                    } else {
                        ch
                    }
                } else {
                    ch
                }
            })
            .collect()
    }
}

impl Rule for SwitchLang {
    fn name(&self) -> &str {
        "switch_lang"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !cmd.output.contains("not found") {
            return false;
        }

        Self::get_matched_layout(&cmd.script).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(layout) = Self::get_matched_layout(&cmd.script) {
            let translated = Self::switch_command(&cmd.script, layout);
            return vec![translated];
        }
        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Mercurial
// ============================================================================

/// Rule that fixes Mercurial (hg) unknown command errors.
///
/// When hg fails with an unknown command, this rule suggests the closest
/// matching command from the suggestions in the output.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::Mercurial;
/// use oops::core::{Command, Rule};
///
/// let rule = Mercurial;
/// let cmd = Command::new("hg branchh", "hg: unknown command 'branchh'\n(did you mean one of branch, branches?)");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Mercurial;

impl Mercurial {
    pub fn new() -> Self {
        Self
    }

    fn extract_possibilities(output: &str) -> Vec<String> {
        // Try "(did you mean one of X, Y?)" pattern
        let did_you_mean_re = Regex::new(r"\(did you mean one of ([^?]+)\?\)").ok();
        if let Some(re) = did_you_mean_re {
            if let Some(captures) = re.captures(output) {
                if let Some(suggestions) = captures.get(1) {
                    return suggestions
                        .as_str()
                        .split(", ")
                        .map(|s| s.trim().to_string())
                        .collect();
                }
            }
        }

        // Try ambiguous command pattern (suggestions on last line)
        let lines: Vec<&str> = output.lines().collect();
        if let Some(last_line) = lines.last() {
            if last_line.starts_with("    ") {
                return last_line
                    .trim()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            }
        }

        vec![]
    }
}

impl Rule for Mercurial {
    fn name(&self) -> &str {
        "mercurial"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["hg"])
            && (cmd.output.contains("hg: unknown command")
                && cmd.output.contains("(did you mean one of ")
                || cmd.output.contains("hg: command '") && cmd.output.contains("' is ambiguous:"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return vec![];
        }

        let possibilities = Self::extract_possibilities(&cmd.output);
        if possibilities.is_empty() {
            return vec![];
        }

        let wrong_cmd = &parts[1];
        if let Some(closest) = get_closest(wrong_cmd, &possibilities, 0.6, true) {
            let mut new_parts = vec![parts[0].clone(), closest];
            new_parts.extend(parts[2..].iter().cloned());
            return vec![new_parts.join(" ")];
        }

        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// SCM Correction
// ============================================================================

/// Rule that corrects SCM (Source Control Management) command mistakes.
///
/// When using the wrong SCM command for a repository (e.g., using git in
/// an hg repo), this rule suggests the correct SCM.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::ScmCorrection;
/// use oops::core::{Command, Rule};
///
/// let rule = ScmCorrection;
/// let cmd = Command::new("git status", "fatal: Not a git repository");
/// // Would match if .hg directory exists
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ScmCorrection;

impl ScmCorrection {
    pub fn new() -> Self {
        Self
    }

    fn get_actual_scm() -> Option<&'static str> {
        if Path::new(".git").is_dir() {
            return Some("git");
        }
        if Path::new(".hg").is_dir() {
            return Some("hg");
        }
        None
    }
}

impl Rule for ScmCorrection {
    fn name(&self) -> &str {
        "scm_correction"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return false;
        }

        let scm = &parts[0];
        let pattern = match scm.as_str() {
            "git" => "fatal: Not a git repository",
            "hg" => "abort: no repository found",
            _ => return false,
        };

        cmd.output.contains(pattern) && Self::get_actual_scm().is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(actual_scm) = Self::get_actual_scm() {
            let parts = cmd.script_parts();
            if !parts.is_empty() {
                let mut new_parts = vec![actual_scm.to_string()];
                new_parts.extend(parts[1..].iter().cloned());
                return vec![new_parts.join(" ")];
            }
        }
        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// ============================================================================
// Unknown Command
// ============================================================================

/// Rule that fixes generic unknown command errors.
///
/// When a command fails with "Unknown command" and provides suggestions,
/// this rule extracts and uses those suggestions.
///
/// # Example
///
/// ```
/// use oops::rules::shell_utils::UnknownCommand;
/// use oops::core::{Command, Rule};
///
/// let rule = UnknownCommand;
/// let cmd = Command::new("foo bar", "bar: Unknown command. Did you mean baz?");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct UnknownCommand;

impl UnknownCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for UnknownCommand {
    fn name(&self) -> &str {
        "unknown_command"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let unknown_re = Regex::new(r"([^:]*): Unknown command.*").ok();
        let did_you_mean_re = Regex::new(r"Did you mean ([^?]*)\?").ok();

        if let (Some(unknown), Some(did_you_mean)) = (unknown_re, did_you_mean_re) {
            return unknown.is_match(&cmd.output) && did_you_mean.is_match(&cmd.output);
        }
        false
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let unknown_re = Regex::new(r"([^:]*): Unknown command.*").ok();
        let did_you_mean_re = Regex::new(r"Did you mean ([^?]*)\?").ok();

        if let (Some(unknown), Some(did_you_mean)) = (unknown_re, did_you_mean_re) {
            if let Some(broken_captures) = unknown.captures(&cmd.output) {
                if let Some(broken_cmd) = broken_captures.get(1) {
                    let suggestions: Vec<String> = did_you_mean
                        .captures_iter(&cmd.output)
                        .filter_map(|c| c.get(1))
                        .map(|m| m.as_str().to_string())
                        .collect();

                    if let Some(closest) = get_closest(broken_cmd.as_str(), &suggestions, 0.6, true)
                    {
                        return vec![replace_argument(&cmd.script, broken_cmd.as_str(), &closest)];
                    }
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
// All Rules
// ============================================================================

/// Returns all shell utility rules.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(AdbUnknownCommand::new()),
        Box::new(AgLiteral::new()),
        Box::new(Dry::new()),
        Box::new(GrepArgumentsOrder::new()),
        Box::new(GrepRecursive::new()),
        Box::new(HasExistsScript::new()),
        Box::new(History::new()),
        Box::new(IfconfigDeviceNotFound::new()),
        Box::new(LongFormHelp::new()),
        Box::new(ProveRecursively::new()),
        Box::new(SedUnterminatedS::new()),
        Box::new(SwitchLang::new()),
        Box::new(Mercurial::new()),
        Box::new(ScmCorrection::new()),
        Box::new(UnknownCommand::new()),
    ]
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ADB Unknown Command tests
    mod adb_unknown_command {
        use super::*;

        #[test]
        fn test_name() {
            let rule = AdbUnknownCommand::new();
            assert_eq!(rule.name(), "adb_unknown_command");
        }

        #[test]
        fn test_matches() {
            let rule = AdbUnknownCommand::new();
            let cmd = Command::new(
                "adb devics",
                "Android Debug Bridge version 1.0.41\nRevision...",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let rule = AdbUnknownCommand::new();
            let cmd = Command::new("adb devices", "List of devices attached\n");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = AdbUnknownCommand::new();
            let cmd = Command::new("adb devics", "Android Debug Bridge version 1.0.41");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("devices"));
        }
    }

    // Ag Literal tests
    mod ag_literal {
        use super::*;

        #[test]
        fn test_name() {
            let rule = AgLiteral::new();
            assert_eq!(rule.name(), "ag_literal");
        }

        #[test]
        fn test_matches() {
            let rule = AgLiteral::new();
            let cmd = Command::new("ag foo.bar", "ERR: run ag with -Q\n");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match() {
            let rule = AgLiteral::new();
            let cmd = Command::new("ag pattern", "matching line");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = AgLiteral::new();
            let cmd = Command::new("ag foo.bar", "run ag with -Q\n");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ag -Q foo.bar"]);
        }
    }

    // Dry tests
    mod dry {
        use super::*;

        #[test]
        fn test_name() {
            let rule = Dry::new();
            assert_eq!(rule.name(), "dry");
        }

        #[test]
        fn test_matches() {
            let rule = Dry::new();
            let cmd = Command::new("git git status", "");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_different_words() {
            let rule = Dry::new();
            let cmd = Command::new("git status", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_single_word() {
            let rule = Dry::new();
            let cmd = Command::new("git", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = Dry::new();
            let cmd = Command::new("git git status", "");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["git status"]);
        }

        #[test]
        fn test_requires_output() {
            let rule = Dry::new();
            assert!(!rule.requires_output());
        }

        #[test]
        fn test_priority() {
            let rule = Dry::new();
            assert_eq!(rule.priority(), 900);
        }
    }

    // Grep Recursive tests
    mod grep_recursive {
        use super::*;

        #[test]
        fn test_name() {
            let rule = GrepRecursive::new();
            assert_eq!(rule.name(), "grep_recursive");
        }

        #[test]
        fn test_matches() {
            let rule = GrepRecursive::new();
            let cmd = Command::new("grep pattern .", "grep: .: Is a directory");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_lowercase() {
            let rule = GrepRecursive::new();
            let cmd = Command::new("grep pattern dir", "grep: dir: is a directory");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let rule = GrepRecursive::new();
            let cmd = Command::new("grep pattern file.txt", "matching line");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = GrepRecursive::new();
            let cmd = Command::new("grep pattern .", "Is a directory");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["grep -r pattern ."]);
        }
    }

    // Sed Unterminated S tests
    mod sed_unterminated_s {
        use super::*;

        #[test]
        fn test_name() {
            let rule = SedUnterminatedS::new();
            assert_eq!(rule.name(), "sed_unterminated_s");
        }

        #[test]
        fn test_matches() {
            let rule = SedUnterminatedS::new();
            let cmd = Command::new(
                "sed 's/foo/bar' file.txt",
                "sed: -e expression #1, char 10: unterminated `s' command",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let rule = SedUnterminatedS::new();
            let cmd = Command::new("sed 's/foo/bar/' file.txt", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = SedUnterminatedS::new();
            let cmd = Command::new("sed s/foo/bar file.txt", "unterminated `s' command");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("s/foo/bar/"));
        }
    }

    // Switch Lang tests
    mod switch_lang {
        use super::*;

        #[test]
        fn test_name() {
            let rule = SwitchLang::new();
            assert_eq!(rule.name(), "switch_lang");
        }

        #[test]
        fn test_requires_output() {
            let rule = SwitchLang::new();
            assert!(rule.requires_output());
        }

        #[test]
        fn test_switch_russian() {
            // Test Russian "ls" typed with wrong layout
            let translated = SwitchLang::switch_command("дыб", RUSSIAN_LAYOUT);
            // Should translate to something
            assert_ne!(translated, "дыб");
        }
    }

    // Mercurial tests
    mod mercurial {
        use super::*;

        #[test]
        fn test_name() {
            let rule = Mercurial::new();
            assert_eq!(rule.name(), "mercurial");
        }

        #[test]
        fn test_matches_unknown_command() {
            let rule = Mercurial::new();
            let cmd = Command::new(
                "hg branchh",
                "hg: unknown command 'branchh'\n(did you mean one of branch, branches?)",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_ambiguous() {
            let rule = Mercurial::new();
            let cmd = Command::new("hg st", "hg: command 'st' is ambiguous:\n    status stash");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let rule = Mercurial::new();
            let cmd = Command::new("hg status", "? untracked.txt");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_extract_possibilities() {
            let output = "hg: unknown command 'branchh'\n(did you mean one of branch, branches?)";
            let possibilities = Mercurial::extract_possibilities(output);
            assert!(possibilities.contains(&"branch".to_string()));
            assert!(possibilities.contains(&"branches".to_string()));
        }
    }

    // SCM Correction tests
    mod scm_correction {
        use super::*;

        #[test]
        fn test_name() {
            let rule = ScmCorrection::new();
            assert_eq!(rule.name(), "scm_correction");
        }

        #[test]
        fn test_requires_output() {
            let rule = ScmCorrection::new();
            assert!(rule.requires_output());
        }
    }

    // Unknown Command tests
    mod unknown_command {
        use super::*;

        #[test]
        fn test_name() {
            let rule = UnknownCommand::new();
            assert_eq!(rule.name(), "unknown_command");
        }

        #[test]
        fn test_matches() {
            let rule = UnknownCommand::new();
            let cmd = Command::new("foo bar", "bar: Unknown command. Did you mean baz?");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match() {
            let rule = UnknownCommand::new();
            let cmd = Command::new("foo bar", "success");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_requires_output() {
            let rule = UnknownCommand::new();
            assert!(rule.requires_output());
        }
    }

    // Long Form Help tests
    mod long_form_help {
        use super::*;

        #[test]
        fn test_name() {
            let rule = LongFormHelp::new();
            assert_eq!(rule.name(), "long_form_help");
        }

        #[test]
        fn test_matches_try_help() {
            let rule = LongFormHelp::new();
            let cmd = Command::new("command -h", "Try 'command --help' for more information.");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_run_help() {
            let rule = LongFormHelp::new();
            let cmd = Command::new("command -h", "Run 'command --help' for details.");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_contains_help() {
            let rule = LongFormHelp::new();
            let cmd = Command::new("command -h", "Use --help to see all options");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match() {
            let rule = LongFormHelp::new();
            let cmd = Command::new("command -h", "Some helpful output");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_with_suggestion() {
            let rule = LongFormHelp::new();
            let cmd = Command::new("command -h", "Try 'command --help' for more information.");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["command --help"]);
        }

        #[test]
        fn test_priority() {
            let rule = LongFormHelp::new();
            assert_eq!(rule.priority(), 5000);
        }
    }

    // History tests
    mod history {
        use super::*;

        #[test]
        fn test_name() {
            let rule = History::new();
            assert_eq!(rule.name(), "shell_history");
        }

        #[test]
        fn test_priority() {
            let rule = History::new();
            assert_eq!(rule.priority(), 9999);
        }

        #[test]
        fn test_requires_output() {
            let rule = History::new();
            assert!(!rule.requires_output());
        }
    }

    // Integration tests
    mod integration {
        use super::*;

        #[test]
        fn test_all_rules_not_empty() {
            let rules = all_rules();
            assert_eq!(rules.len(), 15);
        }

        #[test]
        fn test_all_rules_have_names() {
            let rules = all_rules();
            for rule in rules {
                assert!(!rule.name().is_empty());
            }
        }

        #[test]
        fn test_no_duplicate_names() {
            let rules = all_rules();
            let mut names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
            let original_len = names.len();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), original_len, "Rule names must be unique");
        }

        #[test]
        fn test_all_rules_enabled_by_default() {
            let rules = all_rules();
            for rule in rules {
                assert!(rule.enabled_by_default());
            }
        }
    }
}
