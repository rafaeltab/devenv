use super::commit::CommitDescriptor;
use super::context::CreateContext;
use super::error::CreateError;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BranchDescriptor {
    name: String,
    base: Option<String>,
    commits: Vec<CommitDescriptor>,
}

impl BranchDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            base: None,
            commits: Vec::new(),
        }
    }

    pub fn from(name: &str, base: &str) -> Self {
        Self {
            name: name.to_string(),
            base: Some(base.to_string()),
            commits: Vec::new(),
        }
    }

    pub fn with_commit(mut self, commit: CommitDescriptor) -> Self {
        self.commits.push(commit);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn base(&self) -> Option<&str> {
        self.base.as_deref()
    }

    pub fn commits(&self) -> &[CommitDescriptor] {
        &self.commits
    }

    /// Check if a branch exists in the repository
    fn branch_exists(repo_path: &PathBuf, branch_name: &str) -> bool {
        let output = Command::new("git")
            .args(["rev-parse", "--verify", branch_name])
            .current_dir(repo_path)
            .output();

        matches!(output, Ok(o) if o.status.success())
    }

    pub(crate) fn apply(
        &self,
        repo_path: &PathBuf,
        context: &CreateContext,
    ) -> Result<(), CreateError> {
        // Check if branch already exists
        if Self::branch_exists(repo_path, &self.name) {
            // Branch exists, just checkout to it
            let output = Command::new("git")
                .args(["checkout", &self.name])
                .current_dir(repo_path)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CreateError::GitError(format!(
                    "Failed to checkout existing branch {}: {}",
                    self.name, stderr
                )));
            }
        } else if let Some(base) = &self.base {
            // Create branch from base
            let output = Command::new("git")
                .args(["checkout", "-b", &self.name, base])
                .current_dir(repo_path)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CreateError::GitError(format!(
                    "Failed to create branch from {}: {}",
                    base, stderr
                )));
            }
        } else {
            // Create new branch from current HEAD
            let output = Command::new("git")
                .args(["checkout", "-b", &self.name])
                .current_dir(repo_path)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CreateError::GitError(format!(
                    "Failed to create branch: {}",
                    stderr
                )));
            }
        }

        // Apply commits
        for commit in &self.commits {
            commit.apply(repo_path, context)?;
        }

        Ok(())
    }
}
