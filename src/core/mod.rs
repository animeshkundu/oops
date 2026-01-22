//! Core types and functionality for oops.
//!
//! This module contains the fundamental types used throughout the application:
//! - [`Command`] - Represents a failed command with its output
//! - [`Rule`] - Trait for correction rules
//! - [`CorrectedCommand`] - A suggested correction for a failed command
//! - Corrector functions for matching rules and generating corrections

mod command;
mod corrected;
mod corrector;
mod rule;

pub use command::Command;
pub use corrected::{CorrectedCommand, SideEffect};
pub use corrector::{get_best_correction, get_corrected_commands, get_rules, match_rule};
pub use rule::{for_app, is_app, ForAppRule, Rule};

use anyhow::Result;

/// Options for the fix command operation.
#[derive(Debug, Clone, Default)]
pub struct FixOptions {
    /// Automatically confirm the first suggestion without prompting.
    pub yes: bool,
    /// Keep running the corrected command until it succeeds.
    pub repeat: bool,
    /// Enable instant mode for faster corrections.
    pub instant_mode: bool,
}

/// Main entry point for fixing a failed command.
///
/// This function is called when the user runs `oops` to fix their last command.
/// It loads the command from history or the provided string, matches it against
/// all available rules, and presents corrections to the user.
///
/// # Arguments
///
/// * `command_str` - Optional command string. If None, will be loaded from history.
/// * `options` - Options controlling the fix behavior.
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the fix operation fails.
pub fn fix_command(command_str: Option<&str>, options: &FixOptions) -> Result<()> {
    use tracing::debug;

    // Get settings
    let settings = crate::config::get_settings();

    // Get the command to fix
    let timeout = std::time::Duration::from_secs(settings.wait_command as u64);

    let command = if let Some(cmd_str) = command_str {
        debug!("Using provided command: {}", cmd_str);
        // Re-execute the command to get its output
        let output = crate::output::get_output(cmd_str, timeout).unwrap_or_default();
        debug!("Got output: {}", output);
        Command::new(cmd_str, output)
    } else {
        // Try to get command from environment (set by shell integration)
        let script = std::env::var("TF_HISTORY")
            .or_else(|_| std::env::var("THEFUCK_HISTORY"))
            .unwrap_or_default();

        if script.is_empty() {
            anyhow::bail!("No command to fix. Set up shell integration or provide a command.");
        }

        debug!("Got command from history: {}", script);
        // Re-execute the command to get its output
        let output = crate::output::get_output(&script, timeout).unwrap_or_default();
        debug!("Got output: {}", output);
        Command::new(script, output)
    };

    // Get corrections
    let corrections = get_corrected_commands(&command, &settings);

    if corrections.is_empty() {
        println!("No corrections available for: {}", command.script);
        return Ok(());
    }

    debug!("Found {} corrections", corrections.len());

    // If --yes flag is set, run the first correction automatically
    if options.yes {
        let correction = &corrections[0];
        println!("{}", correction.script);

        if !options.instant_mode {
            correction.run(&command, &settings)?;
        }
        return Ok(());
    }

    // Otherwise, use the UI to let the user select a correction
    // For now, just print the corrections
    println!("Suggestions:");
    for (i, correction) in corrections.iter().enumerate() {
        println!("  {}: {}", i + 1, correction.script);
    }

    // In a full implementation, we'd use the UI module for interactive selection
    // For now, just run the first correction
    if !corrections.is_empty() {
        let correction = &corrections[0];
        correction.run(&command, &settings)?;
    }

    Ok(())
}
