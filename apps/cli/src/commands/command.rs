use std::fmt::Debug;

/// Error type for command execution failures
#[derive(Debug)]
pub enum CommandError {
    /// IO error occurred during command execution
    Io(std::io::Error),
    /// General error with a message
    General(String),
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError::Io(err)
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Io(err) => write!(f, "IO error: {}", err),
            CommandError::General(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

pub trait RafaeltabCommand<TArgs> {
    fn execute(&self, args: TArgs) -> Result<(), CommandError>;
}

/// Trait for commands that can be displayed and executed in the command palette.
///
/// This trait extends `PickerItem` to allow commands to be displayed in
/// a picker and provides a `run` method for execution.
///
/// # Example
///
/// ```ignore
/// use rafaeltab::commands::Command;
/// use rafaeltab::tui::picker_item::PickerItem;
/// use rafaeltab::commands::command_ctx::CommandCtx;
///
/// struct MyCommand;
///
/// impl PickerItem for MyCommand {
///     fn constraint(&self) -> ratatui::layout::Constraint {
///         ratatui::layout::Constraint::Length(1)
///     }
///
///     fn search_text(&self) -> &str {
///         "my command"
///     }
/// }
///
/// impl Command for MyCommand {
///     fn name(&self) -> &str {
///         "my command"
///     }
///
///     fn description(&self) -> &str {
///         "Does something useful"
///     }
///
///     fn run(&self, ctx: &mut CommandCtx) {
///         // Command logic here
///     }
/// }
/// ```
pub trait Command: Debug {
    /// Returns the command name displayed in the palette.
    fn name(&self) -> &str;

    /// Returns the command description displayed in the palette.
    fn description(&self) -> &str;

    /// Execute the command with the given context.
    ///
    /// The context provides access to picker methods for getting user input.
    fn run(&self, ctx: &mut crate::commands::command_ctx::CommandCtx);
}
