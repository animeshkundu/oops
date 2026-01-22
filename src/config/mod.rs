//! Configuration module for oops.
//!
//! This module provides the configuration system for oops, including:
//! - `Settings`: The main configuration struct with all settings
//! - `loader`: Functions for loading settings from files and environment
//!
//! # Configuration Priority
//!
//! Settings are loaded from multiple sources in order of priority (later sources override earlier):
//! 1. Default values
//! 2. Settings file (`~/.config/oops/config.toml`)
//! 3. Environment variables (`THEFUCK_*`)
//! 4. CLI arguments
//!
//! # Example
//!
//! ```rust,ignore
//! use oops::config::{load_settings, get_settings, init_settings};
//! use oops::cli::Cli;
//!
//! // Parse CLI arguments
//! let cli = Cli::parse();
//!
//! // Initialize global settings
//! init_settings(&cli).expect("Failed to load settings");
//!
//! // Access settings anywhere in the application
//! let settings = get_settings();
//! if settings.debug {
//!     println!("Debug mode enabled");
//! }
//! ```
//!
//! # Environment Variables
//!
//! The following environment variables are supported:
//!
//! | Variable | Type | Description |
//! |----------|------|-------------|
//! | `THEFUCK_RULES` | colon-separated list | Enabled rules (e.g., `sudo:git_push`) |
//! | `THEFUCK_EXCLUDE_RULES` | colon-separated list | Rules to exclude |
//! | `THEFUCK_PRIORITY` | rule=num:rule=num | Rule priorities (e.g., `sudo=100:git_push=500`) |
//! | `THEFUCK_REQUIRE_CONFIRMATION` | true/false | Require confirmation before executing |
//! | `THEFUCK_WAIT_COMMAND` | integer | Timeout for normal commands (seconds) |
//! | `THEFUCK_WAIT_SLOW_COMMAND` | integer | Timeout for slow commands (seconds) |
//! | `THEFUCK_NO_COLORS` | true/false | Disable colored output |
//! | `THEFUCK_HISTORY_LIMIT` | integer | Maximum history entries to search |
//! | `THEFUCK_ALTER_HISTORY` | true/false | Alter shell history when fixing |
//! | `THEFUCK_NUM_CLOSE_MATCHES` | integer | Number of suggestions to show |
//! | `THEFUCK_SLOW_COMMANDS` | colon-separated list | Commands with longer timeout |
//! | `THEFUCK_INSTANT_MODE` | true/false | Enable instant mode |
//! | `THEFUCK_DEBUG` | true/false | Enable debug output |

mod loader;
mod settings;

// Re-export main types and functions
pub use loader::{
    create_default_settings_file, ensure_config_dir, ensure_rules_dir, get_config_dir,
    get_rules_dir, get_settings, get_settings_path, init_settings, load_settings, SETTINGS,
};
pub use settings::Settings;
