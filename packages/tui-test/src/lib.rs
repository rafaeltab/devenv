// Public API exports
mod session;
mod terminal;
mod keys;
mod color;
mod text_match;
mod pty_manager;

pub use session::{spawn_tui, TuiSession, TuiSessionBuilder};
pub use keys::Key;
pub use color::{ColorAssertion, ColorMatcher};
pub use text_match::TextMatch;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("Failed to spawn PTY: {0}")]
    PtySpawn(String),
    
    #[error("Failed to write to PTY: {0}")]
    PtyWrite(#[source] std::io::Error),
    
    #[error("Failed to read from PTY: {0}")]
    PtyRead(#[source] std::io::Error),
    
    #[error("Process exited unexpectedly with code {0}")]
    UnexpectedExit(i32),
    
    #[error("Process has not exited yet")]
    ProcessStillRunning,
}
