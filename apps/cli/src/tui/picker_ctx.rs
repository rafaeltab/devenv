use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};

use crate::tui::picker_item::PickerItem;
use crate::tui::pickers::{
    ConfirmPicker, SelectPicker, SimpleItem, TextPicker, TextPickerWithSuggestions,
};

/// Context for running pickers in the terminal.
///
/// `PickerCtx` manages the terminal state and provides methods for
/// displaying various picker types and capturing user input.
///
/// # Example
///
/// ```ignore
/// use rafaeltab::tui::PickerCtx;
///
/// let mut ctx = PickerCtx::new().expect("Failed to create picker context");
///
/// // Run a select picker
/// let items = vec!["Option 1", "Option 2", "Option 3"];
/// let selection = ctx.select(&items, "Choose an option:");
///
/// // Clean up
/// ctx.restore().expect("Failed to restore terminal");
/// ```
pub struct PickerCtx {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl PickerCtx {
    /// Create a new picker context and initialize the terminal.
    ///
    /// This sets up the terminal in raw mode and creates the necessary
    /// backend for rendering TUI components.
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Display a select picker and return the selected item.
    ///
    /// # Arguments
    /// * `items` - The list of items to display
    /// * `prompt` - The prompt text to display
    ///
    /// # Returns
    /// * `Some(SimpleItem)` - The selected item
    /// * `None` - If the user cancels (Esc or Ctrl+C)
    pub fn select(&mut self, items: &[SimpleItem], _prompt: &str) -> Option<SimpleItem> {
        // Clone items to own them
        let simple_items: Vec<SimpleItem> = items.to_vec();
        let mut picker = SelectPicker::new(simple_items);
        picker.run(&mut self.terminal).cloned()
    }

    /// Display a text input picker and return the entered text.
    ///
    /// # Arguments
    /// * `prompt` - The prompt text to display
    ///
    /// # Returns
    /// * `Some(String)` - The entered text
    /// * `None` - If the user cancels (Esc or Ctrl+C)
    pub fn input(&mut self, prompt: &str) -> Option<String> {
        let mut picker = TextPicker::new(prompt);
        picker.run(&mut self.terminal)
    }

    /// Display a text input picker with suggestions.
    ///
    /// # Arguments
    /// * `prompt` - The prompt text to display
    /// * `provider` - A suggestion provider for autocomplete
    ///
    /// # Returns
    /// * `Some(String)` - The entered text
    /// * `None` - If the user cancels (Esc or Ctrl+C)
    pub fn input_with_suggestions(
        &mut self,
        prompt: &str,
        provider: Box<dyn SuggestionProvider>,
    ) -> Option<String> {
        let mut picker = TextPickerWithSuggestions::new(prompt, provider);
        picker.run(&mut self.terminal)
    }

    /// Display a confirm picker and return the user's choice.
    ///
    /// # Arguments
    /// * `prompt` - The prompt text to display
    /// * `default` - The default selection (true for Yes, false for No)
    ///
    /// # Returns
    /// * `Some(true)` - If the user selects Yes
    /// * `Some(false)` - If the user selects No
    /// * `None` - If the user cancels (Esc or Ctrl+C)
    pub fn confirm(&mut self, prompt: &str, default: bool) -> Option<bool> {
        let mut picker = ConfirmPicker::new(prompt).with_default(default);
        picker.run(&mut self.terminal)
    }

    /// Execute a shell command and return to the terminal.
    ///
    /// This temporarily exits the TUI mode to run a command, then
    /// returns control to the shell.
    pub fn execute(&self, command: &str) -> io::Result<()> {
        // For now, just print the command as a placeholder
        // This will be fully implemented later
        println!("Would execute: {}", command);
        Ok(())
    }

    /// Restore the terminal to its original state.
    ///
    /// This should be called before exiting the application to
    /// ensure the terminal is properly cleaned up.
    pub fn restore(&mut self) -> io::Result<()> {
        // Placeholder - will be implemented when we add proper
        // terminal initialization/cleanup
        Ok(())
    }
}

/// Trait for providing suggestions to text pickers.
///
/// Implementors can provide dynamic suggestions based on the current input.
pub trait SuggestionProvider {
    /// Return suggestions for the given input.
    ///
    /// # Arguments
    /// * `input` - The current user input
    ///
    /// # Returns
    /// * `Some(Vec<String>)` - A list of matching suggestions
    /// * `None` - If no suggestions are available
    fn suggestions(&self, input: &str) -> Option<Vec<String>>;
}

/// A simple static suggestion provider.
///
/// This provider returns suggestions from a fixed list based on
/// substring matching.
pub struct StaticSuggestionProvider {
    suggestions: Vec<String>,
}

impl StaticSuggestionProvider {
    /// Create a new provider with the given suggestions.
    pub fn new(suggestions: Vec<String>) -> Self {
        Self { suggestions }
    }
}

impl SuggestionProvider for StaticSuggestionProvider {
    fn suggestions(&self, input: &str) -> Option<Vec<String>> {
        if input.is_empty() {
            return None;
        }

        let matches: Vec<String> = self
            .suggestions
            .iter()
            .filter(|s| s.to_lowercase().contains(&input.to_lowercase()))
            .cloned()
            .collect();

        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }
}
