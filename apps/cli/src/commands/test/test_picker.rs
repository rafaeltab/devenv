use std::env;

use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

use crate::commands::{Command, CommandCtx};
use crate::tui::picker_item::PickerItem;
use crate::tui::pickers::{SelectPicker, SimpleItem};

/// Test command for the select picker.
///
/// This command displays a picker with items from TEST_PICKER_ITEMS
/// environment variable and outputs the selection.
///
/// Environment variables:
/// - TEST_PICKER_ITEMS: Comma-separated list of items (e.g., "Item1,Item2,Item3")
pub struct TestPickerCommand;

impl TestPickerCommand {
    /// Create a new test picker command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestPickerCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PickerItem for TestPickerCommand {
    fn constraint(&self) -> Constraint {
        Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        "test picker"
    }
}

impl WidgetRef for TestPickerCommand {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("test picker").render(area, buf);
    }
}

impl Command for TestPickerCommand {
    fn name(&self) -> &str {
        "test picker"
    }

    fn description(&self) -> &str {
        "Test the select picker"
    }

    fn run(&self, ctx: &mut CommandCtx) {
        // Get items from environment
        let items_str = env::var("TEST_PICKER_ITEMS").unwrap_or_default();
        let items: Vec<SimpleItem> = items_str
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| SimpleItem::new(s.to_string()))
            .collect();

        // Show picker
        let selection = ctx.select(&items, "Select an item:");

        // Output result for test verification
        match selection {
            Some(item) => println!("Some({:?})", item.search_text()),
            None => println!("None"),
        }
    }
}
