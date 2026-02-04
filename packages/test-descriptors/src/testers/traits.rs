use super::command::{Command, CommandResult};
use super::tui_asserter::TuiAsserter;

/// Trait for testers that execute commands and return results.
pub trait CommandTester {
    /// Run a command and return the result.
    fn run(&self, cmd: &Command) -> CommandResult;
}

/// Trait for testers that run TUI applications.
pub trait TuiTester {
    /// The type of asserter returned by this tester.
    type Asserter: TuiAsserter;

    /// Run a command as a TUI application and return an asserter.
    fn run(&self, cmd: &Command) -> Self::Asserter;
}
