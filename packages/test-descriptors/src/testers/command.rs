use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A command to be executed by a tester.
///
/// This is a generic command representation that can be used with any tester.
/// It contains the program name, arguments, environment variables, and working directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    program_name: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    pty_rows: u16,
    pty_cols: u16,
}

impl Command {
    /// Create a new command with the given program name.
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program_name: program.into(),
            args: Vec::new(),
            envs: HashMap::new(),
            working_dir: None,
            pty_rows: 24,
            pty_cols: 80,
        }
    }

    /// Add multiple arguments to the command.
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|s| s.to_string()));
        self
    }

    /// Add a single argument to the command.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set an environment variable for the command.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.envs.insert(key.into(), value.into());
        self
    }

    /// Set multiple environment variables for the command.
    pub fn envs(mut self, envs: HashMap<String, String>) -> Self {
        self.envs.extend(envs);
        self
    }

    /// Set the working directory for the command.
    pub fn cwd(mut self, path: impl AsRef<Path>) -> Self {
        self.working_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the PTY size for TUI testers.
    pub fn pty_size(mut self, rows: u16, cols: u16) -> Self {
        self.pty_rows = rows;
        self.pty_cols = cols;
        self
    }

    /// Get the program name.
    pub fn program(&self) -> &str {
        &self.program_name
    }

    /// Get the arguments as a vector.
    pub fn build_args(&self) -> Vec<String> {
        self.args.clone()
    }

    /// Get the environment variables as a map.
    pub fn build_env(&self) -> HashMap<String, String> {
        self.envs.clone()
    }

    /// Get the working directory.
    pub fn get_cwd(&self) -> Option<PathBuf> {
        self.working_dir.clone()
    }

    /// Get the PTY size (rows, cols).
    pub fn get_pty_size(&self) -> (u16, u16) {
        (self.pty_rows, self.pty_cols)
    }
}

/// Result of running a command via a CommandTester.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Standard output of the command.
    pub stdout: String,
    /// Standard error of the command.
    pub stderr: String,
    /// Exit code of the command.
    pub exit_code: i32,
    /// Whether the command succeeded (exit code 0).
    pub success: bool,
}

impl CommandResult {
    /// Create a new CommandResult.
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            stdout,
            stderr,
            success: exit_code == 0,
            exit_code,
        }
    }
}
