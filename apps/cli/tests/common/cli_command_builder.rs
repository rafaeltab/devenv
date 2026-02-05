use std::collections::HashMap;
use std::path::{Path, PathBuf};
use test_descriptors::testers::Command;
use test_descriptors::TestEnvironment;

/// Builder for rafaeltab CLI commands
///
/// This is a convenience wrapper around `Command` that automatically
/// configures rafaeltab-specific options like the binary path, config path,
/// and tmux socket.
///
/// # Example
///
/// ```ignore
/// let cmd = CliCommandBuilder::new()
///     .with_env(&env)
///     .args(&["tmux", "start"])
///     .build();
///
/// let result = env.testers().cmd().run(&cmd);
/// ```
pub struct CliCommandBuilder {
    args: Vec<String>,
    config_path: Option<PathBuf>,
    tmux_socket: Option<String>,
    cwd: Option<PathBuf>,
    extra_envs: HashMap<String, String>,
}

impl CliCommandBuilder {
    /// Create a new CLI command builder
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            config_path: None,
            tmux_socket: None,
            cwd: None,
            extra_envs: HashMap::new(),
        }
    }

    /// Configure using TestEnvironment (sets config path and tmux socket)
    ///
    /// This extracts the config path and tmux socket from the test environment
    /// and applies them to the command.
    pub fn with_env(mut self, env: &TestEnvironment) -> Self {
        if let Some(config_path) = env.context().config_path() {
            self.config_path = Some(config_path);
        }
        self.tmux_socket = Some(env.tmux_socket().to_string());
        self
    }

    /// Set config path explicitly (overrides with_env)
    pub fn with_config(mut self, path: impl AsRef<Path>) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set tmux socket explicitly (overrides with_env)
    pub fn with_tmux_socket(mut self, socket: impl Into<String>) -> Self {
        self.tmux_socket = Some(socket.into());
        self
    }

    /// Set working directory for the command
    pub fn with_cwd(mut self, dir: impl AsRef<Path>) -> Self {
        self.cwd = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Add a custom environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_envs.insert(key.into(), value.into());
        self
    }

    /// Set the command arguments (the rafaeltab subcommand and its args)
    ///
    /// # Example
    ///
    /// ```ignore
    /// .args(&["tmux", "start"])
    /// .args(&["workspace", "list", "--json"])
    /// ```
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a single argument
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Build the final Command
    ///
    /// This creates a `Command` with:
    /// - The rafaeltab binary path
    /// - `--config <path>` prepended to args (if config path is set)
    /// - `RAFAELTAB_TMUX_SOCKET` environment variable (if tmux socket is set)
    /// - Any additional environment variables
    /// - Working directory (if set)
    pub fn build(self) -> Command {
        let binary_path = env!("CARGO_BIN_EXE_rafaeltab");

        let mut cmd = Command::new(binary_path);

        // Add --config flag before other args
        if let Some(ref config_path) = self.config_path {
            cmd = cmd.arg("--config");
            cmd = cmd.arg(config_path.to_string_lossy().to_string());
        }

        // Add the actual command arguments
        for arg in &self.args {
            cmd = cmd.arg(arg);
        }

        // Set RAFAELTAB_TMUX_SOCKET
        if let Some(ref socket) = self.tmux_socket {
            cmd = cmd.env("RAFAELTAB_TMUX_SOCKET", socket);
        }

        // Add extra environment variables
        for (key, value) in &self.extra_envs {
            cmd = cmd.env(key, value);
        }

        // Set working directory
        if let Some(ref cwd) = self.cwd {
            cmd = cmd.cwd(cwd);
        }

        cmd
    }
}

impl Default for CliCommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}
