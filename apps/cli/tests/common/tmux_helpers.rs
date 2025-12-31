use std::io;
use tempfile::TempDir;
use uuid::Uuid;

/// Test context for tmux integration tests with isolated server
pub struct TmuxTestContext {
    pub socket_name: String,
    pub temp_dir: TempDir,
}

impl TmuxTestContext {
    pub fn new() -> io::Result<Self> {
        let socket_name = format!("rafaeltab_test_{}", Uuid::new_v4());
        let temp_dir = TempDir::new()?;
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

    /// Run a tmux command on the isolated server
    pub fn tmux(&self, args: &[&str]) -> String {
        let mut full_args = vec!["-L", &self.socket_name];
        full_args.extend(args);
        duct::cmd("tmux", full_args)
            .stderr_to_stdout()
            .read()
            .unwrap_or_default()
    }

    /// List sessions on isolated server
    pub fn list_sessions(&self) -> Vec<String> {
        self.tmux(&["list-sessions", "-F", "#{session_name}"])
            .lines()
            .map(String::from)
            .collect()
    }

    /// Check if a session exists
    #[allow(dead_code)]
    pub fn session_exists(&self, session_name: &str) -> bool {
        self.list_sessions().contains(&session_name.to_string())
    }

    /// Kill the test server
    pub fn kill_server(&self) {
        let _ = self.tmux(&["kill-server"]);
    }
}

impl Drop for TmuxTestContext {
    fn drop(&mut self) {
        self.kill_server();
    }
}
