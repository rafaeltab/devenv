use super::full_client_asserter::FullClientAsserter;
use crate::descriptor::tmux_client::TmuxClientHandle;
use crate::descriptor::TmuxSocket;
use crate::testers::command::Command;
use crate::testers::traits::TuiTester;

/// TUI tester that runs commands and captures the full tmux client output.
///
/// This tester uses the existing tmux client PTY spawned by `with_client()`.
/// It sends commands to the tmux pane and reads the full client output,
/// including the tmux UI (status bar, borders, etc.).
#[derive(Debug)]
pub struct TmuxFullClientTester<'a> {
    client: &'a TmuxClientHandle,
    socket: TmuxSocket,
    settle_timeout_ms: u64,
}

impl<'a> TmuxFullClientTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle, socket: &TmuxSocket) -> Self {
        Self {
            client,
            socket: socket.clone(),
            settle_timeout_ms: 100,
        }
    }

    /// Set the settle timeout in milliseconds.
    pub fn settle_timeout(mut self, ms: u64) -> Self {
        self.settle_timeout_ms = ms;
        self
    }
}

impl TuiTester for TmuxFullClientTester<'_> {
    type Asserter = FullClientAsserter;

    fn run(&self, cmd: &Command) -> Self::Asserter {
        // Build the command string with env vars and cwd
        let mut cmd_parts = Vec::new();

        // Add environment variable exports
        for (k, v) in cmd.build_env() {
            // Escape single quotes in values
            let escaped_v = v.replace('\'', "'\\''");
            cmd_parts.push(format!("export {}='{}'", k, escaped_v));
        }

        // Add directory change if specified
        if let Some(cwd) = cmd.get_cwd() {
            cmd_parts.push(format!("cd '{}'", cwd.display()));
        }

        // Add the actual command
        let args = cmd.build_args();
        let args_str = if args.is_empty() {
            String::new()
        } else {
            format!(" {}", args.join(" "))
        };
        cmd_parts.push(format!("{}{}", cmd.program(), args_str));

        let full_cmd = cmd_parts.join("; ");

        // Send the command to the active pane via tmux send-keys
        self.socket
            .run_tmux(&[
                "send-keys",
                "-t",
                self.client.session_name(),
                &full_cmd,
                "Enter",
            ])
            .expect("Failed to send command to tmux pane");

        // Get reader and writer from the client's PTY
        let reader = self
            .client
            .try_clone_reader()
            .expect("Failed to get PTY reader");
        let writer = self.client.take_writer().expect("Failed to get PTY writer");

        let (rows, cols) = self.client.pty_size();

        // Create and return the asserter
        FullClientAsserter::new(
            reader,
            writer,
            rows,
            cols,
            self.settle_timeout_ms,
            self.socket.clone(),
            self.client.session_name().to_string(),
        )
    }
}
