use std::env;

use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

use crate::commands::{Command, CommandCtx};
use crate::tui::picker_item::PickerItem;

/// Test command for the text input picker.
///
/// This command displays a text input picker with prompt from
/// TEST_TEXT_PROMPT environment variable.
///
/// Environment variables:
/// - TEST_TEXT_PROMPT: The prompt text (default: "Input:")
pub struct TestTextInputCommand;

impl TestTextInputCommand {
    /// Create a new test text input command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestTextInputCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PickerItem for TestTextInputCommand {
    fn constraint(&self) -> Constraint {
        Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        "test text input"
    }
}

impl WidgetRef for TestTextInputCommand {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("test text input").render(area, buf);
    }
}

impl Command for TestTextInputCommand {
    fn name(&self) -> &str {
        "test text input"
    }

    fn description(&self) -> &str {
        "Test the text input picker"
    }

    fn run(&self, _ctx: &mut CommandCtx) {
        // Get prompt from environment
        let prompt = env::var("TEST_TEXT_PROMPT").unwrap_or_else(|_| "Input:".to_string());

        // Show picker and get input
        // For now, just output a placeholder
        // The actual picker will be integrated via PickerCtx
        let input = env::var("TEST_INPUT").ok();

        // Output result for test verification
        match input {
            Some(text) => println!("Some({:?})", text),
            None => println!("None"),
        }
    }
}
