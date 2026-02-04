use super::error::CreateError;
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TmuxSocket {
    name: String,
}

impl TmuxSocket {
    pub fn new() -> Self {
        Self {
            name: format!("test-tmux-{}", Uuid::new_v4()),
        }
    }

    pub fn from_name(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn run_tmux(&self, args: &[&str]) -> Result<String, CreateError> {
        let output = Command::new("tmux")
            .arg("-L")
            .arg(&self.name)
            .args(args)
            // Use bash as the default shell for a clean test environment.
            // This avoids issues with user shell configurations (fancy prompts, etc.)
            // and ensures consistent behavior with $'...' escape syntax.
            .env("SHELL", "/bin/bash")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::TmuxError(format!(
                "Tmux command failed: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, CreateError> {
        match self.run_tmux(&["list-sessions", "-F", "#{session_name}"]) {
            Ok(output) => Ok(output.lines().map(|s| s.to_string()).collect()),
            Err(_) => Ok(vec![]), // No sessions exist yet
        }
    }

    pub fn session_exists(&self, name: &str) -> bool {
        self.run_tmux(&["has-session", "-t", name]).is_ok()
    }

    pub fn kill_server(&self) -> Result<(), CreateError> {
        // Try to kill server, ignore errors if no server is running
        let _ = self.run_tmux(&["kill-server"]);
        Ok(())
    }
}

impl Default for TmuxSocket {
    fn default() -> Self {
        Self::new()
    }
}
