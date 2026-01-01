use crate::descriptor::{CreateError, FileChange};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Builder for staged (but not committed) changes
pub struct StagedBuilder {
    changes: Vec<FileChange>,
}

impl StagedBuilder {
    pub(crate) fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Add or modify a file and stage it
    pub fn file(&mut self, path: &str, content: &str) {
        self.changes.push(FileChange::Write {
            path: path.to_string(),
            content: content.to_string(),
        });
    }

    /// Delete a file and stage the deletion
    pub fn delete(&mut self, path: &str) {
        self.changes.push(FileChange::Delete {
            path: path.to_string(),
        });
    }

    pub(crate) fn build(self) -> StagedChanges {
        StagedChanges {
            changes: self.changes,
        }
    }
}

/// Builder for unstaged changes (modified tracked files or untracked files)
pub struct UnstagedBuilder {
    modifications: Vec<FileChange>,
    untracked: Vec<(String, String)>,
}

impl UnstagedBuilder {
    pub(crate) fn new() -> Self {
        Self {
            modifications: Vec::new(),
            untracked: Vec::new(),
        }
    }

    /// Modify an existing tracked file without staging
    pub fn modify(&mut self, path: &str, content: &str) {
        self.modifications.push(FileChange::Write {
            path: path.to_string(),
            content: content.to_string(),
        });
    }

    /// Delete a tracked file without staging
    pub fn delete(&mut self, path: &str) {
        self.modifications.push(FileChange::Delete {
            path: path.to_string(),
        });
    }

    /// Create a new untracked file
    pub fn untracked(&mut self, path: &str, content: &str) {
        self.untracked.push((path.to_string(), content.to_string()));
    }

    pub(crate) fn build(self) -> UnstagedChanges {
        UnstagedChanges {
            modifications: self.modifications,
            untracked: self.untracked,
        }
    }
}

/// Staged changes that haven't been committed
#[derive(Debug, Clone)]
pub struct StagedChanges {
    changes: Vec<FileChange>,
}

impl StagedChanges {
    /// Apply staged changes to a repository path
    pub(crate) fn apply(&self, repo_path: &PathBuf) -> Result<(), CreateError> {
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

        // Stage all changes (but don't commit)
        let output = Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!(
                "Failed to stage changes: {}",
                stderr
            )));
        }

        Ok(())
    }
}

/// Unstaged changes (modifications to tracked files or untracked files)
#[derive(Debug, Clone)]
pub struct UnstagedChanges {
    modifications: Vec<FileChange>,
    untracked: Vec<(String, String)>,
}

impl UnstagedChanges {
    /// Apply unstaged changes to a repository path
    pub(crate) fn apply(&self, repo_path: &PathBuf) -> Result<(), CreateError> {
        // Apply modifications to tracked files (these won't be staged)
        for change in &self.modifications {
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

        // Create untracked files
        for (path, content) in &self.untracked {
            let file_path = repo_path.join(path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, content)?;
        }

        Ok(())
    }
}
