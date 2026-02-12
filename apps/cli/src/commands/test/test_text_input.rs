use std::env;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

use crate::commands::{Command, CommandCtx};

/// Test command for the text input picker.
///
/// This command displays a text input picker with prompt from
/// TEST_TEXT_PROMPT environment variable.
///
/// Environment variables:
/// - TEST_TEXT_PROMPT: The prompt text (default: "Input:")
#[derive(Debug)]
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

    fn run(&self, ctx: &mut CommandCtx) {
        // Get prompt from environment
        let prompt = env::var("TEST_TEXT_PROMPT").unwrap_or_else(|_| "Input:".to_string());

        let input = ctx.input(&prompt);

        // Output result for test verification
        match input {
            Some(text) => println!("Some({:?})", text),
            None => println!("None"),
        }
    }
}
