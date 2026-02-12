use crossterm::cursor::Show;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::{backend::CrosstermBackend};
use ratatui::Terminal;
use std::io::{self, Stdout};

use crate::tui::{PickerItem, pickers::{
    ConfirmPicker, SelectPicker, TextPicker, TextPickerWithSuggestions,
}};

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
    restored: bool,
}

impl PickerCtx {
    /// Create a new picker context and initialize the terminal.
    ///
    /// This sets up the terminal in raw mode, clears the screen,
    /// and creates the necessary backend for rendering TUI components.
    pub fn new() -> io::Result<Self> {
        use crossterm::{
            cursor::Hide,
            execute,
        };

        enable_raw_mode()?;

        // Switch to an alternate screen before creating terminal
        // This ensures any previous terminal content is hidden, and restored once complete
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        Ok(Self { terminal, restored: false })
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
    pub fn select<T: PickerItem>(&mut self, items: &[T], _prompt: &str) -> Option<T> {
        // Clone items to own them
        let mut picker = SelectPicker::new(items.to_vec());
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
        // Show cursor
        execute!(self.terminal.backend_mut(), Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        self.restored = true;
        Ok(())
    }
}

impl Drop for PickerCtx {
    fn drop(&mut self) {
        if !self.restored {
            let _ = self.restore();
        }
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

/// A suggestion provider that sources tags from existing workspaces.
///
/// This provider queries the workspace repository to collect all unique tags
/// across workspaces and provides them as suggestions.
pub struct ExistingTagsSuggestionProvider {
    /// The list of all unique tags collected from workspaces
    all_tags: Vec<String>,
}

impl ExistingTagsSuggestionProvider {
    /// Create a new provider with the given workspace repository.
    ///
    /// This will collect all unique tags from existing workspaces.
    pub fn new(all_tags: Vec<String>) -> Self {
        Self { all_tags }
    }
}

impl SuggestionProvider for ExistingTagsSuggestionProvider {
    fn suggestions(&self, input: &str) -> Option<Vec<String>> {
        if input.is_empty() {
            return Some(self.all_tags.to_vec());
        }

        // Return fuzzy matches from all workspace tags
        let matches: Vec<String> = self
            .all_tags
            .iter()
            .filter(|tag| tag.to_lowercase().contains(&input.to_lowercase()))
            .cloned()
            .collect();

        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }
}
