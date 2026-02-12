//! Command Palette implementation.
//!
//! This module provides the main `CommandPalette` command which displays
//! a picker with all available commands and executes the selected one.

use std::rc::Rc;

use ratatui::layout::Spacing;
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, WidgetRef};

use crate::commands::registry::CommandRegistry;
use crate::commands::{Command, CommandCtx};
use crate::tui::picker_item::PickerItem;
use crate::tui::theme::Theme;

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
#[derive(Debug)]
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
                command: cmd.clone(),
            })
            .collect();

        // Create and run the picker
        let res = ctx.select(&items, "Run a command");
        if let Some(cmd) = res {
            cmd.command.run(ctx);
        }
        // If None, user cancelled - just return
    }
}

/// A simple item for displaying commands in the picker.
#[derive(Debug, Clone)]
struct CommandItem {
    name: String,
    description: String,
    pub command: Rc<dyn Command>,
}

impl PickerItem for CommandItem {
    fn constraint(&self) -> Constraint {
        Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        // Search in both name and description
        &self.name
    }

    fn render(&self, selected: bool) -> Box<dyn WidgetRef> {
        Box::new(CommandItemWidget {
            name: self.name.clone(),
            description: self.description.clone(),
            selected,
        })
    }
}

struct CommandItemWidget {
    name: String,
    description: String,
    selected: bool,
}

impl WidgetRef for CommandItemWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let theme = Theme::default();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(self.name.len().try_into().unwrap()),
                Constraint::Fill(1),
            ])
            .spacing(Spacing::Space(1))
            .split(area);
        let mut name_widget = Paragraph::new(self.name.clone());
        if self.selected {
            name_widget = name_widget.style(theme.selected_style());
        }
        name_widget.render(layout[0], buf);

        let description_widget = Paragraph::new(
            Line::from(self.description.clone())
                .right_aligned()
                .fg(Color::DarkGray),
        );
        description_widget.render(layout[1], buf);
    }
}
