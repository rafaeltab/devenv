use super::capture_pane_asserter::CapturePaneAsserter;
use crate::descriptor::tmux_client::TmuxClientHandle;
use crate::descriptor::TmuxSocket;
use crate::testers::command::Command;
use crate::testers::traits::TuiTester;

/// TUI tester that runs commands inside a tmux pane and captures output via capture-pane.
///
/// Unlike `TmuxFullClientTester`, this tester only captures the pane content
/// (via `tmux capture-pane`), NOT the full tmux client UI. This means you won't
/// see the tmux status bar or other tmux UI elements.
///
/// Input is sent via `tmux send-keys` rather than writing directly to the PTY.
#[derive(Debug)]
pub struct TmuxClientPtyTester<'a> {
    client: &'a TmuxClientHandle,
    socket: &'a TmuxSocket,
    settle_timeout_ms: u64,
}

impl<'a> TmuxClientPtyTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle, socket: &'a TmuxSocket) -> Self {
        Self {
            client,
            socket,
            settle_timeout_ms: 100,
        }
    }

    /// Set the settle timeout in milliseconds.
    pub fn settle_timeout(mut self, ms: u64) -> Self {
        self.settle_timeout_ms = ms;
        self
    }
}

impl TuiTester for TmuxClientPtyTester<'_> {
    type Asserter = CapturePaneAsserter;

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

        // Add the actual command with properly quoted arguments
        let args = cmd.build_args();
        let quoted_args: Vec<String> = args
            .iter()
            .map(|arg| {
                // Use $'...' syntax for proper escape handling in bash/sh
                // This allows literal escape sequences to work
                let escaped = arg
                    .replace('\\', "\\\\")
                    .replace('\'', "\\'")
                    .replace('\x1b', "\\e"); // ESC character -> \e
                format!("$'{}'", escaped)
            })
            .collect();
        let args_str = if quoted_args.is_empty() {
            String::new()
        } else {
            format!(" {}", quoted_args.join(" "))
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

        let (rows, cols) = self.client.pty_size();

        // Create and return the CapturePaneAsserter
        CapturePaneAsserter::new(
            self.socket.clone(),
            self.client.session_name().to_string(),
            rows,
            cols,
            self.settle_timeout_ms,
        )
    }
}
