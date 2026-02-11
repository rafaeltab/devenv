//! Add Workspace command for the command palette.
//!
//! This command provides an interactive flow for adding a new workspace.

use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

use crate::commands::{Command, CommandCtx};
use crate::tui::picker_item::PickerItem;

/// Command to add a new workspace.
///
/// This command guides the user through a multi-step process:
/// 1. Enter workspace name
/// 2. Enter tags (with suggestions from existing workspaces)
/// 3. Confirm the creation
pub struct AddWorkspaceCommand;

impl AddWorkspaceCommand {
    /// Create a new add workspace command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AddWorkspaceCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl PickerItem for AddWorkspaceCommand {
    fn constraint(&self) -> Constraint {
        Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        "add workspace"
    }
}

impl WidgetRef for AddWorkspaceCommand {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("add workspace").render(area, buf);
    }
}

impl Command for AddWorkspaceCommand {
    fn name(&self) -> &str {
        "add workspace"
    }

    fn description(&self) -> &str {
        "Create a workspace in the current directory"
    }

    fn run(&self, _ctx: &mut CommandCtx) {
        // Placeholder implementation
        // Full implementation in Phase 8
        println!("AddWorkspaceCommand would run here");
    }
}
