use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::enable_raw_mode;
use ratatui::prelude::*;
use ratatui::widgets::ListState;
use ratatui::{layout::Layout, DefaultTerminal, Frame};

use super::widgets::command_list::CommandListWidget;
use super::widgets::text_input::{TextInput, TextInputState};
use super::{CommandPaletteCommand, ExampleCommand};

pub struct CommandPalette {
    state: CommandPaletteState,
}

impl KeyListener for CommandPalette {
    fn handle_key(&mut self, key: KeyEvent) {
        let is_ctrl_c =
            key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c');
        if is_ctrl_c {
            self.state = CommandPaletteState::Completed;
            return;
        }

        match self.state {
            CommandPaletteState::CommandSearch(..) => {
                if key.code == KeyCode::Enter {
                    // TODO advance to the next state
                    self.state = CommandPaletteState::Completed;
                    return;
                }
            }
            CommandPaletteState::Completed => panic!("Unreachable state"),
        }

        for key_listener in self.state.get_key_listeners() {
            key_listener.handle_key(key);
        }
    }
}

pub trait KeyListener {
    fn handle_key(&mut self, key: KeyEvent);
}

enum CommandPaletteState {
    CommandSearch(CommandSearchState),
    Completed,
}

struct CommandSearchState {
    pub command_input_state: TextInputState,
    pub commands: Vec<Box<dyn CommandPaletteCommand>>,
}

impl CommandPaletteState {
    fn get_key_listeners(&mut self) -> Vec<&mut dyn KeyListener> {
        match self {
            CommandPaletteState::CommandSearch(command_search_state) => {
                let key_listener: &mut dyn KeyListener =
                    &mut command_search_state.command_input_state;
                vec![key_listener]
            }
            CommandPaletteState::Completed => vec![],
        }
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        CommandPalette {
            state: CommandPaletteState::CommandSearch(CommandSearchState {
                command_input_state: TextInputState::default(),
                commands: vec![
                    Box::new(ExampleCommand {
                        title: "Open workspace".to_string(),
                        description: "Search through the workspaces, and open it".to_string()
                    }),
                    Box::new(ExampleCommand {
                        title: "Add workspace".to_string(),
                        description: "Create a workspace in the current directory".to_string()
                    }),
                    Box::new(ExampleCommand {
                        title: "Open link".to_string(),
                        description: "Search through links, and open them".to_string()
                    }),
                    Box::new(ExampleCommand {
                        title: "Github".to_string(),
                        description: "Open a github repository".to_string()
                    })
                ],
            }),
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        enable_raw_mode()?;
        terminal.clear()?;
        loop {
            if let CommandPaletteState::Completed = self.state {
                break Ok(());
            }
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        match &mut self.state {
            CommandPaletteState::CommandSearch(command_search_state) => {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Min(3), Constraint::Percentage(100)])
                    .split(frame.area());

                frame.render_stateful_widget(
                    TextInput {
                        title: "Enter your command:".to_string(),
                    },
                    layout[0],
                    &mut command_search_state.command_input_state,
                );

                frame.render_stateful_widget(
                    CommandListWidget {
                        commands: &command_search_state.commands,
                        search_text: &command_search_state.command_input_state.current_input,
                    },
                    layout[1],
                    &mut ListState::default(),
                );
            }
            CommandPaletteState::Completed => panic!("Unreachable state"),
        }
    }
}
