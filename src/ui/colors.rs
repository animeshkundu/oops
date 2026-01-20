//! Colored output utilities
//!
//! Provides functions for printing colored text to the terminal using crossterm.

use std::io::{self, Write};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetAttribute, SetForegroundColor, Attribute},
};

/// Print a command script with syntax highlighting.
///
/// The command is printed in cyan/bold to make it stand out.
///
/// # Arguments
///
/// * `script` - The command script to print
pub fn print_command(script: &str) {
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print(script),
        SetAttribute(Attribute::Reset),
        ResetColor,
        Print("\n"),
    );
}

/// Print an error message in red.
///
/// # Arguments
///
/// * `message` - The error message to print
pub fn print_error(message: &str) {
    let mut stderr = io::stderr();
    let _ = execute!(
        stderr,
        SetForegroundColor(Color::Red),
        SetAttribute(Attribute::Bold),
        Print("error: "),
        SetAttribute(Attribute::Reset),
        SetForegroundColor(Color::Red),
        Print(message),
        ResetColor,
        Print("\n"),
    );
}

/// Print a debug message in dark gray.
///
/// Debug messages are only printed when debug mode is enabled.
///
/// # Arguments
///
/// * `message` - The debug message to print
pub fn print_debug(message: &str) {
    let mut stderr = io::stderr();
    let _ = execute!(
        stderr,
        SetForegroundColor(Color::DarkGrey),
        Print("[debug] "),
        Print(message),
        ResetColor,
        Print("\n"),
    );
}

/// Format a suggestion for display in the selection UI.
///
/// Selected suggestions are shown in green with bold text,
/// while unselected suggestions are shown in normal white.
///
/// # Arguments
///
/// * `script` - The command script to format
/// * `is_selected` - Whether this suggestion is currently selected
///
/// # Returns
///
/// A formatted string with ANSI color codes
pub fn format_suggestion(script: &str, is_selected: bool) -> String {
    if is_selected {
        format!(
            "\x1b[1;32m{}\x1b[0m", // Bold green
            script
        )
    } else {
        format!(
            "\x1b[37m{}\x1b[0m", // White/gray
            script
        )
    }
}

/// Print a success message in green.
///
/// # Arguments
///
/// * `message` - The success message to print
pub fn print_success(message: &str) {
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        SetForegroundColor(Color::Green),
        Print(message),
        ResetColor,
        Print("\n"),
    );
}

/// Print a warning message in yellow.
///
/// # Arguments
///
/// * `message` - The warning message to print
pub fn print_warning(message: &str) {
    let mut stderr = io::stderr();
    let _ = execute!(
        stderr,
        SetForegroundColor(Color::Yellow),
        SetAttribute(Attribute::Bold),
        Print("warning: "),
        SetAttribute(Attribute::Reset),
        SetForegroundColor(Color::Yellow),
        Print(message),
        ResetColor,
        Print("\n"),
    );
}

/// Print info text in blue.
///
/// # Arguments
///
/// * `message` - The info message to print
pub fn print_info(message: &str) {
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        SetForegroundColor(Color::Blue),
        Print(message),
        ResetColor,
        Print("\n"),
    );
}

/// Check if the terminal supports colors.
///
/// This checks for common environment variables that indicate color support
/// and whether stdout is connected to a terminal.
///
/// # Returns
///
/// `true` if the terminal likely supports colors, `false` otherwise
pub fn supports_color() -> bool {
    // Check for explicit NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check for TERM=dumb
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check if stdout is a terminal using crossterm's tty detection
    crossterm::tty::IsTty::is_tty(&io::stdout())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_suggestion_selected() {
        let result = format_suggestion("ls -la", true);
        assert!(result.contains("ls -la"));
        assert!(result.contains("\x1b[1;32m")); // Bold green
        assert!(result.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn test_format_suggestion_unselected() {
        let result = format_suggestion("ls -la", false);
        assert!(result.contains("ls -la"));
        assert!(result.contains("\x1b[37m")); // White
        assert!(result.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn test_format_suggestion_with_special_chars() {
        let script = "echo \"hello world\" | grep 'test'";
        let result = format_suggestion(script, true);
        assert!(result.contains(script));
    }

    #[test]
    fn test_format_suggestion_empty_script() {
        let result = format_suggestion("", true);
        assert!(result.contains("\x1b[1;32m"));
        assert!(result.contains("\x1b[0m"));
    }
}
