#![allow(dead_code)]

use super::context::CreateContext;
use super::error::CreateError;
use super::tmux_socket::TmuxSocket;
use super::traits::Descriptor;
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::sync::{Arc, Mutex};

/// Handle to a tmux client attached to a session.
///
/// This represents a running tmux client that can be used for testing.
/// It holds the PTY master and child process for interaction.
pub struct TmuxClientHandle {
    session_name: String,
    pty_rows: u16,
    pty_cols: u16,
    socket_name: String,
    /// The PTY master for reading/writing to the client.
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    /// The child process running the tmux client.
    child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
}

impl TmuxClientHandle {
    pub(crate) fn new(
        session_name: String,
        pty_rows: u16,
        pty_cols: u16,
        socket_name: String,
        master: Box<dyn MasterPty + Send>,
        child: Box<dyn Child + Send + Sync>,
    ) -> Self {
        Self {
            session_name,
            pty_rows,
            pty_cols,
            socket_name,
            master: Arc::new(Mutex::new(master)),
            child: Arc::new(Mutex::new(child)),
        }
    }

    /// Get the name of the session the client is currently attached to.
    ///
    /// This dynamically queries tmux to get the actual current session,
    /// which may have changed if `switch-client` was used.
    pub fn current_session(&self) -> String {
        // Query tmux for the current session using list-clients
        // Format: list clients and find the one matching our PTY
        let socket = TmuxSocket::from_name(self.socket_name.clone());

        // Use display-message to get the current session for this client
        // We need to identify our client - tmux clients are identified by their PTY
        // Since we spawned a tmux client, we can query the session it's attached to
        if let Ok(output) =
            socket.run_tmux(&["list-clients", "-F", "#{client_tty}:#{session_name}"])
        {
            // The output format is: /dev/pts/X:session-name
            // We need to find our client's session
            // Since we may have multiple clients, we'll use the most recent session
            // or fall back to the initial session name
            for line in output.lines() {
                if let Some((_tty, session)) = line.split_once(':') {
                    // Return the first (or only) client's session
                    // In tests, we typically have only one client
                    return session.to_string();
                }
            }
        }

        // Fallback to the initial session name
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

    /// Get the socket name.
    pub fn socket_name(&self) -> &str {
        &self.socket_name
    }

    /// Get a clone of the master PTY Arc for creating backends.
    pub(crate) fn master(&self) -> Arc<Mutex<Box<dyn MasterPty + Send>>> {
        Arc::clone(&self.master)
    }

    /// Try to clone the PTY reader for reading output.
    pub(crate) fn try_clone_reader(
        &self,
    ) -> Result<Box<dyn std::io::Read + Send>, Box<dyn std::error::Error + Send + Sync>> {
        let master = self.master.lock().map_err(|e| e.to_string())?;
        Ok(master.try_clone_reader()?)
    }

    /// Take the PTY writer for writing input.
    pub(crate) fn take_writer(
        &self,
    ) -> Result<Box<dyn std::io::Write + Send>, Box<dyn std::error::Error + Send + Sync>> {
        let master = self.master.lock().map_err(|e| e.to_string())?;
        Ok(master.take_writer()?)
    }

    /// Check if the child process has exited.
    pub fn has_exited(&self) -> bool {
        if let Ok(mut child) = self.child.lock() {
            child.try_wait().ok().flatten().is_some()
        } else {
            false
        }
    }

    /// Wait for the child process to exit and return exit status.
    pub fn wait(&self) -> Option<portable_pty::ExitStatus> {
        if let Ok(mut child) = self.child.lock() {
            child.wait().ok()
        } else {
            None
        }
    }

    /// Kill the child process.
    pub fn kill(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut child = self.child.lock().map_err(|e| e.to_string())?;
        child.kill().map_err(|e| e.into())
    }
}

impl std::fmt::Debug for TmuxClientHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TmuxClientHandle")
            .field("session_name", &self.session_name)
            .field("pty_rows", &self.pty_rows)
            .field("pty_cols", &self.pty_cols)
            .field("socket_name", &self.socket_name)
            .finish()
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

    /// Create the tmux client, spawning it in a PTY.
    pub(crate) fn create_client(&self, socket_name: &str) -> Result<TmuxClientHandle, CreateError> {
        // Verify the session exists
        let socket = TmuxSocket::from_name(socket_name.to_string());
        if !socket.session_exists(&self.session_name) {
            return Err(CreateError::InvalidDescriptor(format!(
                "Cannot attach client to non-existent session: {}",
                self.session_name
            )));
        }

        // Create PTY
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: self.pty_rows,
                cols: self.pty_cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| CreateError::TmuxError(e.to_string()))?;

        // Build the tmux attach command
        let mut cmd = CommandBuilder::new("tmux");
        cmd.arg("-L");
        cmd.arg(socket_name);
        cmd.arg("attach-session");
        cmd.arg("-t");
        cmd.arg(&self.session_name);

        // Spawn the tmux client in the PTY
        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| CreateError::TmuxError(e.to_string()))?;

        // Wait a bit for the client to attach
        std::thread::sleep(std::time::Duration::from_millis(200));

        Ok(TmuxClientHandle::new(
            self.session_name.clone(),
            self.pty_rows,
            self.pty_cols,
            socket_name.to_string(),
            pty_pair.master,
            child,
        ))
    }
}

impl Descriptor for TmuxClientDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        // Get the tmux socket from context
        let socket_name = context.tmux_socket().ok_or_else(|| {
            CreateError::InvalidDescriptor("No tmux socket configured".to_string())
        })?;

        // The actual client creation is deferred to environment creation
        // because we need to register the handle with the environment.
        // This descriptor just validates that the session exists.
        let socket = TmuxSocket::from_name(socket_name);
        if !socket.session_exists(&self.session_name) {
            return Err(CreateError::InvalidDescriptor(format!(
                "Cannot attach client to non-existent session: {}. \
                 Make sure the session is created before the client.",
                self.session_name
            )));
        }

        Ok(())
    }
}
