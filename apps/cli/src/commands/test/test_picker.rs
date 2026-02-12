use std::env;

use crate::commands::{Command, CommandCtx};
use crate::tui::picker_item::PickerItem;
use crate::tui::pickers::{SimpleItem};

/// Test command for the select picker.
///
/// This command displays a picker with items from TEST_PICKER_ITEMS
/// environment variable and outputs the selection.
///
/// Environment variables:
/// - TEST_PICKER_ITEMS: Comma-separated list of items (e.g., "Item1,Item2,Item3")
#[derive(Debug)]
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
