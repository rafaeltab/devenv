use std::io::{self, Write};
use std::process::Command;
use tempfile::{NamedTempFile, TempDir};

/// Test context that manages a temporary config file for isolated testing.
/// The temp file is automatically cleaned up when TestContext is dropped.
pub struct TestContext {
    config_file: NamedTempFile,
}

impl TestContext {
    /// Creates a new test context with the given JSON config content.
    pub fn new(content: &str) -> io::Result<Self> {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(TestContext { config_file: file })
    }

    /// Returns the path to the temporary config file.
    pub fn config_path(&self) -> &str {
        self.config_file.path().to_str().unwrap()
    }
}

pub fn run_cli_with_stdin(args: &[&str], input: &str, config_path: &str) -> (String, String) {
    // Build args with --config flag prepended
    let mut full_args = vec!["--config", config_path];
    full_args.extend_from_slice(args);

    let mut command = Command::new("target/debug/rafaeltab")
        .args(&full_args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    {
        let stdin = command.stdin.as_mut().expect("failed to open stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write to stdin");
    }

    let output = command.wait_with_output().expect("failed to read stdout");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (stdout, stderr)
}

pub fn verify_output(expected: &str, actual: &str) {
    assert_eq!(expected, actual, "Output did not match");
}

/// Test context for tmux integration tests with isolated tmux server.
pub struct TmuxTestContext {
    pub socket_name: String,
    pub temp_dir: TempDir,
}

impl TmuxTestContext {
    pub fn new() -> io::Result<Self> {
        let temp_dir = TempDir::new()?;
        let socket_name = format!("rafaeltab-test-{}", uuid::Uuid::new_v4());
        Ok(Self {
            socket_name,
            temp_dir,
        })
    }

    pub fn socket_name(&self) -> &str {
        &self.socket_name
    }

    pub fn temp_dir_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    pub fn list_sessions(&self) -> Vec<String> {
        let output = Command::new("tmux")
            .args([
                "-L",
                &self.socket_name,
                "list-sessions",
                "-F",
                "#{session_name}",
            ])
            .output();

        match output {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect(),
            _ => Vec::new(),
        }
    }
}

impl Drop for TmuxTestContext {
    fn drop(&mut self) {
        // Kill the tmux server for this socket
        let _ = Command::new("tmux")
            .args(["-L", &self.socket_name, "kill-server"])
            .output();
    }
}
