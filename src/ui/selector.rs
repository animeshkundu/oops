//! Command selection UI
//!
//! Provides an interactive terminal UI for selecting from multiple corrected commands.

use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::core::CorrectedCommand;
use crate::ui::colors;

/// Interactive command selector for choosing from multiple correction options.
///
/// Displays a list of corrected commands and allows the user to navigate
/// and select one using keyboard input.
#[derive(Debug)]
pub struct CommandSelector {
    /// List of corrected commands to select from
    commands: Vec<CorrectedCommand>,
    /// Currently selected index
    selected: usize,
}

impl CommandSelector {
    /// Create a new command selector with the given commands.
    ///
    /// # Arguments
    ///
    /// * `commands` - Vector of corrected commands to display
    ///
    /// # Returns
    ///
    /// A new `CommandSelector` with the first item selected
    pub fn new(commands: Vec<CorrectedCommand>) -> Self {
        Self {
            commands,
            selected: 0,
        }
    }

    /// Display selection UI and return the chosen command.
    ///
    /// Handles keyboard input for navigation:
    /// - Arrow Up/Down, j/k, Ctrl+N/Ctrl+P to navigate
    /// - Enter to select
    /// - Ctrl+C or Escape to abort
    ///
    /// # Returns
    ///
    /// * `Some(&CorrectedCommand)` - The selected command
    /// * `None` - If user pressed Ctrl+C or Escape to abort
    pub fn select(&mut self) -> Option<&CorrectedCommand> {
        if self.commands.is_empty() {
            return None;
        }

        // If only one command, return it directly without UI
        if self.commands.len() == 1 {
            return self.commands.first();
        }

        // Enable raw mode for keyboard input
        if terminal::enable_raw_mode().is_err() {
            // If raw mode fails, just return the first command
            return self.commands.first();
        }

        let result = self.run_selection_loop();

        // Always disable raw mode before returning
        let _ = terminal::disable_raw_mode();

        result
    }

    /// Run the main selection loop.
    fn run_selection_loop(&mut self) -> Option<&CorrectedCommand> {
        let mut stdout = io::stdout();

        // Initial render
        if self.render(&mut stdout).is_err() {
            return self.commands.first();
        }

        loop {
            // Wait for keyboard event
            let event = match event::read() {
                Ok(e) => e,
                Err(_) => return self.commands.first(),
            };

            match event {
                Event::Key(key_event) => {
                    match (key_event.code, key_event.modifiers) {
                        // Abort on Ctrl+C or Escape
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Esc, _) => {
                            self.clear_ui(&mut stdout);
                            return None;
                        }

                        // Select on Enter
                        (KeyCode::Enter, _) => {
                            self.clear_ui(&mut stdout);
                            return self.commands.get(self.selected);
                        }

                        // Move up: Arrow Up, k, or Ctrl+P
                        (KeyCode::Up, _)
                        | (KeyCode::Char('k'), KeyModifiers::NONE)
                        | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                            self.move_up();
                            let _ = self.render(&mut stdout);
                        }

                        // Move down: Arrow Down, j, or Ctrl+N
                        (KeyCode::Down, _)
                        | (KeyCode::Char('j'), KeyModifiers::NONE)
                        | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                            self.move_down();
                            let _ = self.render(&mut stdout);
                        }

                        // Jump to first item with Home or g
                        (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
                            self.selected = 0;
                            let _ = self.render(&mut stdout);
                        }

                        // Jump to last item with End or G
                        (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::NONE) => {
                            self.selected = self.commands.len().saturating_sub(1);
                            let _ = self.render(&mut stdout);
                        }

                        _ => {}
                    }
                }
                Event::Resize(_, _) => {
                    // Re-render on terminal resize
                    let _ = self.render(&mut stdout);
                }
                _ => {}
            }
        }
    }

    /// Move selection up by one.
    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            // Wrap to bottom
            self.selected = self.commands.len().saturating_sub(1);
        }
    }

    /// Move selection down by one.
    fn move_down(&mut self) {
        if self.selected < self.commands.len().saturating_sub(1) {
            self.selected += 1;
        } else {
            // Wrap to top
            self.selected = 0;
        }
    }

    /// Render the selection UI to the terminal.
    fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Move cursor to start position and clear
        queue!(
            stdout,
            cursor::SavePosition,
            terminal::Clear(ClearType::FromCursorDown),
        )?;

        // Print header
        queue!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Select a command (use arrows/j/k to navigate, Enter to select, Ctrl+C to abort):\n\r"),
            ResetColor,
        )?;

        // Print each command option
        for (i, cmd) in self.commands.iter().enumerate() {
            let is_selected = i == self.selected;
            let formatted = colors::format_suggestion(&cmd.script, is_selected);

            if is_selected {
                queue!(stdout, Print(format!("  > {}\n\r", formatted)))?;
            } else {
                queue!(stdout, Print(format!("    {}\n\r", formatted)))?;
            }
        }

        stdout.flush()
    }

    /// Clear the UI from the terminal.
    fn clear_ui(&self, stdout: &mut io::Stdout) {
        let _ = execute!(
            stdout,
            cursor::RestorePosition,
            terminal::Clear(ClearType::FromCursorDown),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_command(script: &str) -> CorrectedCommand {
        CorrectedCommand {
            script: script.to_string(),
            priority: 1000,
            side_effect: None,
        }
    }

    #[test]
    fn test_selector_new() {
        let commands = vec![make_command("ls -la"), make_command("ls -l")];
        let selector = CommandSelector::new(commands);

        assert_eq!(selector.selected, 0);
        assert_eq!(selector.commands.len(), 2);
    }

    #[test]
    fn test_selector_empty_returns_none() {
        let mut selector = CommandSelector::new(vec![]);
        assert!(selector.select().is_none());
    }

    #[test]
    fn test_move_up_wraps() {
        let commands = vec![make_command("a"), make_command("b"), make_command("c")];
        let mut selector = CommandSelector::new(commands);

        assert_eq!(selector.selected, 0);
        selector.move_up();
        assert_eq!(selector.selected, 2); // Wrapped to end
    }

    #[test]
    fn test_move_down_wraps() {
        let commands = vec![make_command("a"), make_command("b"), make_command("c")];
        let mut selector = CommandSelector::new(commands);

        selector.selected = 2;
        selector.move_down();
        assert_eq!(selector.selected, 0); // Wrapped to start
    }

    #[test]
    fn test_move_up_normal() {
        let commands = vec![make_command("a"), make_command("b"), make_command("c")];
        let mut selector = CommandSelector::new(commands);

        selector.selected = 2;
        selector.move_up();
        assert_eq!(selector.selected, 1);
    }

    #[test]
    fn test_move_down_normal() {
        let commands = vec![make_command("a"), make_command("b"), make_command("c")];
        let mut selector = CommandSelector::new(commands);

        selector.selected = 0;
        selector.move_down();
        assert_eq!(selector.selected, 1);
    }
}
