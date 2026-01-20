//! Output module for oops
//!
//! This module handles command re-execution and output capture.
//!
//! Handles:
//! - Running commands and capturing output
//! - Executing corrected commands
//! - Output parsing and formatting
//! - Timeout handling for slow commands

pub mod rerun;

pub use rerun::{
    execute_command, execute_interactive, get_output, get_output_with_slow_handling,
    is_slow_command,
};
