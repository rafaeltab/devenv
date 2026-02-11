//! Command Palette implementation.
//!
//! This module provides the main `CommandPalette` command which displays
//! a picker with all available commands and executes the selected one.

use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

use crate::commands::registry::CommandRegistry;
use crate::commands::{Command, CommandCtx};
use crate::tui::picker_item::PickerItem;
use crate::tui::pickers::SelectPicker;

/// The main command palette command.
///
/// This command displays a picker with all registered commands and
/// executes the selected command.
///
/// # Example
///
/// ```ignore
/// use rafaeltab::commands::command_palette::CommandPalette;
/// use rafaeltab::commands::registry::CommandRegistry;
///
/// let registry = CommandRegistry::new();
/// let palette = CommandPalette::new(registry);
/// ```
pub struct CommandPalette {
    registry: CommandRegistry,
}

impl CommandPalette {
    /// Create a new command palette with the given command registry.
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }

    /// Get a reference to the internal registry.
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }
}

impl PickerItem for CommandPalette {
    fn constraint(&self) -> Constraint {
        Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        "command palette"
    }
}

impl WidgetRef for CommandPalette {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("command palette").render(area, buf);
    }
}

impl Command for CommandPalette {
    fn name(&self) -> &str {
        "command palette"
    }

    fn description(&self) -> &str {
        "Show the command palette"
    }

    fn run(&self, ctx: &mut CommandCtx) {
        // Get all commands from the registry
        let commands = self.registry.commands();

        if commands.is_empty() {
            // No commands available
            println!("No commands available");
            return;
        }

        // Create picker items for commands
        let items: Vec<CommandItem> = commands
            .iter()
            .map(|cmd| CommandItem {
                name: cmd.name().to_string(),
                description: cmd.description().to_string(),
            })
            .collect();

        // Create and run the picker
        let mut picker = SelectPicker::new(items);

        // Get terminal from context (we need access to terminal)
        // For now, we'll use the terminal directly from the picker_ctx
        use ratatui::backend::CrosstermBackend;
        use ratatui::Terminal;
        use std::io;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

        if let Some(selected) = picker.run(&mut terminal) {
            // Find and run the selected command
            if let Some(cmd) = self.registry.find_by_name(&selected.name) {
                cmd.run(ctx);
            }
        }
        // If None, user cancelled - just return
    }
}

/// A simple item for displaying commands in the picker.
#[derive(Debug, Clone)]
struct CommandItem {
    name: String,
    description: String,
}

impl PickerItem for CommandItem {
    fn constraint(&self) -> Constraint {
        Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        // Search in both name and description
        &self.name
    }
}

impl WidgetRef for CommandItem {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let text = format!("{} - {}", self.name, self.description);
        Paragraph::new(text).render(area, buf);
    }
}
