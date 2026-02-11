use std::env;

use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

use crate::commands::{Command, CommandCtx};
use crate::tui::picker_ctx::StaticSuggestionProvider;
use crate::tui::picker_item::PickerItem;

/// Test command for the text input picker with suggestions.
///
/// This command displays a text input picker with suggestions from
/// TEST_SUGGESTIONS environment variable.
///
/// Environment variables:
/// - TEST_SUGGESTIONS: Comma-separated list of suggestions (default: "apple,application,apply")
pub struct TestTextInputSuggestionsCommand;

impl TestTextInputSuggestionsCommand {
    /// Create a new test text input suggestions command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestTextInputSuggestionsCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PickerItem for TestTextInputSuggestionsCommand {
    fn constraint(&self) -> Constraint {
        Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        "test text input suggestions"
    }
}

impl WidgetRef for TestTextInputSuggestionsCommand {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("test text input suggestions").render(area, buf);
    }
}

impl Command for TestTextInputSuggestionsCommand {
    fn name(&self) -> &str {
        "test text input suggestions"
    }

    fn description(&self) -> &str {
        "Test text input with suggestions"
    }

    fn run(&self, ctx: &mut CommandCtx) {
        // Get suggestions from environment or use defaults
        let suggestions_str =
            env::var("TEST_SUGGESTIONS").unwrap_or_else(|_| "apple,application,apply".to_string());
        let suggestions: Vec<String> = suggestions_str.split(',').map(|s| s.to_string()).collect();

        let provider = StaticSuggestionProvider::new(suggestions);

        // Show picker with suggestions
        let input = ctx.input_with_suggestions("Query:", Box::new(provider));

        // Output result for test verification
        match input {
            Some(text) => println!("Some({:?})", text),
            None => println!("None"),
        }
    }
}
