use super::context::CreateContext;
use super::error::CreateError;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum FileChange {
    Write { path: String, content: String },
    Delete { path: String },
}

#[derive(Debug, Clone)]
pub struct CommitDescriptor {
    message: String,
    changes: Vec<FileChange>,
    pushed_to: Option<String>,
    pushed_as: Option<String>,
}

impl CommitDescriptor {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            changes: Vec::new(),
            pushed_to: None,
            pushed_as: None,
        }
    }

    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        self.changes.push(FileChange::Write {
            path: path.to_string(),
            content: content.to_string(),
        });
        self
    }

    pub fn with_delete(mut self, path: &str) -> Self {
        self.changes.push(FileChange::Delete {
            path: path.to_string(),
        });
        self
    }

    pub fn pushed_to(mut self, remote: &str) -> Self {
        self.pushed_to = Some(remote.to_string());
        self
    }

    pub fn pushed_as(mut self, remote: &str, branch: &str) -> Self {
        self.pushed_to = Some(remote.to_string());
        self.pushed_as = Some(branch.to_string());
        self
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn changes(&self) -> &[FileChange] {
        &self.changes
    }

    pub fn get_pushed_to(&self) -> Option<&str> {
        self.pushed_to.as_deref()
    }

    pub fn get_pushed_as(&self) -> Option<&str> {
        self.pushed_as.as_deref()
    }

    pub(crate) fn apply(
        &self,
        repo_path: &PathBuf,
        _context: &CreateContext,
    ) -> Result<(), CreateError> {
        // Apply file changes
        for change in &self.changes {
            match change {
                FileChange::Write { path, content } => {
                    let file_path = repo_path.join(path);
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&file_path, content)?;
                }
                FileChange::Delete { path } => {
                    let file_path = repo_path.join(path);
                    if file_path.exists() {
                        fs::remove_file(&file_path)?;
                    }
                }
            }
        }

        // Stage all changes
        let output = Command::new("git")
            .args(&["add", "-A"])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!(
                "Failed to stage changes: {}",
                stderr
            )));
        }

        // Create commit
        let output = Command::new("git")
            .args(&["commit", "-m", &self.message])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!(
                "Failed to commit: {}",
                stderr
            )));
        }

        Ok(())
    }
}
