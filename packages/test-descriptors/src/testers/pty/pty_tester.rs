use super::pty_asserter::PtyAsserter;
use crate::testers::command::Command;
use crate::testers::internal::PtyBackend;
use crate::testers::traits::TuiTester;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

/// PTY-based TUI tester that runs commands in a pseudo-terminal.
///
/// This tester spawns commands directly in a PTY without any tmux involvement.
/// It provides the fastest TUI testing experience with:
/// - No tmux dependency - tests run even if tmux is not installed
/// - Full key support - all key combinations work
/// - Direct exit code handling - access to process exit status
/// - Simplest debugging - fewer layers to debug
///
/// Note: The `$TMUX` environment variable will NOT be set when using this tester.
#[derive(Debug)]
pub struct PtyTester {
    rows: u16,
    cols: u16,
    settle_timeout_ms: u64,
}

impl PtyTester {
    pub(crate) fn new() -> Self {
        Self {
            rows: 24,
            cols: 80,
            settle_timeout_ms: 100,
        }
    }

    /// Set the terminal size for the PTY.
    pub fn terminal_size(mut self, rows: u16, cols: u16) -> Self {
        self.rows = rows;
        self.cols = cols;
        self
    }

    /// Set the settle timeout in milliseconds.
    pub fn settle_timeout(mut self, ms: u64) -> Self {
        self.settle_timeout_ms = ms;
        self
    }
}

impl TuiTester for PtyTester {
    type Asserter = PtyAsserter;

    fn run(&self, cmd: &Command) -> Self::Asserter {
        // Use Command's PTY size if set, otherwise use tester defaults
        let (cmd_rows, cmd_cols) = cmd.get_pty_size();
        let (rows, cols) = if cmd_rows != 24 || cmd_cols != 80 {
            // Command has custom PTY size
            (cmd_rows, cmd_cols)
        } else {
            // Use tester defaults
            (self.rows, self.cols)
        };

        // 1. Create PTY system and open a PTY pair
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to create PTY");

        // 2. Build the command
        let mut pty_cmd = CommandBuilder::new(cmd.program());
        for arg in cmd.build_args() {
            pty_cmd.arg(arg);
        }

        // Preserve PATH so the command can find executables
        if let Ok(path) = std::env::var("PATH") {
            pty_cmd.env("PATH", path);
        }

        for (key, value) in cmd.build_env() {
            pty_cmd.env(key, value);
        }
        if let Some(cwd) = cmd.get_cwd() {
            pty_cmd.cwd(cwd);
        }

        // 3. Spawn the command in the PTY
        let child = pty_pair
            .slave
            .spawn_command(pty_cmd)
            .expect("Failed to spawn command in PTY");

        // 4. Create the PtyBackend from the master PTY
        let backend = PtyBackend::new(pty_pair).expect("Failed to create PTY backend");

        // 5. Create and return the PtyAsserter
        PtyAsserter::new(backend, rows, cols, child, self.settle_timeout_ms)
    }
}
