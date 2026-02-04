//! Testers module for the testing infrastructure.
//!
//! This module provides traits and implementations for testing commands
//! and TUI applications in various execution contexts.

// Allow dead code since implementations are todo!() and will be used in later phases
#![allow(dead_code)]

// Trait definitions
mod traits;
mod tui_asserter;

// Core types
mod color;
mod command;
mod keys;
mod text_match;

// Implementation folders
mod cmd;
mod factory;
pub(crate) mod internal;
mod pty;
mod tmux_client_cmd;
mod tmux_client_pty;
mod tmux_full_client;

// Public re-exports - Traits
pub use traits::{CommandTester, TuiTester};
pub use tui_asserter::TuiAsserter;

// Public re-exports - Core types
pub use color::{ColorAssertion, ColorMatcher};
pub use command::{Command, CommandResult};
pub use keys::{Key, Modifier};
pub use text_match::TextMatch;

// Public re-exports - Factory
pub use factory::TesterFactory;

// Public re-exports - Testers (users get them from factory, but may need types)
pub use cmd::CmdTester;
pub use pty::{PtyAsserter, PtyTester};
pub use tmux_client_cmd::TmuxClientCmdTester;
pub use tmux_client_pty::{CapturePaneAsserter, TmuxClientPtyTester};
pub use tmux_full_client::{FullClientAsserter, TmuxFullClientTester};
