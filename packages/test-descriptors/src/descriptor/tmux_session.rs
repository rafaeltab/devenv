use super::context::CreateContext;
use super::error::CreateError;
use super::registry::TmuxSessionInfo;
use super::tmux_socket::TmuxSocket;
use super::tmux_window::WindowDescriptor;
use super::traits::Descriptor;

#[derive(Debug, Clone)]
pub struct TmuxSessionDescriptor {
    name: String,
    windows: Vec<WindowDescriptor>,
}

impl TmuxSessionDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            windows: Vec::new(),
        }
    }

    pub fn with_window(mut self, window: WindowDescriptor) -> Self {
        self.windows.push(window);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn windows(&self) -> &[WindowDescriptor] {
        &self.windows
    }
}

impl Descriptor for TmuxSessionDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        // Get the tmux socket from context
        let socket_name = context.tmux_socket().ok_or_else(|| {
            CreateError::InvalidDescriptor("No tmux socket set in context".to_string())
        })?;

        let socket = TmuxSocket::from_name(socket_name);

        // Create session in the root directory
        let working_dir = context.root_path().to_string_lossy();

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

        // Register the session in context
        let session_info = TmuxSessionInfo {
            name: self.name.clone(),
            working_dir: context.root_path().clone(),
        };

        context
            .registry()
            .borrow_mut()
            .register_tmux_session(self.name.clone(), session_info);

        Ok(())
    }
}
