use crate::testers::command::{Command, CommandResult};
use crate::testers::traits::CommandTester;

/// Standard command tester that executes commands as subprocesses.
#[derive(Debug)]
pub struct CmdTester;

impl CmdTester {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl CommandTester for CmdTester {
    fn run(&self, _cmd: &Command) -> CommandResult {
        todo!("Phase 7: Implement CmdTester::run")
    }
}
