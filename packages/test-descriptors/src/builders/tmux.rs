use crate::descriptor::{
    CreateContext, CreateError, Descriptor, TmuxClientDescriptor, TmuxSessionInfo, TmuxSocket,
    WindowDescriptor,
};
use std::path::PathBuf;

/// Builder for configuring a tmux client attached to a session.
pub struct ClientBuilder {
    pty_rows: u16,
    pty_cols: u16,
}

impl ClientBuilder {
    pub(crate) fn new() -> Self {
        Self {
            pty_rows: 24,
            pty_cols: 80,
        }
    }

    /// Set the PTY size for the client.
    pub fn pty_size(&mut self, rows: u16, cols: u16) {
        self.pty_rows = rows;
        self.pty_cols = cols;
    }

    pub(crate) fn build(self, session_name: &str) -> TmuxClientDescriptor {
        TmuxClientDescriptor::new(session_name.to_string(), self.pty_rows, self.pty_cols)
    }
}

pub struct SessionBuilder {
    name: String,
    parent_path: PathBuf,
    windows: Vec<WindowDescriptor>,
    client: Option<ClientBuilder>,
}

impl SessionBuilder {
    pub(crate) fn new(name: &str, parent_path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            parent_path,
            windows: Vec::new(),
            client: None,
        }
    }

    pub fn window(&mut self, name: &str) {
        self.windows.push(WindowDescriptor::new(name));
    }

    pub fn window_with_command(&mut self, name: &str, command: &str) {
        self.windows
            .push(WindowDescriptor::new(name).with_command(command));
    }

    /// Configure a tmux client to be attached to this session.
    ///
    /// Only one client can be attached per test environment.
    pub fn with_client<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ClientBuilder),
    {
        let mut builder = ClientBuilder::new();
        f(&mut builder);
        self.client = Some(builder);
    }

    pub(crate) fn build(self) -> HierarchicalTmuxSessionDescriptor {
        HierarchicalTmuxSessionDescriptor {
            name: self.name.clone(),
            parent_path: self.parent_path,
            windows: self.windows,
            client_descriptor: self.client.map(|c| c.build(&self.name)),
        }
    }
}

/// Hierarchical tmux session descriptor that knows its working directory
#[derive(Debug)]
pub struct HierarchicalTmuxSessionDescriptor {
    name: String,
    parent_path: PathBuf,
    windows: Vec<WindowDescriptor>,
    client_descriptor: Option<TmuxClientDescriptor>,
}

impl HierarchicalTmuxSessionDescriptor {
    /// Get the client descriptor if one is configured.
    pub fn client_descriptor(&self) -> Option<&TmuxClientDescriptor> {
        self.client_descriptor.as_ref()
    }
}

impl Descriptor for HierarchicalTmuxSessionDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        // Get the tmux socket from context
        let socket_name = context.tmux_socket().ok_or_else(|| {
            CreateError::InvalidDescriptor("No tmux socket set in context".to_string())
        })?;

        let socket = TmuxSocket::from_name(socket_name);

        // Working directory is the parent path
        let working_dir = self.parent_path.to_string_lossy();

        if self.windows.is_empty() {
            // Create session with default window
            socket.run_tmux(&["new-session", "-d", "-s", &self.name, "-c", &working_dir])?;
        } else {
            // Create session with first window
            let first_window = &self.windows[0];
            let mut args = vec![
                "new-session",
                "-d",
                "-s",
                &self.name,
                "-n",
                first_window.name(),
                "-c",
                &working_dir,
            ];

            let first_cmd_wrapped = first_window.command().map(|c| c.with_persistent_shell());
            if let Some(ref cmd) = first_cmd_wrapped {
                args.push(cmd);
            }

            socket.run_tmux(&args)?;

            // Create additional windows
            for window in &self.windows[1..] {
                let mut args = vec![
                    "new-window",
                    "-t",
                    &self.name,
                    "-n",
                    window.name(),
                    "-c",
                    &working_dir,
                ];

                let cmd_wrapped = window.command().map(|c| c.with_persistent_shell());
                if let Some(ref cmd) = cmd_wrapped {
                    args.push(cmd);
                }

                socket.run_tmux(&args)?;
            }
        }

        // Register the session in context with the correct working directory
        let session_info = TmuxSessionInfo {
            name: self.name.clone(),
            working_dir: self.parent_path.clone(),
        };

        context
            .registry()
            .borrow_mut()
            .register_tmux_session(self.name.clone(), session_info);

        Ok(())
    }
}
