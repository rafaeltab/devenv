//! Command context for running commands in the command palette.
//!
//! This module provides the `CommandCtx` struct which gives commands
//! access to picker methods and other runtime functionality.

use std::io::{self};
use std::sync::Arc;

use crate::domain::tmux_workspaces::aggregates::workspaces::workspace::{Workspace, WorkspaceTag};
use crate::domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository;
use crate::tui::picker_ctx::{PickerCtx, SuggestionProvider};
use crate::tui::pickers::SimpleItem;

/// A dummy workspace repository for when no real repository is available.
struct DummyWorkspaceRepository;

impl WorkspaceRepository for DummyWorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace> {
        Vec::new()
    }

    fn create_workspace(
        &self,
        _name: String,
        _tags: Vec<String>,
        _root: String,
        _id: String,
    ) -> Workspace {
        Workspace {
            id: String::new(),
            name: String::new(),
            path: String::new(),
            tags: Vec::new(),
            importance: 0,
            worktree: None,
        }
    }
}

/// Context for executing commands in the command palette.
///
/// `CommandCtx` provides commands with access to:
/// - Picker methods (select, input, confirm, etc.)
/// - Workspace repository for data access (optional)
/// - Terminal for TUI operations
///
/// # Example
///
/// ```ignore
/// use rafaeltab::commands::command_ctx::CommandCtx;
///
/// fn my_command(ctx: &mut CommandCtx) {
///     // Get user input
///     let name = ctx.input("Enter name:");
///
///     // Show selection picker
///     let items = vec!["Option 1", "Option 2"];
///     let selected = ctx.select(&items, "Choose:");
///
///     // Confirm action
///     let confirmed = ctx.confirm("Are you sure?", true);
/// }
/// ```
pub struct CommandCtx {
    picker_ctx: PickerCtx,
    workspace_repo: Arc<dyn WorkspaceRepository>,
}

impl CommandCtx {
    /// Create a new command context.
    pub fn new() -> io::Result<Self> {
        let picker_ctx = PickerCtx::new()?;
        let workspace_repo: Arc<dyn WorkspaceRepository> = Arc::new(DummyWorkspaceRepository);

        Ok(Self {
            picker_ctx,
            workspace_repo,
        })
    }

    /// Create a new command context with a workspace repository.
    pub fn with_repository(workspace_repo: Arc<dyn WorkspaceRepository>) -> io::Result<Self> {
        let picker_ctx = PickerCtx::new()?;

        Ok(Self {
            picker_ctx,
            workspace_repo,
        })
    }

    /// Access the workspace repository.
    pub fn workspace_repo(&self) -> &dyn WorkspaceRepository {
        self.workspace_repo.as_ref()
    }

    /// Display a select picker and return the selected item.
    ///
    /// # Arguments
    /// * `items` - The list of items to display
    /// * `prompt` - The prompt text to display
    ///
    /// # Returns
    /// * `Some(SimpleItem)` - The selected item
    /// * `None` - If the user cancels
    pub fn select(&mut self, items: &[SimpleItem], prompt: &str) -> Option<SimpleItem> {
        self.picker_ctx.select(items, prompt)
    }

    /// Display a text input picker and return the entered text.
    ///
    /// # Arguments
    /// * `prompt` - The prompt text to display
    ///
    /// # Returns
    /// * `Some(String)` - The entered text
    /// * `None` - If the user cancels
    pub fn input(&mut self, prompt: &str) -> Option<String> {
        self.picker_ctx.input(prompt)
    }

    /// Display a text input picker with suggestions.
    ///
    /// # Arguments
    /// * `prompt` - The prompt text to display
    /// * `provider` - A suggestion provider for autocomplete
    ///
    /// # Returns
    /// * `Some(String)` - The entered text
    /// * `None` - If the user cancels
    pub fn input_with_suggestions(
        &mut self,
        prompt: &str,
        provider: Box<dyn SuggestionProvider>,
    ) -> Option<String> {
        self.picker_ctx.input_with_suggestions(prompt, provider)
    }

    /// Display a confirm picker and return the user's choice.
    ///
    /// # Arguments
    /// * `prompt` - The prompt text to display
    /// * `default` - The default selection (true for Yes, false for No)
    ///
    /// # Returns
    /// * `Some(true)` - If the user selects Yes
    /// * `Some(false)` - If the user selects No
    /// * `None` - If the user cancels
    pub fn confirm(&mut self, prompt: &str, default: bool) -> Option<bool> {
        self.picker_ctx.confirm(prompt, default)
    }

    /// Execute a shell command.
    ///
    /// This runs a command in the shell and returns control when complete.
    pub fn execute(&self, command: &str) -> io::Result<()> {
        self.picker_ctx.execute(command)
    }

    /// Restore the terminal to its original state.
    pub fn restore(&mut self) -> io::Result<()> {
        self.picker_ctx.restore()
    }
}
