//! UI module for oops
//!
//! This module provides terminal UI components including command selection
//! and colored output utilities.

pub mod colors;
pub mod selector;

pub use colors::{
    format_suggestion, print_command, print_debug, print_error, print_info, print_success,
    print_warning, supports_color,
};
pub use selector::CommandSelector;
