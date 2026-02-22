//! Command Palette implementation.
//!
//! This module provides the main `CommandPalette` command which displays
//! a picker with all available commands and executes the selected one.

use std::sync::Arc;

use ratatui::layout::Spacing;
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, WidgetRef};
use shaku::{Component, Interface};

use crate::commands::registry::CommandRegistry;
use crate::commands::{Command, CommandCtx};
use crate::domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository;
use crate::tui::picker_item::PickerItem;
use crate::tui::theme::Theme;

/// Interface for the command palette.
pub trait CommandPaletteInterface: Interface {
    fn execute(&self);
}

/// The main command palette command.
///
/// This command displays a picker with all registered commands and
/// executes the selected command. It is a shaku Component that injects
/// the workspace repository for creating the command context.
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
#[derive(Component)]
#[shaku(interface = CommandPaletteInterface)]
pub struct CommandPaletteComponent {
    #[shaku(inject)]
    workspace_repository: Arc<dyn WorkspaceRepository>,
}

impl CommandPaletteInterface for CommandPaletteComponent {
    fn execute(&self) {
        // Build the command registry
        let mut registry = CommandRegistry::new();

        // Register normal commands
        use crate::commands::builtin::AddWorkspaceCommand;
        registry.register(AddWorkspaceCommand::new());

        // Register test commands only in TEST_MODE
        if std::env::var("TEST_MODE").is_ok() {
            use crate::commands::test::{
                TestConfirmCommand, TestPickerCommand, TestTextInputCommand,
                TestTextInputSuggestionsCommand,
            };
            registry.register(TestPickerCommand::new());
            registry.register(TestTextInputCommand::new());
            registry.register(TestTextInputSuggestionsCommand::new());
            registry.register(TestConfirmCommand::new());
        }

        let palette = CommandPalette::new(registry);

        if palette.registry().is_empty() {
            println!("No commands available");
            return;
        }

        // Create command context and run
        let mut ctx = CommandCtx::new(self.workspace_repository.clone())
            .expect("Failed to create command context");
        palette.run(&mut ctx);
    }
}

/// The command palette display logic.
/// This is kept as a separate struct to implement the `Command` trait for the picker.
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
    pub command: Arc<dyn Command>,
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
