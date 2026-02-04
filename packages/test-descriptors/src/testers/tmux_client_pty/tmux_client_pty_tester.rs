use super::capture_pane_asserter::CapturePaneAsserter;
use crate::descriptor::tmux_client::TmuxClientHandle;
use crate::descriptor::TmuxSocket;
use crate::testers::command::Command;
use crate::testers::traits::TuiTester;

/// TUI tester that runs commands inside a tmux pane and captures output via capture-pane.
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

    fn run(&self, _cmd: &Command) -> Self::Asserter {
        todo!("Phase 4: Implement TmuxClientPtyTester::run")
    }
}
