//! oops - A blazingly fast command-line typo corrector
//!
//! This is a Rust rewrite inspired by the original Python thefuck project.
//! It provides faster startup time while maintaining full feature parity.

use anyhow::Result;
use tracing::debug;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use oops::cli::Cli;
use oops::{core, shells};

fn main() -> Result<()> {
    // Parse CLI arguments first to check for debug flag
    let cli = Cli::parse_with_placeholder();

    // Initialize logging based on debug flag or THEFUCK_DEBUG env var
    init_logging(cli.debug);

    debug!("oops starting with args: {:?}", cli);

    // Dispatch to appropriate command
    if cli.alias {
        // Generate shell alias
        handle_alias()?;
    } else if let Some(ref logger_file) = cli.shell_logger {
        // Shell logger mode (internal use)
        handle_shell_logger(logger_file)?;
    } else {
        // Default: fix command
        handle_fix_command(&cli)?;
    }

    Ok(())
}

/// Initialize the tracing subscriber for logging.
fn init_logging(debug_enabled: bool) {
    let filter = if debug_enabled {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_env("THEFUCK_DEBUG").unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();
}

/// Handle the --alias flag to generate shell alias.
fn handle_alias() -> Result<()> {
    debug!("Generating shell alias");
    shells::generate_alias()
}

/// Handle the shell logger mode (internal use by shell integration).
fn handle_shell_logger(logger_file: &str) -> Result<()> {
    debug!("Shell logger mode: {}", logger_file);
    shells::run_shell_logger(logger_file)
}

/// Handle the default fix command mode.
fn handle_fix_command(cli: &Cli) -> Result<()> {
    debug!("Fix command mode");

    // If force-command is specified, use that instead of detecting
    let command = if let Some(ref forced) = cli.force_command {
        debug!("Using forced command: {}", forced);
        Some(forced.clone())
    } else {
        cli.get_command_string()
    };

    // Create fix options from CLI args
    let options = core::FixOptions {
        yes: cli.yes,
        repeat: cli.repeat,
        instant_mode: cli.instant_mode,
    };

    core::fix_command(command.as_deref(), &options)
}
