//! Configuration file loading for oops.
//!
//! This module handles loading settings from multiple sources in order of priority:
//! 1. Default values
//! 2. Settings file (~/.config/oops/config.toml or ~/.config/thefuck/settings.toml for migration)
//! 3. Environment variables (THEFUCK_* for backward compatibility)
//! 4. CLI arguments

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use tracing::{debug, warn};

use super::Settings;
use crate::cli::Cli;

/// Global settings instance, lazily initialized.
///
/// This is initialized with default settings and should be updated
/// via `init_settings()` after CLI arguments are parsed.
pub static SETTINGS: Lazy<std::sync::RwLock<Settings>> =
    Lazy::new(|| std::sync::RwLock::new(Settings::default()));

/// Initialize global settings with CLI arguments.
///
/// This should be called once at startup after parsing CLI arguments.
pub fn init_settings(cli_args: &Cli) -> Result<()> {
    let settings = load_settings(cli_args)?;
    let mut global_settings = SETTINGS
        .write()
        .map_err(|e| anyhow::anyhow!("Failed to acquire settings lock: {}", e))?;
    *global_settings = settings;
    Ok(())
}

/// Get a read-only reference to the current settings.
///
/// # Panics
///
/// Panics if the settings lock is poisoned.
pub fn get_settings() -> impl std::ops::Deref<Target = Settings> {
    SETTINGS.read().expect("Settings lock poisoned")
}

/// Load settings from all sources in order of priority.
///
/// Settings are loaded from:
/// 1. Default values
/// 2. Settings file (~/.config/oops/config.toml or fallback to ~/.config/thefuck/)
/// 3. Environment variables (THEFUCK_* for backward compatibility)
/// 4. CLI arguments
///
/// Later sources override earlier ones.
pub fn load_settings(cli_args: &Cli) -> Result<Settings> {
    // Start with defaults
    let mut settings = Settings::default();
    debug!("Starting with default settings");

    // Load from config file if it exists
    let config_path = get_settings_path();
    if config_path.exists() {
        debug!("Loading settings from: {}", config_path.display());
        match load_from_file(&config_path) {
            Ok(file_settings) => {
                settings.merge(&file_settings);
                debug!("Merged settings from config file");
            }
            Err(e) => {
                warn!("Failed to load config file: {}", e);
            }
        }
    } else {
        debug!("Config file not found at: {}", config_path.display());
    }

    // Override with environment variables
    let env_settings = load_from_env();
    settings.merge(&env_settings);
    debug!("Applied environment variable overrides");

    // Override with CLI arguments
    apply_cli_args(&mut settings, cli_args);
    debug!("Applied CLI argument overrides");

    Ok(settings)
}

/// Get the path to the settings file.
///
/// Returns the settings file path in the oops config directory.
/// Falls back to thefuck config directory for migration support.
pub fn get_settings_path() -> PathBuf {
    get_config_dir().join("settings.toml")
}

/// Get the path to the rules directory.
///
/// Returns the rules directory path in the oops config directory.
pub fn get_rules_dir() -> PathBuf {
    get_config_dir().join("rules")
}

/// Get the oops configuration directory.
///
/// Returns the oops config directory path. Currently uses ~/.config/thefuck/
/// for backward compatibility with existing configurations.
pub fn get_config_dir() -> PathBuf {
    // Try XDG config directory first
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("thefuck");
    }

    // Fallback to home directory
    if let Some(home_dir) = dirs::home_dir() {
        return home_dir.join(".config").join("thefuck");
    }

    // Last resort: current directory
    PathBuf::from(".thefuck")
}

/// Ensure the configuration directory exists.
///
/// Creates the directory and any parent directories if they don't exist.
pub fn ensure_config_dir() -> Result<PathBuf> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory: {}", config_dir.display()))?;
    }
    Ok(config_dir)
}

/// Ensure the rules directory exists.
///
/// Creates the directory and any parent directories if they don't exist.
pub fn ensure_rules_dir() -> Result<PathBuf> {
    let rules_dir = get_rules_dir();
    if !rules_dir.exists() {
        fs::create_dir_all(&rules_dir)
            .with_context(|| format!("Failed to create rules directory: {}", rules_dir.display()))?;
    }
    Ok(rules_dir)
}

/// Load settings from a TOML file.
fn load_from_file(path: &PathBuf) -> Result<Settings> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let settings: Settings = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(settings)
}

