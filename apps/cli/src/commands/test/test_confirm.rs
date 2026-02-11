use std::env;

use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

use crate::commands::{Command, CommandCtx};
use crate::tui::picker_item::PickerItem;

/// Test command for the confirm picker.
///
/// This command displays a confirm picker with prompt from
/// TEST_CONFIRM_PROMPT environment variable.
///
/// Environment variables:
/// - TEST_CONFIRM_PROMPT: The prompt text (default: "Confirm?")
/// - TEST_CONFIRM_DEFAULT: The default selection (default: "true")
pub struct TestConfirmCommand;

impl TestConfirmCommand {
    /// Create a new test confirm command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestConfirmCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PickerItem for TestConfirmCommand {
    fn constraint(&self) -> Constraint {
        Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        "test confirm"
    }
}

impl WidgetRef for TestConfirmCommand {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("test confirm").render(area, buf);
    }
}

impl Command for TestConfirmCommand {
    fn name(&self) -> &str {
        "test confirm"
    }

    fn description(&self) -> &str {
        "Test the confirm picker"
    }

    fn run(&self, ctx: &mut CommandCtx) {
        // Get prompt and default from environment
        let prompt = env::var("TEST_CONFIRM_PROMPT").unwrap_or_else(|_| "Confirm?".to_string());
        let default = env::var("TEST_CONFIRM_DEFAULT")
            .map(|v| v == "true")
            .unwrap_or(true);

        // Show confirm picker
        let confirmed = ctx.confirm(&prompt, default);

        // Output result for test verification
        match confirmed {
            Some(true) => println!("Some(true)"),
            Some(false) => println!("Some(false)"),
            None => println!("None"),
        }
    }
}
