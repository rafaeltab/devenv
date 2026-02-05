use crate::testers::command::{Command, CommandResult};
use crate::testers::traits::CommandTester;
use std::process::Command as StdCommand;

/// Standard command tester that executes commands as subprocesses.
#[derive(Debug)]
pub struct CmdTester;

impl CmdTester {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl CommandTester for CmdTester {
    fn run(&self, cmd: &Command) -> CommandResult {
        // Build the std::process::Command
        let mut process = StdCommand::new(cmd.program());

        // Add arguments
        for arg in cmd.build_args() {
            process.arg(arg);
        }

        // Clear environment to ensure we're running "outside tmux"
        // Then add the specified environment variables
        process.env_clear();
        for (key, value) in cmd.build_env() {
            process.env(key, value);
        }

        // Set working directory if specified
        if let Some(cwd) = cmd.get_cwd() {
            process.current_dir(cwd);
        }

        // Execute and capture output
        match process.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                CommandResult {
                    stdout,
                    stderr,
                    exit_code,
                    success: output.status.success(),
                }
            }
            Err(e) => CommandResult {
                stdout: String::new(),
                stderr: format!("Failed to execute command: {}", e),
                exit_code: -1,
                success: false,
            },
        }
    }
}
