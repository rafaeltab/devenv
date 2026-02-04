use crate::descriptor::tmux_client::TmuxClientHandle;
use crate::descriptor::TmuxSocket;
use crate::testers::command::{Command, CommandResult};
use crate::testers::traits::CommandTester;

/// Command tester that executes commands inside a tmux client via run-shell.
#[derive(Debug)]
pub struct TmuxClientCmdTester<'a> {
    client: &'a TmuxClientHandle,
    socket: &'a TmuxSocket,
}

impl<'a> TmuxClientCmdTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle, socket: &'a TmuxSocket) -> Self {
        Self { client, socket }
    }
}

impl CommandTester for TmuxClientCmdTester<'_> {
    fn run(&self, _cmd: &Command) -> CommandResult {
        todo!("Phase 6: Implement TmuxClientCmdTester::run")
    }
}
