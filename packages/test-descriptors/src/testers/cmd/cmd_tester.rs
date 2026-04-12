use crate::testers::command::{Command, CommandResult};
use crate::testers::traits::CommandTester;
use std::io::Write;
use std::process::{Command as StdCommand, Stdio};

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

        // Preserve PATH so the command can find executables like tmux
        if let Ok(path) = std::env::var("PATH") {
            process.env("PATH", path);
        }

        for (key, value) in cmd.build_env() {
            process.env(key, value);
        }

        // Set working directory if specified
        if let Some(cwd) = cmd.get_cwd() {
            process.current_dir(cwd);
        }

        // Execute and capture output
        let has_stdin = cmd.get_stdin().is_some();
        if has_stdin {
            let mut process = StdCommand::new(cmd.program());
            for arg in cmd.build_args() {
                process.arg(arg);
            }
            process.env_clear();
            if let Ok(path) = std::env::var("PATH") {
                process.env("PATH", path);
            }
            for (key, value) in cmd.build_env() {
                process.env(key, value);
            }
            if let Some(cwd) = cmd.get_cwd() {
                process.current_dir(cwd);
            }
            process
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            match process.spawn() {
                Ok(mut child) => {
                    if let Some(stdin_data) = cmd.get_stdin()
                        && let Some(mut stdin_pipe) = child.stdin.take()
                    {
                        let _ = stdin_pipe.write_all(stdin_data.as_bytes());
                        drop(stdin_pipe);
                    }
                    match child.wait_with_output() {
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
                            stderr: format!("Failed to wait for command: {}", e),
                            exit_code: -1,
                            success: false,
                        },
                    }
                }
                Err(e) => CommandResult {
                    stdout: String::new(),
                    stderr: format!("Failed to execute command: {}", e),
                    exit_code: -1,
                    success: false,
                },
            }
        } else {
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
}