/// Load settings from environment variables.
///
/// Supported environment variables:
/// - `THEFUCK_RULES`: colon-separated list of rules
/// - `THEFUCK_EXCLUDE_RULES`: colon-separated list of rules to exclude
/// - `THEFUCK_PRIORITY`: format "rule=num:rule=num"
/// - `THEFUCK_REQUIRE_CONFIRMATION`: "true" or "false"
/// - `THEFUCK_WAIT_COMMAND`: integer (seconds)
/// - `THEFUCK_WAIT_SLOW_COMMAND`: integer (seconds)
/// - `THEFUCK_NO_COLORS`: "true" or "false"
/// - `THEFUCK_HISTORY_LIMIT`: integer
/// - `THEFUCK_ALTER_HISTORY`: "true" or "false"
/// - `THEFUCK_NUM_CLOSE_MATCHES`: integer
/// - `THEFUCK_INSTANT_MODE`: "true" or "false"
/// - `THEFUCK_DEBUG`: "true" or "false"
fn load_from_env() -> Settings {
    let mut settings = Settings::default();

    // THEFUCK_RULES: colon-separated list
    if let Ok(rules) = env::var("THEFUCK_RULES") {
        settings.rules = parse_colon_separated(&rules);
        debug!("THEFUCK_RULES: {:?}", settings.rules);
    }

    // THEFUCK_EXCLUDE_RULES: colon-separated list
    if let Ok(exclude_rules) = env::var("THEFUCK_EXCLUDE_RULES") {
        settings.exclude_rules = parse_colon_separated(&exclude_rules);
        debug!("THEFUCK_EXCLUDE_RULES: {:?}", settings.exclude_rules);
    }

    // THEFUCK_PRIORITY: format "rule=num:rule=num"
    if let Ok(priority_str) = env::var("THEFUCK_PRIORITY") {
        settings.priority = parse_priority(&priority_str);
        debug!("THEFUCK_PRIORITY: {:?}", settings.priority);
    }

    // THEFUCK_REQUIRE_CONFIRMATION: "true" or "false"
    if let Ok(value) = env::var("THEFUCK_REQUIRE_CONFIRMATION") {
        settings.require_confirmation = parse_bool(&value, true);
        debug!("THEFUCK_REQUIRE_CONFIRMATION: {}", settings.require_confirmation);
    }

    // THEFUCK_WAIT_COMMAND: integer (seconds)
    if let Ok(value) = env::var("THEFUCK_WAIT_COMMAND") {
        if let Ok(secs) = value.parse::<u64>() {
            settings.wait_command = secs;
            debug!("THEFUCK_WAIT_COMMAND: {}", settings.wait_command);
        } else {
            warn!("Invalid THEFUCK_WAIT_COMMAND value: {}", value);
        }
    }

    // THEFUCK_WAIT_SLOW_COMMAND: integer (seconds)
    if let Ok(value) = env::var("THEFUCK_WAIT_SLOW_COMMAND") {
        if let Ok(secs) = value.parse::<u64>() {
            settings.wait_slow_command = secs;
            debug!("THEFUCK_WAIT_SLOW_COMMAND: {}", settings.wait_slow_command);
        } else {
            warn!("Invalid THEFUCK_WAIT_SLOW_COMMAND value: {}", value);
        }
    }

    // THEFUCK_NO_COLORS: "true" or "false"
    if let Ok(value) = env::var("THEFUCK_NO_COLORS") {
        settings.no_colors = parse_bool(&value, false);
        debug!("THEFUCK_NO_COLORS: {}", settings.no_colors);
    }

    // THEFUCK_HISTORY_LIMIT: integer
    if let Ok(value) = env::var("THEFUCK_HISTORY_LIMIT") {
        if let Ok(limit) = value.parse::<usize>() {
            settings.history_limit = Some(limit);
            debug!("THEFUCK_HISTORY_LIMIT: {:?}", settings.history_limit);
        } else {
            warn!("Invalid THEFUCK_HISTORY_LIMIT value: {}", value);
        }
    }

    // THEFUCK_ALTER_HISTORY: "true" or "false"
    if let Ok(value) = env::var("THEFUCK_ALTER_HISTORY") {
        settings.alter_history = parse_bool(&value, true);
        debug!("THEFUCK_ALTER_HISTORY: {}", settings.alter_history);
    }

    // THEFUCK_NUM_CLOSE_MATCHES: integer
    if let Ok(value) = env::var("THEFUCK_NUM_CLOSE_MATCHES") {
        if let Ok(num) = value.parse::<usize>() {
            settings.num_close_matches = num;
            debug!("THEFUCK_NUM_CLOSE_MATCHES: {}", settings.num_close_matches);
        } else {
            warn!("Invalid THEFUCK_NUM_CLOSE_MATCHES value: {}", value);
        }
    }

    // THEFUCK_SLOW_COMMANDS: colon-separated list
    if let Ok(slow_commands) = env::var("THEFUCK_SLOW_COMMANDS") {
        settings.slow_commands = parse_colon_separated(&slow_commands);
        debug!("THEFUCK_SLOW_COMMANDS: {:?}", settings.slow_commands);
    }

    // THEFUCK_EXCLUDED_SEARCH_PATH_PREFIXES: colon-separated list
    if let Ok(prefixes) = env::var("THEFUCK_EXCLUDED_SEARCH_PATH_PREFIXES") {
        settings.excluded_search_path_prefixes = parse_colon_separated(&prefixes);
        debug!(
            "THEFUCK_EXCLUDED_SEARCH_PATH_PREFIXES: {:?}",
            settings.excluded_search_path_prefixes
        );
    }

    // THEFUCK_INSTANT_MODE: "true" or "false"
    if let Ok(value) = env::var("THEFUCK_INSTANT_MODE") {
        settings.instant_mode = parse_bool(&value, false);
        debug!("THEFUCK_INSTANT_MODE: {}", settings.instant_mode);
    }

    // THEFUCK_DEBUG: "true" or "false"
    if let Ok(value) = env::var("THEFUCK_DEBUG") {
        settings.debug = parse_bool(&value, false);
        debug!("THEFUCK_DEBUG: {}", settings.debug);
    }

    settings
}

