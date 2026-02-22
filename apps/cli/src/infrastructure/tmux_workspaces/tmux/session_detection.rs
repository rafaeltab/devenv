use std::sync::Arc;

use shaku::{Component, Interface};

use super::connection::TmuxConnectionInterface;

/// Interface for detecting the current tmux session.
pub trait SessionDetection: Interface {
    /// Get the name of the current tmux session.
    /// Returns None if not running inside tmux.
    fn get_current_tmux_session(&self) -> Option<String>;
}

/// Implementation of session detection that uses the DI-provided tmux connection.
#[derive(Component)]
#[shaku(interface = SessionDetection)]
pub struct ImplSessionDetection {
    #[shaku(inject)]
    connection: Arc<dyn TmuxConnectionInterface>,
}

impl SessionDetection for ImplSessionDetection {
    fn get_current_tmux_session(&self) -> Option<String> {
        // Check if we're in tmux by checking $TMUX environment variable
        if std::env::var("TMUX").is_err() {
            return None;
        }

        // Get session name using tmux display-message via the DI connection
        let mut cmd = self.connection.std_command();
        cmd.args(["display-message", "-p", "#{session_name}"]);

        let output = cmd.output().ok()?;

        if !output.status.success() {
            return None;
        }

        let session_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if session_name.is_empty() {
            return None;
        }

        Some(session_name)
    }
}
