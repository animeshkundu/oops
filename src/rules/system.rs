//! System and file operation rules.
//!
//! This module contains rules for correcting common system and file operation errors:
//!
//! - [`CatDir`] - Suggests ls when cat on directory
//! - [`ChmodX`] - Adds +x for permission denied on scripts
//! - [`CpCreateDestination`] - Creates destination dir for cp
//! - [`CpOmittingDirectory`] - Adds -r for directories
//! - [`DirtyUntar`] - Handles tar extracting to current dir
//! - [`DirtyUnzip`] - Handles zip extracting to current dir
//! - [`FixFile`] - Suggest file when "No such file"
//! - [`LnNoHardLink`] - Suggests -s for hard link errors
//! - [`LnSOrder`] - Fixes ln -s argument order
//! - [`LsAll`] - Adds -a when looking for hidden files
//! - [`LsLah`] - Suggests -lah for better listing
//! - [`MkdirP`] - Adds -p for nested dirs
//! - [`RmDir`] - Adds -r for directories
//! - [`RmRoot`] - Warning for rm -rf /
//! - [`Touch`] - Creates parent dirs for touch
//! - [`Man`] - Fixes man command errors
//! - [`ManNoSpace`] - Fixes "man-page" -> "man page"
//! - [`Open`] - Fixes open command (macOS/Linux)

use crate::core::{is_app, Command, Rule};
use regex::Regex;
use std::path::Path;

// =============================================================================
// CatDir - Suggests ls when cat on directory
// =============================================================================

/// Rule that suggests using `ls` when `cat` is used on a directory.
///
/// # Example
///
/// ```
/// use oops::rules::system::CatDir;
/// use oops::core::{Command, Rule};
///
/// let rule = CatDir;
/// let cmd = Command::new("cat /tmp", "cat: /tmp: Is a directory");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CatDir;

