//! Built-in commands for the command palette.
//!
//! This module provides standard commands available in the command palette,
//! such as adding workspaces, switching tmux sessions, etc.

pub mod add_workspace;

pub use add_workspace::AddWorkspaceCommand;
