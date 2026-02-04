use super::pty_asserter::PtyAsserter;
use crate::testers::command::Command;
use crate::testers::traits::TuiTester;

/// PTY-based TUI tester that runs commands in a pseudo-terminal.
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

    fn run(&self, _cmd: &Command) -> Self::Asserter {
        todo!("Phase 5: Implement PtyTester::run")
    }
}
