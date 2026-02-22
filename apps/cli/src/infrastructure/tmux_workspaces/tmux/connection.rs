use duct::Expression;
use shaku::{Component, Interface};

/// Interface for creating tmux commands
pub trait TmuxConnectionInterface: Interface {
    /// Build a tmux command with the appropriate socket arguments
    fn cmd(&self, args: &[&str]) -> Expression;

    /// Build a tmux command from owned string args
    fn cmd_owned(&self, args: &[String]) -> Expression;

    /// Build a std::process::Command (for popup_repository which uses that)
    fn std_command(&self) -> std::process::Command;
}

/// Configuration for connecting to a tmux server.
/// Allows specifying a custom socket for test isolation.
#[derive(Component)]
#[shaku(interface = TmuxConnectionInterface)]
pub struct TmuxConnection {
    #[shaku(default)]
    socket_name: Option<String>,
}

impl Default for TmuxConnection {
    fn default() -> Self {
        Self { socket_name: None }
    }
}

impl TmuxConnection {
    pub fn with_socket(socket: String) -> Self {
        Self {
            socket_name: Some(socket),
        }
    }
}

impl TmuxConnectionInterface for TmuxConnection {
    fn cmd(&self, args: &[&str]) -> Expression {
        let mut full_args: Vec<String> = Vec::new();
        if let Some(socket) = &self.socket_name {
            full_args.push("-L".to_string());
            full_args.push(socket.clone());
        }
        for arg in args {
            full_args.push(arg.to_string());
        }
        duct::cmd("tmux", full_args)
    }

    fn cmd_owned(&self, args: &[String]) -> Expression {
        let mut full_args: Vec<String> = Vec::new();
        if let Some(socket) = &self.socket_name {
            full_args.push("-L".to_string());
            full_args.push(socket.clone());
        }
        for arg in args {
            full_args.push(arg.clone());
        }
        duct::cmd("tmux", full_args)
    }

    fn std_command(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new("tmux");
        if let Some(socket) = &self.socket_name {
            cmd.args(["-L", socket]);
        }
        cmd
    }
}