/// Apply CLI arguments to settings.
///
/// CLI arguments have the highest priority and override all other sources.
fn apply_cli_args(settings: &mut Settings, cli_args: &Cli) {
    // --yes / -y: disable confirmation
    if cli_args.yes {
        settings.require_confirmation = false;
        debug!("CLI: require_confirmation = false (--yes)");
    }

    // --debug / -d: enable debug output
    if cli_args.debug {
        settings.debug = true;
        debug!("CLI: debug = true (--debug)");
    }

    // --enable-experimental-instant-mode: enable instant mode
    if cli_args.instant_mode {
        settings.instant_mode = true;
        debug!("CLI: instant_mode = true (--enable-experimental-instant-mode)");
    }
}

/// Parse a colon-separated string into a vector of strings.
///
/// Empty strings are filtered out.
fn parse_colon_separated(value: &str) -> Vec<String> {
    value
        .split(':')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse a priority string in the format "rule=num:rule=num".
fn parse_priority(value: &str) -> HashMap<String, i32> {
    let mut priority = HashMap::new();

    for pair in value.split(':') {
        let parts: Vec<&str> = pair.split('=').collect();
        if parts.len() == 2 {
            let rule_name = parts[0].trim();
            if let Ok(priority_value) = parts[1].trim().parse::<i32>() {
                priority.insert(rule_name.to_string(), priority_value);
            } else {
                warn!("Invalid priority value for rule '{}': {}", rule_name, parts[1]);
            }
        }
    }

    priority
}

/// Parse a boolean string value.
///
/// Recognizes "true", "1", "yes", "on" as true, and "false", "0", "no", "off" as false.
/// Returns the default value if the string doesn't match any known value.
fn parse_bool(value: &str, default: bool) -> bool {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => {
            warn!("Invalid boolean value '{}', using default: {}", value, default);
            default
        }
    }
}

