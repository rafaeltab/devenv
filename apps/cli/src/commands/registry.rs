//! Command registry for managing command palette commands.
//!
//! This module provides the `CommandRegistry` which maintains a list
//! of commands that can be displayed and executed in the command palette.

use crate::commands::Command;

/// Registry for command palette commands.
///
/// `CommandRegistry` maintains a collection of commands and provides
/// methods for registering and finding commands by name.
///
/// # Example
///
/// ```ignore
/// use rafaeltab::commands::registry::CommandRegistry;
/// use rafaeltab::commands::Command;
///
/// let mut registry = CommandRegistry::new();
/// registry.register(MyCommand);
/// registry.register(AnotherCommand);
///
/// // Find a command by name
/// if let Some(cmd) = registry.find_by_name("my command") {
///     // Execute the command
/// }
/// ```
pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
}

impl CommandRegistry {
    /// Create a new empty command registry.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Register a command in the registry.
    ///
    /// Returns a mutable reference to self for method chaining.
    pub fn register(&mut self, command: impl Command + 'static) -> &mut Self {
        self.commands.push(Box::new(command));
        self
    }

    /// Get all registered commands.
    pub fn commands(&self) -> &[Box<dyn Command>] {
        &self.commands
    }

    /// Find a command by its name.
    ///
    /// Returns `Some(&dyn Command)` if found, `None` otherwise.
    pub fn find_by_name(&self, name: &str) -> Option<&dyn Command> {
        self.commands
            .iter()
            .find(|cmd| cmd.name() == name)
            .map(|cmd| cmd.as_ref())
    }

    /// Get the number of registered commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
