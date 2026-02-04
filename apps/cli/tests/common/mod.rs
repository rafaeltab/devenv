pub mod rafaeltab_descriptors;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use test_descriptors::TestEnvironment;
use tui_test::{spawn_tui, TuiSession};

/// Builder for running CLI commands in tests
#[derive(Debug, Clone)]
pub struct CliTestRunner {
    config_path: Option<PathBuf>,
    tmux_socket: Option<String>,
    cwd: Option<PathBuf>,
    extra_envs: HashMap<String, String>,
}

impl CliTestRunner {
    /// Create a new CLI test runner with default settings
    pub fn new() -> Self {
        Self {
            config_path: None,
            tmux_socket: None,
            cwd: None,
            extra_envs: HashMap::new(),
        }
    }

    /// Configure using TestEnvironment (sets config and tmux socket)
    pub fn with_env(mut self, env: &TestEnvironment) -> Self {
        if let Some(config_path) = env.context().config_path() {
            self.config_path = Some(config_path);
        }
        self.tmux_socket = Some(env.tmux_socket().to_string());
        self
    }

    /// Set config path explicitly
    pub fn with_config<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set tmux socket explicitly
    pub fn with_tmux(mut self, socket: impl Into<String>) -> Self {
        self.tmux_socket = Some(socket.into());
        self
    }

    /// Set working directory
    pub fn with_cwd<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.cwd = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Add custom environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_envs.insert(key.into(), value.into());
        self
    }

    /// Execute CLI command, returns (stdout, stderr, success)
    pub fn run(self, args: &[&str]) -> (String, String, bool) {
        // Build args with --config flag prepended
        let mut full_args = vec![];

        if let Some(ref config_path) = self.config_path {
            full_args.push("--config");
            full_args.push(config_path.to_str().unwrap());
        }

        full_args.extend_from_slice(args);

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_rafaeltab"));
        cmd.args(&full_args);

        // Set working directory if specified
        if let Some(ref cwd) = self.cwd {
            cmd.current_dir(cwd);
        }

        // Set tmux socket environment variable if specified
        if let Some(ref tmux_socket) = self.tmux_socket {
            cmd.env("RAFAELTAB_TMUX_SOCKET", tmux_socket);
        }

        // Set any extra environment variables
        for (key, value) in &self.extra_envs {
            cmd.env(key, value);
        }

        let output = cmd
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute CLI with args {:?}: {}", args, e));

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        (stdout, stderr, success)
    }

    /// Convert to TUI builder
    pub fn with_tui(self) -> TuiCliTestRunner {
        TuiCliTestRunner::new(self)
    }
}

impl Default for CliTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for running TUI commands in tests
#[derive(Debug)]
pub struct TuiCliTestRunner {
    base: CliTestRunner,
    timeout_ms: Option<u64>,
    terminal_rows: u16,
    terminal_cols: u16,
    settle_timeout_ms: u64,
    stdin: Option<String>,
}

impl TuiCliTestRunner {
    fn new(base: CliTestRunner) -> Self {
        Self {
            base,
            timeout_ms: None,
            terminal_rows: 40,
            terminal_cols: 120,
            settle_timeout_ms: 300,
            stdin: None,
        }
    }

    /// Set overall timeout for the TUI session (in milliseconds)
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set terminal dimensions
    pub fn with_terminal_size(mut self, rows: u16, cols: u16) -> Self {
        self.terminal_rows = rows;
        self.terminal_cols = cols;
        self
    }

    /// Set settle timeout (wait for UI to stabilize)
    pub fn with_settle_timeout(mut self, ms: u64) -> Self {
        self.settle_timeout_ms = ms;
        self
    }

    /// Provide stdin input
    pub fn with_stdin(mut self, input: impl Into<String>) -> Self {
        self.stdin = Some(input.into());
        self
    }

    /// Spawn TUI and return TuiSession
    pub fn run(self, args: &[&str]) -> TuiSession {
        // Build args with --config flag prepended
        let mut full_args = vec![];

        if let Some(ref config_path) = self.base.config_path {
            full_args.push("--config");
            full_args.push(config_path.to_str().unwrap());
        }

        full_args.extend_from_slice(args);

        let mut builder = spawn_tui(env!("CARGO_BIN_EXE_rafaeltab"), &full_args);

        // Set working directory if specified
        if let Some(ref cwd) = self.base.cwd {
            builder = builder.cwd(cwd);
        }

        // Set tmux socket environment variable if specified
        if let Some(ref tmux_socket) = self.base.tmux_socket {
            builder = builder.env("RAFAELTAB_TMUX_SOCKET", tmux_socket);
        }

        // Set any extra environment variables
        for (key, value) in &self.base.extra_envs {
            builder = builder.env(key, value);
        }

        // Set terminal size
        builder = builder.terminal_size(self.terminal_rows, self.terminal_cols);

        // Set settle timeout
        builder = builder.settle_timeout(self.settle_timeout_ms);

        // TODO: Handle stdin input if provided (would need tui_test support)
        // For now, stdin is stored but not used directly in spawn
        let _ = self.stdin;

        // Handle timeout if specified (would need tui_test support for overall timeout)
        let _ = self.timeout_ms;

        builder.spawn().expect("Failed to spawn TUI")
    }
}
