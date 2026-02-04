#![allow(dead_code)]

use super::context::CreateContext;
use super::error::CreateError;
use super::traits::Descriptor;

/// Handle to a tmux client attached to a session.
///
/// This represents a running tmux client that can be used for testing.
#[derive(Debug)]
pub struct TmuxClientHandle {
    session_name: String,
    pty_rows: u16,
    pty_cols: u16,
    // In a full implementation, this would hold:
    // - pty_pair: PtyPair from portable_pty
    // - child: Box<dyn Child + Send + Sync>
}

impl TmuxClientHandle {
    pub(crate) fn new(session_name: String, pty_rows: u16, pty_cols: u16) -> Self {
        Self {
            session_name,
            pty_rows,
            pty_cols,
        }
    }

    /// Get the name of the session the client is attached to.
    pub fn current_session(&self) -> String {
        self.session_name.clone()
    }

    /// Get the PTY size (rows, cols).
    pub fn pty_size(&self) -> (u16, u16) {
        (self.pty_rows, self.pty_cols)
    }

    /// Get the session name.
    pub fn session_name(&self) -> &str {
        &self.session_name
    }
}

/// Descriptor for creating a tmux client attached to a session.
#[derive(Debug)]
pub struct TmuxClientDescriptor {
    session_name: String,
    pty_rows: u16,
    pty_cols: u16,
}

impl TmuxClientDescriptor {
    pub fn new(session_name: String, pty_rows: u16, pty_cols: u16) -> Self {
        Self {
            session_name,
            pty_rows,
            pty_cols,
        }
    }

    pub fn session_name(&self) -> &str {
        &self.session_name
    }

    pub fn pty_size(&self) -> (u16, u16) {
        (self.pty_rows, self.pty_cols)
    }
}

impl Descriptor for TmuxClientDescriptor {
    fn create(&self, _context: &CreateContext) -> Result<(), CreateError> {
        todo!("Phase 3: Implement TmuxClientDescriptor::create")
    }
}