impl Rule for CatDir {
    fn name(&self) -> &str {
        "cat_dir"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["cat"]) && cmd.output.contains("Is a directory")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace only the first occurrence of "cat" with "ls"
        if let Some(pos) = cmd.script.find("cat") {
            let mut new_cmd = cmd.script.clone();
            new_cmd.replace_range(pos..pos + 3, "ls");
            vec![new_cmd]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// ChmodX - Adds +x for permission denied on scripts
// =============================================================================

/// Rule that adds execute permission when running a script fails with permission denied.
///
/// # Example
///
/// ```
/// use oops::rules::system::ChmodX;
/// use oops::core::{Command, Rule};
///
/// let rule = ChmodX;
/// let cmd = Command::new("./script.sh", "permission denied");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ChmodX;

impl Rule for ChmodX {
    fn name(&self) -> &str {
        "chmod_x"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.starts_with("./")
            && cmd.output.to_lowercase().contains("permission denied")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return vec![];
        }

        // Get the script path (without the ./ prefix for chmod)
        let script_path = &parts[0];
        let chmod_path = script_path.strip_prefix("./").unwrap_or(script_path);

        vec![format!("chmod +x {} && {}", chmod_path, cmd.script)]
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// CpCreateDestination - Creates destination dir for cp
// =============================================================================

/// Rule that creates the destination directory when cp/mv fails because it doesn't exist.
///
/// # Example
///
/// ```
/// use oops::rules::system::CpCreateDestination;
/// use oops::core::{Command, Rule};
///
/// let rule = CpCreateDestination;
/// let cmd = Command::new("cp file.txt /new/path/", "No such file or directory");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CpCreateDestination;

impl Rule for CpCreateDestination {
    fn name(&self) -> &str {
        "cp_create_destination"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["cp", "mv"])
            && (cmd.output.contains("No such file or directory")
                || (cmd.output.starts_with("cp: directory")
                    && cmd.output.trim_end().ends_with("does not exist")))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return vec![];
        }

        // Get the last argument (destination)
        let dest = &parts[parts.len() - 1];
        vec![format!("mkdir -p {} && {}", dest, cmd.script)]
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// CpOmittingDirectory - Adds -r for directories
// =============================================================================

/// Rule that adds -a flag when copying a directory without it.
///
/// # Example
///
/// ```
/// use oops::rules::system::CpOmittingDirectory;
/// use oops::core::{Command, Rule};
///
/// let rule = CpOmittingDirectory;
/// let cmd = Command::new("cp mydir /tmp/", "omitting directory 'mydir'");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct CpOmittingDirectory;

impl Rule for CpOmittingDirectory {
    fn name(&self) -> &str {
        "cp_omitting_directory"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let output_lower = cmd.output.to_lowercase();
        is_app(cmd, &["cp"])
            && (output_lower.contains("omitting directory")
                || output_lower.contains("is a directory"))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace "cp" with "cp -a" at the start
        if let Ok(re) = Regex::new(r"^cp\b") {
            let new_cmd = re.replace(&cmd.script, "cp -a").to_string();
            vec![new_cmd]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// DirtyUntar - Handles tar extracting to current dir
// =============================================================================

/// Tar extensions that this rule handles.
const TAR_EXTENSIONS: &[&str] = &[
    ".tar", ".tar.Z", ".tar.bz2", ".tar.gz", ".tar.lz",
    ".tar.lzma", ".tar.xz", ".taz", ".tb2", ".tbz", ".tbz2",
    ".tgz", ".tlz", ".txz", ".tz",
];

/// Rule that suggests extracting tar to a subdirectory to avoid polluting current dir.
///
/// # Example
///
/// ```
/// use oops::rules::system::DirtyUntar;
/// use oops::core::{Command, Rule};
///
/// let rule = DirtyUntar;
/// let cmd = Command::new("tar xf archive.tar.gz", "");
/// // Note: requires output=false
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DirtyUntar;

impl DirtyUntar {
    /// Check if the command is a tar extract operation.
    fn is_tar_extract(script: &str) -> bool {
        if script.contains("--extract") {
            return true;
        }

        let parts: Vec<&str> = script.split_whitespace().collect();
        parts.len() > 1 && parts[1].contains('x')
    }

    /// Find the tar file and its base name from command parts.
    fn tar_file(parts: &[String]) -> Option<(String, String)> {
        for part in parts {
            for ext in TAR_EXTENSIONS {
                if part.ends_with(ext) {
                    let base = part[..part.len() - ext.len()].to_string();
                    return Some((part.clone(), base));
                }
            }
        }
        None
    }
}

impl Rule for DirtyUntar {
    fn name(&self) -> &str {
        "dirty_untar"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["tar"])
            && !cmd.script.contains("-C")
            && Self::is_tar_extract(&cmd.script)
            && Self::tar_file(&cmd.script_parts()).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some((_, base)) = Self::tar_file(&cmd.script_parts()) {
            // Quote the directory name for shell safety
            let dir = shell_quote(&base);
            vec![format!("mkdir -p {} && {} -C {}", dir, cmd.script, dir)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        false
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// DirtyUnzip - Handles zip extracting to current dir
// =============================================================================

/// Rule that suggests extracting zip to a subdirectory.
///
/// Note: The original Python implementation checks if the zip has multiple files.
/// For simplicity, we check if -d is not already specified.
///
/// # Example
///
/// ```
/// use oops::rules::system::DirtyUnzip;
/// use oops::core::{Command, Rule};
///
/// let rule = DirtyUnzip;
/// let cmd = Command::new("unzip archive.zip", "");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DirtyUnzip;

impl DirtyUnzip {
    /// Find the zip file from command parts.
    fn zip_file(parts: &[String]) -> Option<String> {
        for part in parts.iter().skip(1) {
            if !part.starts_with('-') {
                if part.ends_with(".zip") {
                    return Some(part.clone());
                } else {
                    return Some(format!("{}.zip", part));
                }
            }
        }
        None
    }
}

impl Rule for DirtyUnzip {
    fn name(&self) -> &str {
        "dirty_unzip"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["unzip"])
            && !cmd.script.contains("-d")
            && Self::zip_file(&cmd.script_parts()).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(zip_file) = Self::zip_file(&cmd.script_parts()) {
            // Get base name without .zip extension
            let base = zip_file.strip_suffix(".zip").unwrap_or(&zip_file);
            let dir = shell_quote(base);
            vec![format!("{} -d {}", cmd.script, dir)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        false
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// FixFile - Suggest file when "No such file"
// =============================================================================

/// Patterns for detecting file:line:col references in error output.
const FIX_FILE_PATTERNS: &[&str] = &[
    // js, node:
    r"^\s+at (?P<file>[^:\n]+):(?P<line>\d+):(?P<col>\d+)",
    // cargo:
    r"^\s+(?P<file>[^:\n]+):(?P<line>\d+):(?P<col>\d+)",
    // python:
    r#"^\s+File "(?P<file>[^"]+)", line (?P<line>\d+)"#,
    // awk:
    r"^awk: (?P<file>[^:\n]+):(?P<line>\d+):",
    // git:
    r"^fatal: bad config file line (?P<line>\d+) in (?P<file>[^:\n]+)",
    // llc:
    r"^llc: (?P<file>[^:\n]+):(?P<line>\d+):(?P<col>\d+):",
    // lua:
    r"^lua: (?P<file>[^:\n]+):(?P<line>\d+):",
    // fish:
    r"^(?P<file>[^:\n]+) \(line (?P<line>\d+)\):",
    // bash, sh, ssh:
    r"^(?P<file>[^:\n]+): line (?P<line>\d+): ",
    // cargo, clang, gcc, go, pep8, rustc:
    r"^(?P<file>[^:\n]+):(?P<line>\d+):(?P<col>\d+)",
    // ghc, make, ruby, zsh:
    r"^(?P<file>[^:\n]+):(?P<line>\d+):",
    // perl:
    r"at (?P<file>[^:\n]+) line (?P<line>\d+)",
];

/// Rule that opens the editor at the file and line mentioned in error output.
///
/// # Example
///
/// ```
/// use oops::rules::system::FixFile;
/// use oops::core::{Command, Rule};
///
/// let rule = FixFile;
/// let cmd = Command::new("python script.py", "  File \"script.py\", line 10");
/// // Requires EDITOR env var to be set
/// ```
#[derive(Debug, Clone, Default)]
pub struct FixFile {
    patterns: Vec<Regex>,
}

impl FixFile {
    pub fn new() -> Self {
        let patterns = FIX_FILE_PATTERNS
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();
        Self { patterns }
    }

    fn search_output(&self, output: &str) -> Option<(String, String, Option<String>)> {
        for pattern in &self.patterns {
            if let Some(caps) = pattern.captures(output) {
                if let Some(file_match) = caps.name("file") {
                    let file = file_match.as_str().to_string();
                    // Check if file exists
                    if Path::new(&file).is_file() {
                        let line = caps.name("line").map(|m| m.as_str().to_string()).unwrap_or_default();
                        let col = caps.name("col").map(|m| m.as_str().to_string());
                        return Some((file, line, col));
                    }
                }
            }
        }
        None
    }
}

impl Rule for FixFile {
    fn name(&self) -> &str {
        "fix_file"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Check if EDITOR is set
        if std::env::var("EDITOR").is_err() {
            return false;
        }

        self.search_output(&cmd.output).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let editor = match std::env::var("EDITOR") {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        if let Some((file, line, _col)) = self.search_output(&cmd.output) {
            let editor_call = format!("{} {} +{}", editor, file, line);
            vec![format!("{} && {}", editor_call, cmd.script)]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// LnNoHardLink - Suggests -s for hard link errors
// =============================================================================

/// Rule that suggests using symbolic link when hard link fails.
///
/// # Example
///
/// ```
/// use oops::rules::system::LnNoHardLink;
/// use oops::core::{Command, Rule};
///
/// let rule = LnNoHardLink;
/// let cmd = Command::new("ln barDir barLink", "hard link not allowed for directory");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct LnNoHardLink;

impl Rule for LnNoHardLink {
    fn name(&self) -> &str {
        "ln_no_hard_link"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["ln"]) && cmd.output.contains("hard link not allowed for directory")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace "ln " with "ln -s " at the start
        if let Ok(re) = Regex::new(r"^ln\s") {
            let new_cmd = re.replace(&cmd.script, "ln -s ").to_string();
            vec![new_cmd]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// LnSOrder - Fixes ln -s argument order
// =============================================================================

/// Rule that fixes the argument order for ln -s when destination already exists.
///
/// # Example
///
/// ```
/// use oops::rules::system::LnSOrder;
/// use oops::core::{Command, Rule};
///
/// let rule = LnSOrder;
/// let cmd = Command::new("ln -s target link", "File exists");
/// // Will check if 'target' or 'link' exists and swap if needed
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct LnSOrder;

impl LnSOrder {
    /// Get the destination (existing path) from the command parts.
    fn get_destination(parts: &[String]) -> Option<String> {
        for part in parts {
            if part != "ln" && part != "-s" && part != "--symbolic" {
                if Path::new(part).exists() {
                    return Some(part.clone());
                }
            }
        }
        None
    }
}

impl Rule for LnSOrder {
    fn name(&self) -> &str {
        "ln_s_order"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        is_app(cmd, &["ln"])
            && (parts.contains(&"-s".to_string()) || parts.contains(&"--symbolic".to_string()))
            && cmd.output.contains("File exists")
            && Self::get_destination(&parts).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if let Some(dest) = Self::get_destination(&parts) {
            let mut new_parts: Vec<String> = parts
                .iter()
                .filter(|p| *p != &dest)
                .cloned()
                .collect();
            new_parts.push(dest);
            vec![new_parts.join(" ")]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// LsAll - Adds -a when looking for hidden files
// =============================================================================

/// Rule that suggests using ls -A when ls returns empty output.
///
/// # Example
///
/// ```
/// use oops::rules::system::LsAll;
/// use oops::core::{Command, Rule};
///
/// let rule = LsAll;
/// let cmd = Command::new("ls", "");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct LsAll;

impl Rule for LsAll {
    fn name(&self) -> &str {
        "ls_all"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["ls"]) && cmd.output.trim().is_empty()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return vec![];
        }

        // Reconstruct with -A flag
        let mut new_parts = vec!["ls".to_string(), "-A".to_string()];
        new_parts.extend(parts.iter().skip(1).cloned());
        vec![new_parts.join(" ")]
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// LsLah - Suggests -lah for better listing
// =============================================================================

/// Rule that suggests using ls -lah for better listing.
///
/// # Example
///
/// ```
/// use oops::rules::system::LsLah;
/// use oops::core::{Command, Rule};
///
/// let rule = LsLah;
/// let cmd = Command::new("ls mydir", "file1 file2");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct LsLah;

impl Rule for LsLah {
    fn name(&self) -> &str {
        "ls_lah"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        is_app(cmd, &["ls"]) && !parts.is_empty() && !cmd.script.contains("ls -")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.is_empty() {
            return vec![];
        }

        // Replace "ls" with "ls -lah"
        let mut new_parts = vec!["ls -lah".to_string()];
        new_parts.extend(parts.iter().skip(1).cloned());
        vec![new_parts.join(" ")]
    }

    fn priority(&self) -> i32 {
        9000 // Lower priority - this is more of a suggestion
    }
}

// =============================================================================
// MkdirP - Adds -p for nested dirs
// =============================================================================

/// Rule that adds -p flag when mkdir fails for nested directories.
///
/// # Example
///
/// ```
/// use oops::rules::system::MkdirP;
/// use oops::core::{Command, Rule};
///
/// let rule = MkdirP;
/// let cmd = Command::new("mkdir a/b/c", "No such file or directory");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MkdirP;

impl Rule for MkdirP {
    fn name(&self) -> &str {
        "mkdir_p"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("mkdir") && cmd.output.contains("No such file or directory")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Replace "mkdir" with "mkdir -p"
        if let Ok(re) = Regex::new(r"\bmkdir\s") {
            let new_cmd = re.replace(&cmd.script, "mkdir -p ").to_string();
            vec![new_cmd]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// RmDir - Adds -r for directories
// =============================================================================

/// Rule that adds -rf flag when rm fails because target is a directory.
///
/// # Example
///
/// ```
/// use oops::rules::system::RmDir;
/// use oops::core::{Command, Rule};
///
/// let rule = RmDir;
/// let cmd = Command::new("rm mydir", "is a directory");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct RmDir;

impl Rule for RmDir {
    fn name(&self) -> &str {
        "rm_dir"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("rm") && cmd.output.to_lowercase().contains("is a directory")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Use -r for hdfs, -rf for regular rm
        let args = if cmd.script.contains("hdfs") {
            "-r"
        } else {
            "-rf"
        };

        if let Ok(re) = Regex::new(r"\brm\s") {
            let replacement = format!("rm {} ", args);
            let new_cmd = re.replace(&cmd.script, replacement.as_str()).to_string();
            vec![new_cmd]
        } else {
            vec![]
        }
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// RmRoot - Warning for rm -rf /
// =============================================================================

/// Rule that adds --no-preserve-root when trying to rm -rf /.
///
/// This rule is disabled by default for safety.
///
/// # Example
///
/// ```
/// use oops::rules::system::RmRoot;
/// use oops::core::{Command, Rule};
///
/// let rule = RmRoot;
/// let cmd = Command::new("rm -rf /", "will not remove '/' without --no-preserve-root");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct RmRoot;

impl Rule for RmRoot {
    fn name(&self) -> &str {
        "rm_root"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        parts.contains(&"rm".to_string())
            && parts.contains(&"/".to_string())
            && !cmd.script.contains("--no-preserve-root")
            && cmd.output.contains("--no-preserve-root")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("{} --no-preserve-root", cmd.script)]
    }

    fn enabled_by_default(&self) -> bool {
        false // Disabled by default for safety
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// Touch - Creates parent dirs for touch
// =============================================================================

/// Rule that creates parent directories when touch fails.
///
/// # Example
///
/// ```
/// use oops::rules::system::Touch;
/// use oops::core::{Command, Rule};
///
/// let rule = Touch;
/// let cmd = Command::new("touch /new/path/file.txt", "No such file or directory");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Touch;

impl Rule for Touch {
    fn name(&self) -> &str {
        "touch"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["touch"]) && cmd.output.contains("No such file or directory")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Extract the path from error message
        // Pattern: touch: cannot touch 'path/file': No such file or directory
        // or: touch: 'path/file': No such file or directory
        let re = Regex::new(r"touch: (?:cannot touch ')?(.+)/[^/']+").ok();

        if let Some(re) = re {
            if let Some(caps) = re.captures(&cmd.output) {
                if let Some(path_match) = caps.get(1) {
                    let path = path_match.as_str().trim_end_matches('\'');
                    return vec![format!("mkdir -p {} && {}", path, cmd.script)];
                }
            }
        }

        // Fallback: try to get parent directory from command arguments
        let parts = cmd.script_parts();
        if parts.len() >= 2 {
            let file_path = &parts[parts.len() - 1];
            if let Some(parent) = Path::new(file_path).parent() {
                if !parent.as_os_str().is_empty() {
                    return vec![format!("mkdir -p {} && {}", parent.display(), cmd.script)];
                }
            }
        }

        vec![]
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// Man - Fixes man command errors
// =============================================================================

/// Rule that suggests alternative man page sections or --help.
///
/// # Example
///
/// ```
/// use oops::rules::system::Man;
/// use oops::core::{Command, Rule};
///
/// let rule = Man;
/// let cmd = Command::new("man printf", "No manual entry for printf");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Man;

impl Rule for Man {
    fn name(&self) -> &str {
        "man"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let parts = cmd.script_parts();
        is_app(cmd, &["man"]) && parts.len() >= 2
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return vec![];
        }

        // If command has section 3, try section 2
        if cmd.script.contains(" 3 ") || cmd.script.contains("3") && parts.len() > 2 {
            if parts.iter().any(|p| p == "3") {
                return vec![cmd.script.replace(" 3 ", " 2 ").replace(" 3", " 2")];
            }
        }

        // If command has section 2, try section 3
        if cmd.script.contains(" 2 ") || cmd.script.contains("2") && parts.len() > 2 {
            if parts.iter().any(|p| p == "2") {
                return vec![cmd.script.replace(" 2 ", " 3 ").replace(" 2", " 3")];
            }
        }

        let last_arg = &parts[parts.len() - 1];
        let help_command = format!("{} --help", last_arg);

        // If there's no man page, suggest --help
        let no_manual_msg = format!("No manual entry for {}", last_arg);
        if cmd.output.trim() == no_manual_msg {
            return vec![help_command];
        }

        // Otherwise, suggest section 3, section 2, and --help
        let mut results = Vec::new();

        // Build command with section 3
        let mut cmd3_parts: Vec<String> = parts.iter().cloned().collect();
        cmd3_parts.insert(1, "3".to_string());
        results.push(cmd3_parts.join(" "));

        // Build command with section 2
        let mut cmd2_parts: Vec<String> = parts.iter().cloned().collect();
        cmd2_parts.insert(1, "2".to_string());
        results.push(cmd2_parts.join(" "));

        results.push(help_command);
        results
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// ManNoSpace - Fixes "man-page" -> "man page"
// =============================================================================

/// Rule that fixes "man<topic>" to "man <topic>" when there's no space.
///
/// # Example
///
/// ```
/// use oops::rules::system::ManNoSpace;
/// use oops::core::{Command, Rule};
///
/// let rule = ManNoSpace;
/// let cmd = Command::new("manprintf", "command not found");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ManNoSpace;

impl Rule for ManNoSpace {
    fn name(&self) -> &str {
        "man_no_space"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.starts_with("man")
            && !cmd.script.starts_with("man ")
            && cmd.output.to_lowercase().contains("command not found")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        // Insert space after "man"
        let rest = &cmd.script[3..];
        vec![format!("man {}", rest)]
    }

    fn priority(&self) -> i32 {
        2000
    }
}

// =============================================================================
// Open - Fixes open command (macOS/Linux)
// =============================================================================

/// Common URL TLDs for detecting URLs.
const URL_TLDS: &[&str] = &[
    ".com", ".edu", ".info", ".io", ".ly", ".me", ".net", ".org", ".se", "www.",
];

/// Rule that fixes the open command for URLs and missing files.
///
/// # Example
///
/// ```
/// use oops::rules::system::Open;
/// use oops::core::{Command, Rule};
///
/// let rule = Open;
/// let cmd = Command::new("open github.com", "The file does not exist");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Open;

impl Open {
    fn is_arg_url(script: &str) -> bool {
        URL_TLDS.iter().any(|tld| script.contains(tld))
    }
}

impl Rule for Open {
    fn name(&self) -> &str {
        "open"
    }

    fn is_match(&self, cmd: &Command) -> bool {
        let output = cmd.output.trim();
        is_app(cmd, &["open", "xdg-open", "gnome-open", "kde-open"])
            && (Self::is_arg_url(&cmd.script)
                || (output.starts_with("The file ") && output.ends_with(" does not exist.")))
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let output = cmd.output.trim();
        let mut results = Vec::new();

        if Self::is_arg_url(&cmd.script) {
            // Add http:// prefix
            results.push(cmd.script.replace("open ", "open http://"));
        } else if output.starts_with("The file ") && output.ends_with(" does not exist.") {
            // Suggest creating the file or directory
            let parts: Vec<&str> = cmd.script.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let arg = parts[1];
                results.push(format!("touch {} && {}", arg, cmd.script));
                results.push(format!("mkdir {} && {}", arg, cmd.script));
            }
        }

        results
    }

    fn priority(&self) -> i32 {
        1000
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Simple shell quoting for safety.
fn shell_quote(s: &str) -> String {
    // If the string has no special characters, return as-is
    if s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

// =============================================================================
// all_rules() - Returns all rules in this module
// =============================================================================

/// Returns all system and file operation rules.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(CatDir),
        Box::new(ChmodX),
        Box::new(CpCreateDestination),
        Box::new(CpOmittingDirectory),
        Box::new(DirtyUntar),
        Box::new(DirtyUnzip),
        Box::new(FixFile::new()),
        Box::new(LnNoHardLink),
        Box::new(LnSOrder),
        Box::new(LsAll),
        Box::new(LsLah),
        Box::new(MkdirP),
        Box::new(RmDir),
        Box::new(RmRoot),
        Box::new(Touch),
        Box::new(Man),
        Box::new(ManNoSpace),
        Box::new(Open),
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // CatDir Tests
    // -------------------------------------------------------------------------
    mod cat_dir {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(CatDir.name(), "cat_dir");
        }

        #[test]
        fn test_matches_cat_directory() {
            let cmd = Command::new("cat /tmp", "cat: /tmp: Is a directory");
            assert!(CatDir.is_match(&cmd));
        }

        #[test]
        fn test_no_match_successful_cat() {
            let cmd = Command::new("cat file.txt", "file contents");
            assert!(!CatDir.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("cat /tmp", "cat: /tmp: Is a directory");
            let fixes = CatDir.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ls /tmp"]);
        }
    }

    // -------------------------------------------------------------------------
    // ChmodX Tests
    // -------------------------------------------------------------------------
    mod chmod_x {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(ChmodX.name(), "chmod_x");
        }

        #[test]
        fn test_matches_permission_denied() {
            let cmd = Command::new("./script.sh", "bash: ./script.sh: Permission denied");
            assert!(ChmodX.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_command() {
            let cmd = Command::new("ls", "Permission denied");
            assert!(!ChmodX.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("./script.sh", "Permission denied");
            let fixes = ChmodX.get_new_command(&cmd);
            assert_eq!(fixes, vec!["chmod +x script.sh && ./script.sh"]);
        }
    }

    // -------------------------------------------------------------------------
    // CpCreateDestination Tests
    // -------------------------------------------------------------------------
    mod cp_create_destination {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(CpCreateDestination.name(), "cp_create_destination");
        }

        #[test]
        fn test_matches_no_such_directory() {
            let cmd = Command::new("cp file.txt /new/path/", "No such file or directory");
            assert!(CpCreateDestination.is_match(&cmd));
        }

        #[test]
        fn test_matches_mv_no_such_directory() {
            let cmd = Command::new("mv file.txt /new/path/", "No such file or directory");
            assert!(CpCreateDestination.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("cp file.txt /new/path/", "No such file or directory");
            let fixes = CpCreateDestination.get_new_command(&cmd);
            assert_eq!(fixes, vec!["mkdir -p /new/path/ && cp file.txt /new/path/"]);
        }
    }

    // -------------------------------------------------------------------------
    // CpOmittingDirectory Tests
    // -------------------------------------------------------------------------
    mod cp_omitting_directory {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(CpOmittingDirectory.name(), "cp_omitting_directory");
        }

        #[test]
        fn test_matches_omitting_directory() {
            let cmd = Command::new("cp mydir /tmp/", "cp: omitting directory 'mydir'");
            assert!(CpOmittingDirectory.is_match(&cmd));
        }

        #[test]
        fn test_matches_is_directory() {
            let cmd = Command::new("cp mydir /tmp/", "cp: mydir is a directory");
            assert!(CpOmittingDirectory.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("cp mydir /tmp/", "cp: omitting directory 'mydir'");
            let fixes = CpOmittingDirectory.get_new_command(&cmd);
            assert_eq!(fixes, vec!["cp -a mydir /tmp/"]);
        }
    }

    // -------------------------------------------------------------------------
    // DirtyUntar Tests
    // -------------------------------------------------------------------------
    mod dirty_untar {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(DirtyUntar.name(), "dirty_untar");
        }

        #[test]
        fn test_matches_tar_extract() {
            let cmd = Command::new("tar xf archive.tar.gz", "");
            assert!(DirtyUntar.is_match(&cmd));
        }

        #[test]
        fn test_matches_tar_extract_verbose() {
            let cmd = Command::new("tar xvf archive.tar", "");
            assert!(DirtyUntar.is_match(&cmd));
        }

        #[test]
        fn test_matches_tar_extract_long() {
            let cmd = Command::new("tar --extract -f archive.tgz", "");
            assert!(DirtyUntar.is_match(&cmd));
        }

        #[test]
        fn test_no_match_with_c_flag() {
            let cmd = Command::new("tar xf archive.tar.gz -C /tmp", "");
            assert!(!DirtyUntar.is_match(&cmd));
        }

        #[test]
        fn test_no_match_non_tar() {
            let cmd = Command::new("unzip archive.zip", "");
            assert!(!DirtyUntar.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("tar xf archive.tar.gz", "");
            let fixes = DirtyUntar.get_new_command(&cmd);
            assert_eq!(fixes, vec!["mkdir -p archive && tar xf archive.tar.gz -C archive"]);
        }

        #[test]
        fn test_requires_no_output() {
            assert!(!DirtyUntar.requires_output());
        }
    }

    // -------------------------------------------------------------------------
    // DirtyUnzip Tests
    // -------------------------------------------------------------------------
    mod dirty_unzip {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(DirtyUnzip.name(), "dirty_unzip");
        }

        #[test]
        fn test_matches_unzip() {
            let cmd = Command::new("unzip archive.zip", "");
            assert!(DirtyUnzip.is_match(&cmd));
        }

        #[test]
        fn test_no_match_with_d_flag() {
            let cmd = Command::new("unzip archive.zip -d /tmp", "");
            assert!(!DirtyUnzip.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("unzip archive.zip", "");
            let fixes = DirtyUnzip.get_new_command(&cmd);
            assert_eq!(fixes, vec!["unzip archive.zip -d archive"]);
        }

        #[test]
        fn test_requires_no_output() {
            assert!(!DirtyUnzip.requires_output());
        }
    }

    // -------------------------------------------------------------------------
    // LnNoHardLink Tests
    // -------------------------------------------------------------------------
    mod ln_no_hard_link {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(LnNoHardLink.name(), "ln_no_hard_link");
        }

        #[test]
        fn test_matches_hard_link_error() {
            let cmd = Command::new("ln barDir barLink", "ln: 'barDir': hard link not allowed for directory");
            assert!(LnNoHardLink.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_error() {
            let cmd = Command::new("ln file link", "ln: failed to create link");
            assert!(!LnNoHardLink.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("ln barDir barLink", "hard link not allowed for directory");
            let fixes = LnNoHardLink.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ln -s barDir barLink"]);
        }
    }

    // -------------------------------------------------------------------------
    // LsAll Tests
    // -------------------------------------------------------------------------
    mod ls_all {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(LsAll.name(), "ls_all");
        }

        #[test]
        fn test_matches_empty_output() {
            let cmd = Command::new("ls", "");
            assert!(LsAll.is_match(&cmd));
        }

        #[test]
        fn test_no_match_with_output() {
            let cmd = Command::new("ls", "file1 file2");
            assert!(!LsAll.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("ls", "");
            let fixes = LsAll.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ls -A"]);
        }

        #[test]
        fn test_get_new_command_with_path() {
            let cmd = Command::new("ls /tmp", "");
            let fixes = LsAll.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ls -A /tmp"]);
        }
    }

    // -------------------------------------------------------------------------
    // LsLah Tests
    // -------------------------------------------------------------------------
    mod ls_lah {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(LsLah.name(), "ls_lah");
        }

        #[test]
        fn test_matches_plain_ls() {
            let cmd = Command::new("ls mydir", "file1 file2");
            assert!(LsLah.is_match(&cmd));
        }

        #[test]
        fn test_no_match_with_flags() {
            let cmd = Command::new("ls -l mydir", "file1 file2");
            assert!(!LsLah.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("ls mydir", "file1 file2");
            let fixes = LsLah.get_new_command(&cmd);
            assert_eq!(fixes, vec!["ls -lah mydir"]);
        }

        #[test]
        fn test_lower_priority() {
            assert!(LsLah.priority() > 1000);
        }
    }

    // -------------------------------------------------------------------------
    // MkdirP Tests
    // -------------------------------------------------------------------------
    mod mkdir_p {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(MkdirP.name(), "mkdir_p");
        }

        #[test]
        fn test_matches_no_such_directory() {
            let cmd = Command::new("mkdir a/b/c", "mkdir: cannot create directory 'a/b/c': No such file or directory");
            assert!(MkdirP.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let cmd = Command::new("mkdir newdir", "");
            assert!(!MkdirP.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("mkdir a/b/c", "No such file or directory");
            let fixes = MkdirP.get_new_command(&cmd);
            assert_eq!(fixes, vec!["mkdir -p a/b/c"]);
        }
    }

    // -------------------------------------------------------------------------
    // RmDir Tests
    // -------------------------------------------------------------------------
    mod rm_dir {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(RmDir.name(), "rm_dir");
        }

        #[test]
        fn test_matches_is_directory() {
            let cmd = Command::new("rm mydir", "rm: cannot remove 'mydir': Is a directory");
            assert!(RmDir.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("rm mydir", "Is a directory");
            let fixes = RmDir.get_new_command(&cmd);
            assert_eq!(fixes, vec!["rm -rf mydir"]);
        }

        #[test]
        fn test_get_new_command_hdfs() {
            let cmd = Command::new("hdfs dfs -rm /path/dir", "Is a directory");
            let fixes = RmDir.get_new_command(&cmd);
            // Note: This matches "rm" in the script, should have -r not -rf for hdfs
            assert!(fixes[0].contains("-r"));
        }
    }

    // -------------------------------------------------------------------------
    // RmRoot Tests
    // -------------------------------------------------------------------------
    mod rm_root {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(RmRoot.name(), "rm_root");
        }

        #[test]
        fn test_matches_rm_root() {
            let cmd = Command::new("rm -rf /", "rm: it is dangerous to operate recursively on '/'\nrm: use --no-preserve-root to override this failsafe");
            assert!(RmRoot.is_match(&cmd));
        }

        #[test]
        fn test_no_match_other_path() {
            let cmd = Command::new("rm -rf /tmp", "rm: cannot remove '/tmp': Permission denied");
            assert!(!RmRoot.is_match(&cmd));
        }

        #[test]
        fn test_disabled_by_default() {
            assert!(!RmRoot.enabled_by_default());
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("rm -rf /", "--no-preserve-root");
            let fixes = RmRoot.get_new_command(&cmd);
            assert_eq!(fixes, vec!["rm -rf / --no-preserve-root"]);
        }
    }

    // -------------------------------------------------------------------------
    // Touch Tests
    // -------------------------------------------------------------------------
    mod touch {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(Touch.name(), "touch");
        }

        #[test]
        fn test_matches_no_such_directory() {
            let cmd = Command::new("touch /new/path/file.txt", "touch: cannot touch '/new/path/file.txt': No such file or directory");
            assert!(Touch.is_match(&cmd));
        }

        #[test]
        fn test_no_match_success() {
            let cmd = Command::new("touch file.txt", "");
            assert!(!Touch.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("touch /new/path/file.txt", "touch: cannot touch '/new/path/file.txt': No such file or directory");
            let fixes = Touch.get_new_command(&cmd);
            assert!(fixes[0].contains("mkdir -p") && fixes[0].contains("touch"));
        }
    }

    // -------------------------------------------------------------------------
    // Man Tests
    // -------------------------------------------------------------------------
    mod man {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(Man.name(), "man");
        }

        #[test]
        fn test_matches_man_command() {
            let cmd = Command::new("man printf", "No manual entry for printf");
            assert!(Man.is_match(&cmd));
        }

        #[test]
        fn test_no_match_bare_man() {
            let cmd = Command::new("man", "What manual page do you want?");
            assert!(!Man.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_no_manual() {
            let cmd = Command::new("man printf", "No manual entry for printf");
            let fixes = Man.get_new_command(&cmd);
            assert_eq!(fixes, vec!["printf --help"]);
        }

        #[test]
        fn test_get_new_command_multiple_suggestions() {
            let cmd = Command::new("man printf", "Some other error");
            let fixes = Man.get_new_command(&cmd);
            assert!(fixes.len() >= 2);
            assert!(fixes.iter().any(|f| f.contains("3")));
            assert!(fixes.iter().any(|f| f.contains("2")));
        }
    }

    // -------------------------------------------------------------------------
    // ManNoSpace Tests
    // -------------------------------------------------------------------------
    mod man_no_space {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(ManNoSpace.name(), "man_no_space");
        }

        #[test]
        fn test_matches_man_no_space() {
            let cmd = Command::new("manprintf", "manprintf: command not found");
            assert!(ManNoSpace.is_match(&cmd));
        }

        #[test]
        fn test_no_match_man_with_space() {
            let cmd = Command::new("man printf", "No manual entry");
            assert!(!ManNoSpace.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let cmd = Command::new("manprintf", "command not found");
            let fixes = ManNoSpace.get_new_command(&cmd);
            assert_eq!(fixes, vec!["man printf"]);
        }

        #[test]
        fn test_lower_priority() {
            assert!(ManNoSpace.priority() > 1000);
        }
    }

    // -------------------------------------------------------------------------
    // Open Tests
    // -------------------------------------------------------------------------
    mod open {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(Open.name(), "open");
        }

        #[test]
        fn test_matches_url() {
            let cmd = Command::new("open github.com", "");
            assert!(Open.is_match(&cmd));
        }

        #[test]
        fn test_matches_file_not_exist() {
            let cmd = Command::new("open myfile", "The file ~/myfile does not exist.");
            assert!(Open.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_url() {
            let cmd = Command::new("open github.com", "");
            let fixes = Open.get_new_command(&cmd);
            assert_eq!(fixes, vec!["open http://github.com"]);
        }

        #[test]
        fn test_get_new_command_missing_file() {
            let cmd = Command::new("open myfile", "The file ~/myfile does not exist.");
            let fixes = Open.get_new_command(&cmd);
            assert!(fixes.iter().any(|f| f.contains("touch")));
            assert!(fixes.iter().any(|f| f.contains("mkdir")));
        }
    }

    // -------------------------------------------------------------------------
    // Integration Tests
    // -------------------------------------------------------------------------
    mod integration {
        use super::*;

        #[test]
        fn test_all_rules_returns_all() {
            let rules = all_rules();
            assert_eq!(rules.len(), 18);
        }

        #[test]
        fn test_all_rules_have_unique_names() {
            let rules = all_rules();
            let mut names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
            let original_len = names.len();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), original_len);
        }

        #[test]
        fn test_shell_quote_simple() {
            assert_eq!(shell_quote("simple"), "simple");
            assert_eq!(shell_quote("with-dash"), "with-dash");
            assert_eq!(shell_quote("with_underscore"), "with_underscore");
        }

        #[test]
        fn test_shell_quote_special() {
            assert_eq!(shell_quote("with space"), "'with space'");
            assert_eq!(shell_quote("with'quote"), "'with'\\''quote'");
        }
    }
}
