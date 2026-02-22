use duct::Expression;
use shaku::{Component, Interface};
use std::sync::Arc;

use crate::di::SocketNameProvider;

/// Configuration for connecting to a tmux server.
/// Allows specifying a custom socket for test isolation.
pub trait TmuxConnection: Interface {
    /// Build a duct Expression for running tmux commands
    fn cmd(&self, args: Vec<String>) -> Expression;

    /// Build a std::process::Command (for popup_repository which uses that)
    fn std_command(&self) -> std::process::Command;
}

/// Production implementation of TmuxConnection
#[derive(Component)]
#[shaku(interface = TmuxConnection)]
pub struct TmuxConnectionImpl {
    #[shaku(inject)]
    socket_name_provider: Arc<dyn SocketNameProvider>,
}

impl TmuxConnectionImpl {
    /// Connect to the default tmux server (reads from RAFAELTAB_TMUX_SOCKET env var)
    pub fn default() -> Self {
        Self {
            socket_name_provider: Arc::new(crate::di::SocketNameOption::default())
                as Arc<dyn SocketNameProvider>,
        }
    }

    /// Connect to a tmux server with a specific socket name (-L flag)
    /// Note: For test isolation, set RAFAELTAB_TMUX_SOCKET environment variable instead
    /// This method exists for API compatibility but delegates to env var reading
    pub fn with_socket(_socket_name: impl Into<String>) -> Self {
        // Just use default which reads from env - this ensures test isolation works
        Self::default()
    }
}

impl TmuxConnection for TmuxConnectionImpl {
    fn cmd(&self, args: Vec<String>) -> Expression {
        let mut full_args: Vec<String> = Vec::new();

        if let Some(socket) = self.socket_name_provider.socket_name() {
            full_args.push("-L".to_string());
            full_args.push(socket.clone());
        }

        for arg in args {
            full_args.push(arg);
        }

        duct::cmd("tmux", full_args)
    }

    fn std_command(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new("tmux");
        if let Some(socket) = self.socket_name_provider.socket_name() {
            cmd.args(["-L", &socket]);
        }
        cmd
    }
}
