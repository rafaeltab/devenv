use duct::Expression;

/// Configuration for connecting to a tmux server.
/// Allows specifying a custom socket for test isolation.
pub struct TmuxConnection {
    socket_name: Option<String>,
}

impl TmuxConnection {
    /// Connect to the default tmux server
    pub fn default() -> Self {
        Self { socket_name: None }
    }

    /// Connect to a tmux server with a specific socket name (-L flag)
    pub fn with_socket(socket_name: impl Into<String>) -> Self {
        Self {
            socket_name: Some(socket_name.into()),
        }
    }

    /// Build a tmux command with the appropriate socket arguments
    pub fn cmd<I, S>(&self, args: I) -> Expression
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        let mut full_args: Vec<String> = Vec::new();

        if let Some(ref socket) = self.socket_name {
            full_args.push("-L".to_string());
            full_args.push(socket.clone());
        }

        for arg in args {
            full_args.push(arg.as_ref().to_string_lossy().to_string());
        }

        duct::cmd("tmux", full_args)
    }

    /// Build a std::process::Command (for popup_repository which uses that)
    pub fn std_command(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new("tmux");
        if let Some(ref socket) = self.socket_name {
            cmd.args(["-L", socket]);
        }
        cmd
    }
}