/// Create a default settings file if it doesn't exist.
///
/// This is useful for first-time setup.
pub fn create_default_settings_file() -> Result<PathBuf> {
    let config_dir = ensure_config_dir()?;
    let settings_path = config_dir.join("settings.toml");

    if !settings_path.exists() {
        let default_settings = Settings::default();
        let toml_content = toml::to_string_pretty(&default_settings)
            .context("Failed to serialize default settings")?;

        let header = r#"# oops Configuration File
# For more information, see: https://github.com/anthropics/oops

"#;

        fs::write(&settings_path, format!("{}{}", header, toml_content))
            .with_context(|| format!("Failed to write settings file: {}", settings_path.display()))?;

        debug!("Created default settings file at: {}", settings_path.display());
    }

    Ok(settings_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::env;
    use std::sync::Mutex;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
    const ENV_VARS: &[&str] = &[
        "THEFUCK_RULES",
        "THEFUCK_EXCLUDE_RULES",
        "THEFUCK_PRIORITY",
        "THEFUCK_REQUIRE_CONFIRMATION",
        "THEFUCK_WAIT_COMMAND",
        "THEFUCK_WAIT_SLOW_COMMAND",
        "THEFUCK_NO_COLORS",
        "THEFUCK_HISTORY_LIMIT",
        "THEFUCK_ALTER_HISTORY",
        "THEFUCK_NUM_CLOSE_MATCHES",
        "THEFUCK_SLOW_COMMANDS",
        "THEFUCK_EXCLUDED_SEARCH_PATH_PREFIXES",
        "THEFUCK_INSTANT_MODE",
        "THEFUCK_DEBUG",
    ];

    fn clear_env_vars() {
        for var in ENV_VARS {
            env::remove_var(var);
        }
    }

    #[test]
    fn test_parse_colon_separated() {
        assert_eq!(
            parse_colon_separated("git_push:sudo:cd_mkdir"),
            vec!["git_push", "sudo", "cd_mkdir"]
        );
        assert_eq!(parse_colon_separated("single"), vec!["single"]);
        assert_eq!(parse_colon_separated(""), Vec::<String>::new());
        assert_eq!(
            parse_colon_separated(" spaced : values "),
            vec!["spaced", "values"]
        );
    }

    #[test]
    fn test_parse_priority() {
        let priority = parse_priority("sudo=100:git_push=500");
        assert_eq!(priority.get("sudo"), Some(&100));
        assert_eq!(priority.get("git_push"), Some(&500));
    }

    #[test]
    fn test_parse_priority_empty() {
        let priority = parse_priority("");
        assert!(priority.is_empty());
    }

    #[test]
    fn test_parse_priority_invalid() {
        let priority = parse_priority("sudo=abc:git_push=500");
        assert!(priority.get("sudo").is_none());
        assert_eq!(priority.get("git_push"), Some(&500));
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("true", false));
        assert!(parse_bool("1", false));
        assert!(parse_bool("yes", false));
        assert!(parse_bool("on", false));
        assert!(parse_bool("TRUE", false));
        assert!(parse_bool("True", false));

        assert!(!parse_bool("false", true));
        assert!(!parse_bool("0", true));
        assert!(!parse_bool("no", true));
        assert!(!parse_bool("off", true));
        assert!(!parse_bool("FALSE", true));

        // Invalid values should return default
        assert!(parse_bool("invalid", true));
        assert!(!parse_bool("invalid", false));
    }

    #[test]
    fn test_get_config_dir() {
        let config_dir = get_config_dir();
        assert!(config_dir.ends_with("thefuck"));
    }

    #[test]
    fn test_get_settings_path() {
        let settings_path = get_settings_path();
        assert!(settings_path.ends_with("settings.toml"));
    }

    #[test]
    fn test_get_rules_dir() {
        let rules_dir = get_rules_dir();
        assert!(rules_dir.ends_with("rules"));
    }

    #[test]
    fn test_load_from_env_rules() {
        let _guard = ENV_LOCK.lock().expect("Failed to lock env mutex");
        clear_env_vars();
        // Set environment variable
        env::set_var("THEFUCK_RULES", "sudo:git_push");

        let settings = load_from_env();
        assert_eq!(settings.rules, vec!["sudo", "git_push"]);

        // Clean up
        clear_env_vars();
    }

    #[test]
    fn test_load_from_env_debug() {
        let _guard = ENV_LOCK.lock().expect("Failed to lock env mutex");
        clear_env_vars();
        env::set_var("THEFUCK_DEBUG", "true");

        let settings = load_from_env();
        assert!(settings.debug);

        clear_env_vars();
    }

    #[test]
    fn test_load_from_env_wait_command() {
        let _guard = ENV_LOCK.lock().expect("Failed to lock env mutex");
        clear_env_vars();
        env::set_var("THEFUCK_WAIT_COMMAND", "10");

        let settings = load_from_env();
        assert_eq!(settings.wait_command, 10);

        clear_env_vars();
    }

    #[test]
    fn test_apply_cli_args() {
        let mut settings = Settings::default();
        let cli = Cli {
            alias: false,
            yes: true,
            repeat: false,
            debug: true,
            instant_mode: true,
            force_command: None,
            shell_logger: None,
            command: vec![],
        };

        apply_cli_args(&mut settings, &cli);

        assert!(!settings.require_confirmation);
        assert!(settings.debug);
        assert!(settings.instant_mode);
    }

    #[test]
    fn test_load_settings_with_defaults() {
        let _guard = ENV_LOCK.lock().expect("Failed to lock env mutex");
        clear_env_vars();
        let cli = Cli {
            alias: false,
            yes: false,
            repeat: false,
            debug: false,
            instant_mode: false,
            force_command: None,
            shell_logger: None,
            command: vec![],
        };

        let settings = load_settings(&cli).unwrap();

        // Should have default values
        assert_eq!(settings.rules, vec!["ALL"]);
        assert!(settings.require_confirmation);
        assert_eq!(settings.wait_command, 3);
    }
}
