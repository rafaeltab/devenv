#![allow(unused_imports)]

use crate::descriptor::tmux_client::TmuxClientHandle;
use crate::descriptor::TmuxSocket;
use crate::environment::TestEnvironment;

use super::cmd::CmdTester;
use super::pty::PtyTester;
use super::tmux_client_cmd::TmuxClientCmdTester;
use super::tmux_client_pty::TmuxClientPtyTester;
use super::tmux_full_client::TmuxFullClientTester;

/// Factory for creating testers from a test environment.
///
/// This provides access to all available testers, configured based on
/// the environment's setup (e.g., whether a tmux client is available).
pub struct TesterFactory<'a> {
    env: &'a TestEnvironment,
}

impl<'a> TesterFactory<'a> {
    pub(crate) fn new(env: &'a TestEnvironment) -> Self {
        Self { env }
    }

    /// Get a standard command tester (subprocess execution).
    pub fn cmd(&self) -> CmdTester {
        CmdTester::new()
    }

    /// Get a PTY-based TUI tester (direct PTY, outside tmux).
    pub fn pty(&self) -> PtyTester {
        PtyTester::new()
    }

    /// Get a command tester that runs inside a tmux client via run-shell.
    ///
    /// # Panics
    /// Panics if no tmux client is configured in the environment.
    pub fn tmux_client_cmd(&self) -> TmuxClientCmdTester<'a> {
        let client = self
            .env
            .tmux_client()
            .expect("No tmux client configured in environment");
        TmuxClientCmdTester::new(client, self.env.tmux())
    }

    /// Get a TUI tester that runs inside a tmux pane (uses capture-pane).
    ///
    /// # Panics
    /// Panics if no tmux client is configured in the environment.
    pub fn tmux_client_pty(&self) -> TmuxClientPtyTester<'a> {
        let client = self
            .env
            .tmux_client()
            .expect("No tmux client configured in environment");
        TmuxClientPtyTester::new(client, self.env.tmux())
    }

    /// Get a TUI tester that captures the full tmux client output (including tmux UI).
    ///
    /// # Panics
    /// Panics if no tmux client is configured in the environment.
    pub fn tmux_full_client(&self) -> TmuxFullClientTester<'a> {
        let client = self
            .env
            .tmux_client()
            .expect("No tmux client configured in environment");
        TmuxFullClientTester::new(client)
    }
}
