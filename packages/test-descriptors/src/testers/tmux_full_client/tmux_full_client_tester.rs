use super::full_client_asserter::FullClientAsserter;
use crate::descriptor::tmux_client::TmuxClientHandle;
use crate::testers::command::Command;
use crate::testers::traits::TuiTester;

/// TUI tester that runs commands and captures the full tmux client output.
#[derive(Debug)]
pub struct TmuxFullClientTester<'a> {
    client: &'a TmuxClientHandle,
    settle_timeout_ms: u64,
}

impl<'a> TmuxFullClientTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle) -> Self {
        Self {
            client,
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

    fn run(&self, _cmd: &Command) -> Self::Asserter {
        todo!("Phase 3: Implement TmuxFullClientTester::run")
    }
}
