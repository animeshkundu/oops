//! Language and framework rules (Python, Rails, React Native, etc.)
//!
//! This module contains correction rules for common language and framework errors:
//!
//! - Python: [`PythonExecute`], [`PythonModuleError`]
//! - Rails: [`RailsMigrationsPending`]
//! - React Native: [`ReactNativeCommandUnrecognized`]
//! - NixOS: [`NixosCmdNotFound`]
//! - Omnienv: [`OmnienvNoSuchCommand`]
//! - Django South: [`DjangoSouthGhost`], [`DjangoSouthMerge`]
//! - PHP: [`PhpS`]
//! - Virtualenv: [`WorkonDoesntExists`]
//! - Yarn: [`YarnAlias`], [`YarnCommandNotFound`], [`YarnCommandReplaced`], [`YarnHelp`]
//! - npm: [`NpmRunScript`]

use crate::core::{is_app, Command, Rule};
use crate::utils::{get_close_matches, replace_argument};
use regex::Regex;
use std::path::PathBuf;

// =============================================================================
// Python Rules
// =============================================================================

/// Rule that suggests appending .py extension when running Python files.
///
/// When a user runs `python foo` but the file is actually `foo.py`,
/// this rule suggests the corrected command.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::PythonExecute;
/// use oops::core::{Command, Rule};
///
/// let rule = PythonExecute;
/// let cmd = Command::new("python foo", "can't open file 'foo': No such file or directory");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PythonExecute;

