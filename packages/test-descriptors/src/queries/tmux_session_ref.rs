use super::shell::ShellOutput;
use crate::descriptor::TmuxSocket;
use crate::environment::TestEnvironment;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct TmuxSessionRef<'a> {
    pub(crate) name: String,
    pub(crate) working_dir: PathBuf,
    pub(crate) env: &'a TestEnvironment,
}

impl<'a> TmuxSessionRef<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// Check if the session exists in the tmux server
    pub fn exists(&self) -> bool {
        self.socket().session_exists(&self.name)
    }

    /// Get the list of window names in this session
    pub fn windows(&self) -> Vec<String> {
        let socket = self.socket();
        let result = socket.run_tmux(&["list-windows", "-t", &self.name, "-F", "#{window_name}"]);

        match result {
            Ok(output) => output
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Check if a specific window exists in this session
    pub fn has_window(&self, window_name: &str) -> bool {
        self.windows().contains(&window_name.to_string())
    }

    /// Get the number of windows in this session
    pub fn window_count(&self) -> usize {
        self.windows().len()
    }

    /// Run a shell command in this tmux session's working directory
    ///
    /// This runs the command directly (not inside tmux) but uses the
    /// session's working directory as the current directory.
    pub fn run_shell(&self, command: &str) -> ShellOutput {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .output()
            .expect("Failed to run shell command");

        ShellOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status,
        }
    }

    /// Run a shell command with arguments in this tmux session's working directory
    pub fn run_shell_args(&self, program: &str, args: &[&str]) -> ShellOutput {
        let output = Command::new(program)
            .args(args)
            .current_dir(&self.working_dir)
            .output()
            .expect("Failed to run shell command");

        ShellOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status,
        }
    }

    fn socket(&self) -> TmuxSocket {
        TmuxSocket::from_name(self.env.tmux_socket().to_string())
    }
}
