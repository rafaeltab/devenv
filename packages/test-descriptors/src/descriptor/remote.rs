use super::context::CreateContext;
use super::error::CreateError;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RemoteDescriptor {
    name: String,
}

impl RemoteDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn create_bare_repo(&self, context: &CreateContext) -> Result<PathBuf, CreateError> {
        // Create a unique directory for this bare repository
        let bare_repo_name = format!("bare-{}-{}.git", self.name, Uuid::new_v4());
        let bare_path = context.root_path().join(&bare_repo_name);

        // Create the directory
        std::fs::create_dir_all(&bare_path)?;

        // Initialize as bare repository
        let output = Command::new("git")
            .args(["init", "--bare"])
            .current_dir(&bare_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!(
                "Failed to init bare repo: {}",
                stderr
            )));
        }

        Ok(bare_path)
    }
}