impl Rule for PythonExecute {
    fn name(&self) -> &str {
        "python_execute"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        // Must be a python command
        if !is_app(cmd, &["python", "python3", "python2"]) {
            return false;
        }

        // Must not already end with .py
        if cmd.script.ends_with(".py") {
            return false;
        }

        // Check for "No such file or directory" or "can't open file" error
        cmd.output.contains("No such file or directory")
            || cmd.output.contains("can't open file")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("{}.py", cmd.script)]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that suggests installing missing Python modules via pip.
///
/// When Python raises a ModuleNotFoundError, this rule suggests
/// installing the missing module with pip and re-running the command.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::PythonModuleError;
/// use oops::core::{Command, Rule};
///
/// let rule = PythonModuleError;
/// let cmd = Command::new("python app.py", "ModuleNotFoundError: No module named 'requests'");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PythonModuleError;

impl PythonModuleError {
    /// Extract the missing module name from the error output.
    fn extract_module_name(output: &str) -> Option<String> {
        let re = Regex::new(r"ModuleNotFoundError: No module named '([^']+)'").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Rule for PythonModuleError {
    fn name(&self) -> &str {
        "python_module_error"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output.contains("ModuleNotFoundError: No module named '")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(module) = Self::extract_module_name(&cmd.output) {
            // Use && to chain commands (install then run)
            vec![format!("pip install {} && {}", module, cmd.script)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Rails Rules
// =============================================================================

/// Rule that suggests running pending Rails migrations.
///
/// When Rails indicates that migrations are pending, this rule extracts
/// the suggested migration command and runs it before re-running the original command.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::RailsMigrationsPending;
/// use oops::core::{Command, Rule};
///
/// let rule = RailsMigrationsPending;
/// let output = "Migrations are pending. To resolve this issue, run:\n  bin/rails db:migrate";
/// let cmd = Command::new("rails server", output);
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct RailsMigrationsPending;

impl RailsMigrationsPending {
    /// Extract the migration command from the error output.
    fn extract_migration_command(output: &str) -> Option<String> {
        let re = Regex::new(r"To resolve this issue, run:\s*\n?\s*(.+?)(?:\n|$)").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }
}

impl Rule for RailsMigrationsPending {
    fn name(&self) -> &str {
        "rails_migrations_pending"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.output
            .contains("Migrations are pending. To resolve this issue, run:")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(migration_cmd) = Self::extract_migration_command(&cmd.output) {
            vec![format!("{} && {}", migration_cmd, cmd.script)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// React Native Rules
// =============================================================================

/// Rule that corrects unrecognized React Native commands.
///
/// When react-native reports an unrecognized command, this rule suggests
/// similar valid commands.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::ReactNativeCommandUnrecognized;
/// use oops::core::{Command, Rule};
///
/// let rule = ReactNativeCommandUnrecognized;
/// let cmd = Command::new("react-native rn-android", "Unrecognized command 'rn-android'");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ReactNativeCommandUnrecognized;

/// Common React Native commands for fuzzy matching.
const REACT_NATIVE_COMMANDS: &[&str] = &[
    "start",
    "run-android",
    "run-ios",
    "bundle",
    "unbundle",
    "link",
    "unlink",
    "install",
    "uninstall",
    "log-android",
    "log-ios",
    "info",
    "upgrade",
    "config",
    "doctor",
    "init",
    "eject",
    "clean",
    "dependencies",
];

impl ReactNativeCommandUnrecognized {
    /// Extract the unrecognized command from the error output.
    fn extract_bad_command(output: &str) -> Option<String> {
        let re = Regex::new(r"Unrecognized command '([^']*)'").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Rule for ReactNativeCommandUnrecognized {
    fn name(&self) -> &str {
        "react_native_command_unrecognized"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["react-native"])
            && Self::extract_bad_command(&cmd.output).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(bad_cmd) = Self::extract_bad_command(&cmd.output) {
            let commands: Vec<String> = REACT_NATIVE_COMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect();
            let matches = get_close_matches(&bad_cmd, &commands, 3, 0.6);

            matches
                .into_iter()
                .map(|good_cmd| replace_argument(&cmd.script, &bad_cmd, &good_cmd))
                .collect()
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// NixOS Rules
// =============================================================================

/// Rule that suggests installing packages on NixOS.
///
/// When a command is not found on NixOS and nix-env suggests a package,
/// this rule extracts the suggestion and runs it before the original command.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::NixosCmdNotFound;
/// use oops::core::{Command, Rule};
///
/// let rule = NixosCmdNotFound;
/// let output = "command not found: htop\nnix-env -iA nixos.htop";
/// let cmd = Command::new("htop", output);
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NixosCmdNotFound;

impl NixosCmdNotFound {
    /// Extract the nix-env install command from the output.
    fn extract_nix_install(output: &str) -> Option<String> {
        let re = Regex::new(r"nix-env -iA ([^\s]+)").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Check if NixOS is available on this system.
    fn is_nix_available() -> bool {
        // Check if /etc/nixos exists or if nix-env is available
        PathBuf::from("/etc/nixos").exists()
            || crate::utils::which("nix-env").is_some()
    }
}

impl Rule for NixosCmdNotFound {
    fn name(&self) -> &str {
        "nixos_cmd_not_found"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn enabled_by_default(&self) -> bool {
        Self::is_nix_available()
    }

    fn is_match(&self, cmd: &Command) -> bool {
        Self::extract_nix_install(&cmd.output).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(package) = Self::extract_nix_install(&cmd.output) {
            vec![format!("nix-env -iA {} && {}", package, cmd.script)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Omnienv Rules (pyenv, rbenv, nodenv, goenv)
// =============================================================================

/// Rule that corrects invalid omnienv (pyenv, rbenv, nodenv, goenv) commands.
///
/// When an omnienv tool reports "no such command", this rule suggests
/// similar valid commands or common typo corrections.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::OmnienvNoSuchCommand;
/// use oops::core::{Command, Rule};
///
/// let rule = OmnienvNoSuchCommand;
/// let cmd = Command::new("pyenv list", "pyenv: no such command 'list'");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct OmnienvNoSuchCommand;

/// Supported omnienv applications.
const OMNIENV_APPS: &[&str] = &["pyenv", "rbenv", "nodenv", "goenv"];

/// Common typo corrections for omnienv commands.
const OMNIENV_TYPO_CORRECTIONS: &[(&str, &[&str])] = &[
    ("list", &["versions", "install --list"]),
    ("remove", &["uninstall"]),
];

/// Common omnienv commands for fuzzy matching.
const OMNIENV_COMMANDS: &[&str] = &[
    "commands",
    "local",
    "global",
    "shell",
    "install",
    "uninstall",
    "rehash",
    "version",
    "versions",
    "which",
    "whence",
    "shims",
    "init",
    "root",
    "prefix",
    "hooks",
    "completions",
    "exec",
    "help",
];

impl OmnienvNoSuchCommand {
    /// Check if any omnienv tool is available.
    fn is_omnienv_available() -> bool {
        OMNIENV_APPS.iter().any(|app| crate::utils::which(app).is_some())
    }

    /// Extract the bad command from the error output.
    fn extract_bad_command(output: &str) -> Option<String> {
        let re = Regex::new(r"env: no such command [`']([^'`]*)'").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Rule for OmnienvNoSuchCommand {
    fn name(&self) -> &str {
        "omnienv_no_such_command"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn enabled_by_default(&self) -> bool {
        Self::is_omnienv_available()
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, OMNIENV_APPS) && cmd.output.contains("env: no such command ")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let Some(bad_cmd) = Self::extract_bad_command(&cmd.output) else {
            return vec![];
        };

        let mut suggestions = Vec::new();

        // First, check for common typo corrections
        for (typo, corrections) in OMNIENV_TYPO_CORRECTIONS {
            if *typo == bad_cmd {
                for correction in *corrections {
                    suggestions.push(replace_argument(&cmd.script, &bad_cmd, correction));
                }
            }
        }

        // Then try fuzzy matching against known commands
        let commands: Vec<String> = OMNIENV_COMMANDS.iter().map(|s| s.to_string()).collect();
        let matches = get_close_matches(&bad_cmd, &commands, 3, 0.6);

        for good_cmd in matches {
            let suggestion = replace_argument(&cmd.script, &bad_cmd, &good_cmd);
            if !suggestions.contains(&suggestion) {
                suggestions.push(suggestion);
            }
        }

        suggestions
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Django South Rules
// =============================================================================

/// Rule that adds --delete-ghost-migrations flag for Django South.
///
/// When Django South migration fails due to ghost migrations, this rule
/// suggests adding the --delete-ghost-migrations flag.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::DjangoSouthGhost;
/// use oops::core::{Command, Rule};
///
/// let rule = DjangoSouthGhost;
/// let output = "... or pass --delete-ghost-migrations to delete these migrations";
/// let cmd = Command::new("python manage.py migrate", output);
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DjangoSouthGhost;

impl Rule for DjangoSouthGhost {
    fn name(&self) -> &str {
        "django_south_ghost"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("manage.py")
            && cmd.script.contains("migrate")
            && cmd.output.contains("or pass --delete-ghost-migrations")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("{} --delete-ghost-migrations", cmd.script)]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that adds --merge flag for Django South migration conflicts.
///
/// When Django South detects conflicting migrations, this rule suggests
/// adding the --merge flag to attempt the migration.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::DjangoSouthMerge;
/// use oops::core::{Command, Rule};
///
/// let rule = DjangoSouthMerge;
/// let output = "--merge: will just attempt the migration";
/// let cmd = Command::new("python manage.py migrate", output);
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct DjangoSouthMerge;

impl Rule for DjangoSouthMerge {
    fn name(&self) -> &str {
        "django_south_merge"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        cmd.script.contains("manage.py")
            && cmd.script.contains("migrate")
            && cmd.output.contains("--merge: will just attempt the migration")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![format!("{} --merge", cmd.script)]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// PHP Rules
// =============================================================================

/// Rule that fixes PHP -s (lowercase) to -S (uppercase) for the built-in server.
///
/// PHP's built-in web server uses -S (uppercase), but users often type -s.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::PhpS;
/// use oops::core::{Command, Rule};
///
/// let rule = PhpS;
/// let cmd = Command::new("php -s localhost:8000", "");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PhpS;

impl Rule for PhpS {
    fn name(&self) -> &str {
        "php_s"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["php"]) {
            return false;
        }

        let parts = cmd.script_parts();
        // Need at least 2 parts (php and something else)
        if parts.len() < 2 {
            return false;
        }

        // Check if -s is present and not at the end
        let has_s_flag = parts.iter().any(|p| p == "-s");
        let ends_with_s = parts.last().map(|p| p == "-s").unwrap_or(false);

        has_s_flag && !ends_with_s
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        vec![replace_argument(&cmd.script, "-s", "-S")]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// =============================================================================
// Virtualenv Rules
// =============================================================================

/// Rule that corrects misspelled virtualenv names in workon command.
///
/// When the user tries to activate a virtualenv that doesn't exist,
/// this rule suggests similar existing environments or creating a new one.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::WorkonDoesntExists;
/// use oops::core::{Command, Rule};
///
/// let rule = WorkonDoesntExists;
/// let cmd = Command::new("workon myenv", "");
/// // Will match if ~/.virtualenvs/myenv doesn't exist
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct WorkonDoesntExists;

impl WorkonDoesntExists {
    /// Get all available virtualenvs from ~/.virtualenvs.
    fn get_all_environments() -> Vec<String> {
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => return vec![],
        };

        let virtualenvs_dir = home.join(".virtualenvs");
        if !virtualenvs_dir.is_dir() {
            return vec![];
        }

        let entries = match std::fs::read_dir(&virtualenvs_dir) {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect()
    }
}

impl Rule for WorkonDoesntExists {
    fn name(&self) -> &str {
        "workon_doesnt_exists"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["workon"]) {
            return false;
        }

        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return false;
        }

        let env_name = &parts[1];
        let available = Self::get_all_environments();

        // Match if the requested environment is not in the available list
        !available.contains(env_name)
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return vec![];
        }

        let misspelled_env = &parts[1];
        let available = Self::get_all_environments();

        let mut suggestions = Vec::new();

        // Try to find similar environment names
        if !available.is_empty() {
            let matches = get_close_matches(misspelled_env, &available, 3, 0.6);
            for matched_env in matches {
                suggestions.push(replace_argument(&cmd.script, misspelled_env, &matched_env));
            }
        }

        // Always offer to create a new virtualenv
        suggestions.push(format!("mkvirtualenv {}", misspelled_env));

        suggestions
    }

    fn requires_output(&self) -> bool {
        false
    }
}

// =============================================================================
// Yarn Rules
// =============================================================================

/// Rule that accepts Yarn's "Did you mean" suggestions.
///
/// When Yarn suggests an alternative command with "Did you mean",
/// this rule extracts and uses that suggestion.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::YarnAlias;
/// use oops::core::{Command, Rule};
///
/// let rule = YarnAlias;
/// let cmd = Command::new("yarn instal", "Did you mean `yarn install`?");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct YarnAlias;

impl YarnAlias {
    /// Extract the suggested command from Yarn's "Did you mean" message.
    fn extract_suggestion(output: &str) -> Option<String> {
        let re = Regex::new(r#"Did you mean [`"](?:yarn )?([^`"]*)[`"]"#).ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Rule for YarnAlias {
    fn name(&self) -> &str {
        "yarn_alias"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["yarn"]) && cmd.output.contains("Did you mean")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(fix) = Self::extract_suggestion(&cmd.output) {
            let parts = cmd.script_parts();
            if parts.len() >= 2 {
                let broken = &parts[1];
                return vec![replace_argument(&cmd.script, broken, &fix)];
            }
        }
        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that corrects Yarn command not found errors.
///
/// When Yarn reports "Command not found", this rule suggests similar
/// valid commands or npm command equivalents.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::YarnCommandNotFound;
/// use oops::core::{Command, Rule};
///
/// let rule = YarnCommandNotFound;
/// let cmd = Command::new("yarn require express", "error Command \"require\" not found.");
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct YarnCommandNotFound;

/// Known Yarn commands for fuzzy matching.
const YARN_COMMANDS: &[&str] = &[
    "add",
    "audit",
    "autoclean",
    "bin",
    "cache",
    "check",
    "config",
    "create",
    "dedupe",
    "exec",
    "generate-lock-entry",
    "global",
    "help",
    "import",
    "info",
    "init",
    "install",
    "licenses",
    "link",
    "list",
    "login",
    "logout",
    "node",
    "outdated",
    "owner",
    "pack",
    "policies",
    "publish",
    "remove",
    "run",
    "tag",
    "team",
    "test",
    "unlink",
    "unplug",
    "upgrade",
    "upgrade-interactive",
    "version",
    "versions",
    "why",
    "workspace",
    "workspaces",
];

/// npm to Yarn command mappings for common migrations.
const NPM_TO_YARN_COMMANDS: &[(&str, &str)] = &[
    ("require", "add"),
    ("i", "install"),
    ("it", "install --test"),
    ("cit", "clean-install --test"),
    ("un", "remove"),
    ("rb", "rebuild"),
    ("up", "upgrade"),
];

impl YarnCommandNotFound {
    /// Extract the not found command from the error output.
    fn extract_bad_command(output: &str) -> Option<String> {
        let re = Regex::new(r#"error Command "([^"]*)" not found\."#).ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Rule for YarnCommandNotFound {
    fn name(&self) -> &str {
        "yarn_command_not_found"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["yarn"]) && Self::extract_bad_command(&cmd.output).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let Some(bad_cmd) = Self::extract_bad_command(&cmd.output) else {
            return vec![];
        };

        // First check for npm command equivalents
        for (npm_cmd, yarn_cmd) in NPM_TO_YARN_COMMANDS {
            if *npm_cmd == bad_cmd {
                return vec![replace_argument(&cmd.script, &bad_cmd, yarn_cmd)];
            }
        }

        // Otherwise try fuzzy matching
        let commands: Vec<String> = YARN_COMMANDS.iter().map(|s| s.to_string()).collect();
        let matches = get_close_matches(&bad_cmd, &commands, 3, 0.6);

        matches
            .into_iter()
            .map(|good_cmd| replace_argument(&cmd.script, &bad_cmd, &good_cmd))
            .collect()
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that handles Yarn deprecated/replaced commands.
///
/// When Yarn suggests running a different command instead,
/// this rule extracts and uses that replacement.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::YarnCommandReplaced;
/// use oops::core::{Command, Rule};
///
/// let rule = YarnCommandReplaced;
/// let output = "Run \"yarn add --dev\" instead";
/// let cmd = Command::new("yarn install --save-dev", output);
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct YarnCommandReplaced;

impl YarnCommandReplaced {
    /// Extract the replacement command from the output.
    fn extract_replacement(output: &str) -> Option<String> {
        let re = Regex::new(r#"Run "([^"]*)" instead"#).ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Rule for YarnCommandReplaced {
    fn name(&self) -> &str {
        "yarn_command_replaced"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        is_app(cmd, &["yarn"]) && Self::extract_replacement(&cmd.output).is_some()
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(replacement) = Self::extract_replacement(&cmd.output) {
            vec![replacement]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

/// Rule that opens Yarn documentation when help is requested.
///
/// When Yarn suggests visiting documentation, this rule opens the URL.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::YarnHelp;
/// use oops::core::{Command, Rule};
///
/// let rule = YarnHelp;
/// let output = "Visit https://yarnpkg.com/en/docs/cli/add for documentation about this command.";
/// let cmd = Command::new("yarn help add", output);
/// assert!(rule.is_match(&cmd));
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct YarnHelp;

impl YarnHelp {
    /// Extract the documentation URL from the output.
    fn extract_url(output: &str) -> Option<String> {
        let re = Regex::new(r"Visit ([^ ]*) for documentation about this command\.").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Get the command to open a URL based on the platform.
    fn get_open_command(url: &str) -> String {
        if cfg!(target_os = "macos") {
            format!("open {}", url)
        } else if cfg!(target_os = "windows") {
            format!("start {}", url)
        } else {
            // Linux and others - try xdg-open first
            format!("xdg-open {}", url)
        }
    }
}

impl Rule for YarnHelp {
    fn name(&self) -> &str {
        "yarn_help"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["yarn"]) {
            return false;
        }

        let parts = cmd.script_parts();
        parts.len() >= 2
            && parts[1] == "help"
            && cmd.output.contains("for documentation about this command.")
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        if let Some(url) = Self::extract_url(&cmd.output) {
            vec![Self::get_open_command(&url)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// npm Rules
// =============================================================================

/// Rule that adds 'run-script' for npm scripts that need it.
///
/// When npm shows usage help because a script name was used without 'run',
/// this rule suggests adding 'run-script' to execute the script.
///
/// # Example
///
/// ```
/// use oops::rules::frameworks::NpmRunScript;
/// use oops::core::{Command, Rule};
///
/// let rule = NpmRunScript;
/// let cmd = Command::new("npm build", "Usage: npm <command>");
/// // Will match if 'build' is a script in package.json
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NpmRunScript;

impl NpmRunScript {
    /// Check if npm is available.
    fn is_npm_available() -> bool {
        crate::utils::which("npm").is_some()
    }

    /// Get scripts from package.json in the current directory.
    /// Returns a cached or computed list of script names.
    fn get_scripts() -> Vec<String> {
        // Try to read package.json
        let package_json = std::path::Path::new("package.json");
        if !package_json.exists() {
            return vec![];
        }

        let content = match std::fs::read_to_string(package_json) {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        // Parse JSON and extract scripts
        // Using a simple regex-based approach to avoid adding json dependency
        let re = match Regex::new(r#""scripts"\s*:\s*\{([^}]*)\}"#) {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        let scripts_block = match re.captures(&content) {
            Some(caps) => caps.get(1).map(|m| m.as_str()).unwrap_or(""),
            None => return vec![],
        };

        // Extract script names
        let script_re = match Regex::new(r#""([^"]+)"\s*:"#) {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        script_re
            .captures_iter(scripts_block)
            .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }
}

impl Rule for NpmRunScript {
    fn name(&self) -> &str {
        "npm_run_script"
    }

    fn priority(&self) -> i32 {
        1000
    }

    fn enabled_by_default(&self) -> bool {
        Self::is_npm_available()
    }

    fn is_match(&self, cmd: &Command) -> bool {
        if !is_app(cmd, &["npm"]) {
            return false;
        }

        // Check for usage error
        if !cmd.output.contains("Usage: npm <command>") {
            return false;
        }

        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return false;
        }

        // Check if already using run/run-script
        if parts.iter().any(|p| p.starts_with("ru")) {
            return false;
        }

        // Check if the command is actually a script name
        let scripts = Self::get_scripts();
        scripts.contains(&parts[1])
    }

    fn get_new_command(&self, cmd: &Command) -> Vec<String> {
        let parts = cmd.script_parts();
        if parts.len() < 2 {
            return vec![];
        }

        // Insert 'run-script' after 'npm'
        let mut new_parts = vec![parts[0].clone(), "run-script".to_string()];
        new_parts.extend(parts[1..].iter().cloned());

        vec![new_parts.join(" ")]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

// =============================================================================
// Module Exports
// =============================================================================

/// Returns all framework rules as boxed trait objects.
///
/// This function creates instances of all framework and language rules
/// for registration with the rule system.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // Python rules
        Box::new(PythonExecute),
        Box::new(PythonModuleError),
        // Rails rules
        Box::new(RailsMigrationsPending),
        // React Native rules
        Box::new(ReactNativeCommandUnrecognized),
        // NixOS rules
        Box::new(NixosCmdNotFound),
        // Omnienv rules
        Box::new(OmnienvNoSuchCommand),
        // Django South rules
        Box::new(DjangoSouthGhost),
        Box::new(DjangoSouthMerge),
        // PHP rules
        Box::new(PhpS),
        // Virtualenv rules
        Box::new(WorkonDoesntExists),
        // Yarn rules
        Box::new(YarnAlias),
        Box::new(YarnCommandNotFound),
        Box::new(YarnCommandReplaced),
        Box::new(YarnHelp),
        // npm rules
        Box::new(NpmRunScript),
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // PythonExecute tests
    // -------------------------------------------------------------------------

    mod python_execute {
        use super::*;

        #[test]
        fn test_name() {
            let rule = PythonExecute;
            assert_eq!(rule.name(), "python_execute");
        }

        #[test]
        fn test_matches_no_such_file() {
            let rule = PythonExecute;
            let cmd = Command::new(
                "python foo",
                "python: can't open file 'foo': [Errno 2] No such file or directory",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_cant_open_file() {
            let rule = PythonExecute;
            let cmd = Command::new("python3 script", "can't open file 'script'");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_already_py() {
            let rule = PythonExecute;
            let cmd = Command::new("python foo.py", "No such file or directory");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_different_error() {
            let rule = PythonExecute;
            let cmd = Command::new("python foo", "SyntaxError");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = PythonExecute;
            let cmd = Command::new("python foo", "No such file or directory");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["python foo.py"]);
        }
    }

    // -------------------------------------------------------------------------
    // PythonModuleError tests
    // -------------------------------------------------------------------------

    mod python_module_error {
        use super::*;

        #[test]
        fn test_name() {
            let rule = PythonModuleError;
            assert_eq!(rule.name(), "python_module_error");
        }

        #[test]
        fn test_matches() {
            let rule = PythonModuleError;
            let cmd = Command::new(
                "python app.py",
                "Traceback:\n  ...\nModuleNotFoundError: No module named 'requests'",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_import_error() {
            let rule = PythonModuleError;
            let cmd = Command::new("python app.py", "ImportError: No module named foo");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = PythonModuleError;
            let cmd = Command::new(
                "python app.py",
                "ModuleNotFoundError: No module named 'requests'",
            );
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes.len(), 1);
            assert!(fixes[0].contains("pip install requests"));
            assert!(fixes[0].contains("python app.py"));
        }

        #[test]
        fn test_extract_module_name() {
            let output = "ModuleNotFoundError: No module named 'flask'";
            assert_eq!(
                PythonModuleError::extract_module_name(output),
                Some("flask".to_string())
            );
        }
    }

    // -------------------------------------------------------------------------
    // RailsMigrationsPending tests
    // -------------------------------------------------------------------------

    mod rails_migrations_pending {
        use super::*;

        #[test]
        fn test_name() {
            let rule = RailsMigrationsPending;
            assert_eq!(rule.name(), "rails_migrations_pending");
        }

        #[test]
        fn test_matches() {
            let rule = RailsMigrationsPending;
            let output =
                "Migrations are pending. To resolve this issue, run:\n  bin/rails db:migrate";
            let cmd = Command::new("rails server", output);
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match() {
            let rule = RailsMigrationsPending;
            let cmd = Command::new("rails server", "Server started");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = RailsMigrationsPending;
            let output =
                "Migrations are pending. To resolve this issue, run:\n  bin/rails db:migrate";
            let cmd = Command::new("rails server", output);
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes.len(), 1);
            assert!(fixes[0].contains("bin/rails db:migrate"));
            assert!(fixes[0].contains("rails server"));
        }
    }

    // -------------------------------------------------------------------------
    // ReactNativeCommandUnrecognized tests
    // -------------------------------------------------------------------------

    mod react_native_command_unrecognized {
        use super::*;

        #[test]
        fn test_name() {
            let rule = ReactNativeCommandUnrecognized;
            assert_eq!(rule.name(), "react_native_command_unrecognized");
        }

        #[test]
        fn test_matches() {
            let rule = ReactNativeCommandUnrecognized;
            let cmd = Command::new(
                "react-native rn-android",
                "Unrecognized command 'rn-android'",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_not_react_native() {
            let rule = ReactNativeCommandUnrecognized;
            let cmd = Command::new("npm run-android", "Unrecognized command 'run-android'");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = ReactNativeCommandUnrecognized;
            let cmd = Command::new(
                "react-native rn-android",
                "Unrecognized command 'rn-android'",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("run-android"));
        }
    }

    // -------------------------------------------------------------------------
    // NixosCmdNotFound tests
    // -------------------------------------------------------------------------

    mod nixos_cmd_not_found {
        use super::*;

        #[test]
        fn test_name() {
            let rule = NixosCmdNotFound;
            assert_eq!(rule.name(), "nixos_cmd_not_found");
        }

        #[test]
        fn test_matches() {
            let rule = NixosCmdNotFound;
            let output = "command not found: htop\nnix-env -iA nixos.htop";
            let cmd = Command::new("htop", output);
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match() {
            let rule = NixosCmdNotFound;
            let cmd = Command::new("htop", "command not found: htop");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = NixosCmdNotFound;
            let output = "command not found: htop\nnix-env -iA nixos.htop";
            let cmd = Command::new("htop", output);
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes.len(), 1);
            assert!(fixes[0].contains("nix-env -iA nixos.htop"));
            assert!(fixes[0].contains("htop"));
        }
    }

    // -------------------------------------------------------------------------
    // OmnienvNoSuchCommand tests
    // -------------------------------------------------------------------------

    mod omnienv_no_such_command {
        use super::*;

        #[test]
        fn test_name() {
            let rule = OmnienvNoSuchCommand;
            assert_eq!(rule.name(), "omnienv_no_such_command");
        }

        #[test]
        fn test_matches_pyenv() {
            let rule = OmnienvNoSuchCommand;
            let cmd = Command::new("pyenv list", "pyenv: no such command `list'");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_rbenv() {
            let rule = OmnienvNoSuchCommand;
            let cmd = Command::new("rbenv list", "rbenv: no such command `list'");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_different_app() {
            let rule = OmnienvNoSuchCommand;
            let cmd = Command::new("git list", "git: no such command 'list'");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_typo() {
            let rule = OmnienvNoSuchCommand;
            let cmd = Command::new("pyenv list", "pyenv: no such command `list'");
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            // Should suggest "versions" as a correction for "list"
            assert!(fixes.iter().any(|f| f.contains("versions")));
        }
    }

    // -------------------------------------------------------------------------
    // DjangoSouthGhost tests
    // -------------------------------------------------------------------------

    mod django_south_ghost {
        use super::*;

        #[test]
        fn test_name() {
            let rule = DjangoSouthGhost;
            assert_eq!(rule.name(), "django_south_ghost");
        }

        #[test]
        fn test_matches() {
            let rule = DjangoSouthGhost;
            let output = "error or pass --delete-ghost-migrations to fix";
            let cmd = Command::new("python manage.py migrate", output);
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_not_migrate() {
            let rule = DjangoSouthGhost;
            let output = "error or pass --delete-ghost-migrations to fix";
            let cmd = Command::new("python manage.py runserver", output);
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = DjangoSouthGhost;
            let output = "error or pass --delete-ghost-migrations to fix";
            let cmd = Command::new("python manage.py migrate", output);
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["python manage.py migrate --delete-ghost-migrations"]);
        }
    }

    // -------------------------------------------------------------------------
    // DjangoSouthMerge tests
    // -------------------------------------------------------------------------

    mod django_south_merge {
        use super::*;

        #[test]
        fn test_name() {
            let rule = DjangoSouthMerge;
            assert_eq!(rule.name(), "django_south_merge");
        }

        #[test]
        fn test_matches() {
            let rule = DjangoSouthMerge;
            let output = "--merge: will just attempt the migration";
            let cmd = Command::new("python manage.py migrate", output);
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = DjangoSouthMerge;
            let output = "--merge: will just attempt the migration";
            let cmd = Command::new("python manage.py migrate", output);
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["python manage.py migrate --merge"]);
        }
    }

    // -------------------------------------------------------------------------
    // PhpS tests
    // -------------------------------------------------------------------------

    mod php_s {
        use super::*;

        #[test]
        fn test_name() {
            let rule = PhpS;
            assert_eq!(rule.name(), "php_s");
        }

        #[test]
        fn test_matches() {
            let rule = PhpS;
            let cmd = Command::new("php -s localhost:8000", "");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_uppercase_s() {
            let rule = PhpS;
            let cmd = Command::new("php -S localhost:8000", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_s_at_end() {
            let rule = PhpS;
            let cmd = Command::new("php -s", "");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = PhpS;
            let cmd = Command::new("php -s localhost:8000", "");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["php -S localhost:8000"]);
        }

        #[test]
        fn test_requires_output() {
            let rule = PhpS;
            assert!(!rule.requires_output());
        }
    }

    // -------------------------------------------------------------------------
    // WorkonDoesntExists tests
    // -------------------------------------------------------------------------

    mod workon_doesnt_exists {
        use super::*;

        #[test]
        fn test_name() {
            let rule = WorkonDoesntExists;
            assert_eq!(rule.name(), "workon_doesnt_exists");
        }

        #[test]
        fn test_get_new_command_includes_mkvirtualenv() {
            let rule = WorkonDoesntExists;
            let cmd = Command::new("workon nonexistent_env_xyz", "");
            let fixes = rule.get_new_command(&cmd);
            // Should at least suggest creating the virtualenv
            assert!(fixes.iter().any(|f| f.contains("mkvirtualenv nonexistent_env_xyz")));
        }

        #[test]
        fn test_requires_output() {
            let rule = WorkonDoesntExists;
            assert!(!rule.requires_output());
        }
    }

    // -------------------------------------------------------------------------
    // YarnAlias tests
    // -------------------------------------------------------------------------

    mod yarn_alias {
        use super::*;

        #[test]
        fn test_name() {
            let rule = YarnAlias;
            assert_eq!(rule.name(), "yarn_alias");
        }

        #[test]
        fn test_matches() {
            let rule = YarnAlias;
            let cmd = Command::new("yarn instal", "Did you mean `yarn install`?");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_matches_double_quotes() {
            let rule = YarnAlias;
            let cmd = Command::new("yarn instal", "Did you mean \"yarn install\"?");
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_not_yarn() {
            let rule = YarnAlias;
            let cmd = Command::new("npm instal", "Did you mean `npm install`?");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = YarnAlias;
            let cmd = Command::new("yarn instal", "Did you mean `install`?");
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["yarn install"]);
        }

        #[test]
        fn test_extract_suggestion_with_yarn_prefix() {
            let output = "Did you mean `yarn install`?";
            assert_eq!(
                YarnAlias::extract_suggestion(output),
                Some("install".to_string())
            );
        }
    }

    // -------------------------------------------------------------------------
    // YarnCommandNotFound tests
    // -------------------------------------------------------------------------

    mod yarn_command_not_found {
        use super::*;

        #[test]
        fn test_name() {
            let rule = YarnCommandNotFound;
            assert_eq!(rule.name(), "yarn_command_not_found");
        }

        #[test]
        fn test_matches() {
            let rule = YarnCommandNotFound;
            let cmd = Command::new(
                "yarn require express",
                "error Command \"require\" not found.",
            );
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_not_yarn() {
            let rule = YarnCommandNotFound;
            let cmd = Command::new("npm require express", "error Command \"require\" not found.");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command_npm_equivalent() {
            let rule = YarnCommandNotFound;
            let cmd = Command::new(
                "yarn require express",
                "error Command \"require\" not found.",
            );
            let fixes = rule.get_new_command(&cmd);
            assert!(!fixes.is_empty());
            assert!(fixes[0].contains("add"));
        }
    }

    // -------------------------------------------------------------------------
    // YarnCommandReplaced tests
    // -------------------------------------------------------------------------

    mod yarn_command_replaced {
        use super::*;

        #[test]
        fn test_name() {
            let rule = YarnCommandReplaced;
            assert_eq!(rule.name(), "yarn_command_replaced");
        }

        #[test]
        fn test_matches() {
            let rule = YarnCommandReplaced;
            let output = "Run \"yarn add --dev\" instead";
            let cmd = Command::new("yarn install --save-dev", output);
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = YarnCommandReplaced;
            let output = "Run \"yarn add --dev\" instead";
            let cmd = Command::new("yarn install --save-dev", output);
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes, vec!["yarn add --dev"]);
        }
    }

    // -------------------------------------------------------------------------
    // YarnHelp tests
    // -------------------------------------------------------------------------

    mod yarn_help {
        use super::*;

        #[test]
        fn test_name() {
            let rule = YarnHelp;
            assert_eq!(rule.name(), "yarn_help");
        }

        #[test]
        fn test_matches() {
            let rule = YarnHelp;
            let output =
                "Visit https://yarnpkg.com/en/docs/cli/add for documentation about this command.";
            let cmd = Command::new("yarn help add", output);
            assert!(rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_not_help() {
            let rule = YarnHelp;
            let output =
                "Visit https://yarnpkg.com/en/docs/cli/add for documentation about this command.";
            let cmd = Command::new("yarn add", output);
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_get_new_command() {
            let rule = YarnHelp;
            let output =
                "Visit https://yarnpkg.com/en/docs/cli/add for documentation about this command.";
            let cmd = Command::new("yarn help add", output);
            let fixes = rule.get_new_command(&cmd);
            assert_eq!(fixes.len(), 1);
            assert!(fixes[0].contains("https://yarnpkg.com/en/docs/cli/add"));
        }

        #[test]
        fn test_extract_url() {
            let output =
                "Visit https://yarnpkg.com/en/docs/cli/add for documentation about this command.";
            assert_eq!(
                YarnHelp::extract_url(output),
                Some("https://yarnpkg.com/en/docs/cli/add".to_string())
            );
        }
    }

    // -------------------------------------------------------------------------
    // NpmRunScript tests
    // -------------------------------------------------------------------------

    mod npm_run_script {
        use super::*;

        #[test]
        fn test_name() {
            let rule = NpmRunScript;
            assert_eq!(rule.name(), "npm_run_script");
        }

        #[test]
        fn test_no_match_with_run() {
            let rule = NpmRunScript;
            let cmd = Command::new("npm run build", "Usage: npm <command>");
            assert!(!rule.is_match(&cmd));
        }

        #[test]
        fn test_no_match_not_npm() {
            let rule = NpmRunScript;
            let cmd = Command::new("yarn build", "Usage: npm <command>");
            assert!(!rule.is_match(&cmd));
        }
    }

    // -------------------------------------------------------------------------
    // Integration tests
    // -------------------------------------------------------------------------

    mod integration {
        use super::*;

        #[test]
        fn test_all_rules_have_names() {
            let rules = all_rules();
            for rule in rules {
                assert!(!rule.name().is_empty());
            }
        }

        #[test]
        fn test_all_rules_count() {
            let rules = all_rules();
            assert_eq!(rules.len(), 15);
        }

        #[test]
        fn test_no_duplicate_names() {
            let rules = all_rules();
            let mut names: Vec<&str> = rules.iter().map(|r| r.name()).collect();
            let original_len = names.len();
            names.sort();
            names.dedup();
            assert_eq!(names.len(), original_len, "Rule names should be unique");
        }
    }
}
