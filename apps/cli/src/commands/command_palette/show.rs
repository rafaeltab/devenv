use std::io::stdout;

use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::{command_palette::tui::CommandPalette, commands::command::RafaeltabCommand};

#[derive(Default)]
pub struct CommandPaletteShowCommand;

pub struct CommandPaletteShowOptions {}

impl RafaeltabCommand<CommandPaletteShowOptions> for CommandPaletteShowCommand {
    fn execute(&self, _options: CommandPaletteShowOptions) {
        color_eyre::install().expect("Failed to install color eyre");
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).expect("");
        let mut command_palette = CommandPalette::new();
        let result = command_palette.run(terminal);
        ratatui::restore();
        result.expect("")
    }
}
