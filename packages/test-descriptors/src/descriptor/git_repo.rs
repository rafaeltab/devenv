use super::context::CreateContext;
use super::error::CreateError;
use super::traits::{Descriptor, PathDescriptor};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct GitRepoDescriptor {
    name: String,
}

impl GitRepoDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    fn run_git(&self, repo_path: &PathBuf, args: &[&str]) -> Result<(), CreateError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!(
                "Git command failed: {}",
                stderr
            )));
        }

        Ok(())
    }
}

impl Descriptor for GitRepoDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        let path = self.path(context);

        // Create directory and all parent directories
        fs::create_dir_all(&path)?;

        // Initialize git repository
        let output = Command::new("git")
            .args(&["init"])
            .current_dir(&path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!(
                "Failed to init repo: {}",
                stderr
            )));
        }

        // Configure git user for this repository
        self.run_git(&path, &["config", "user.name", "Test User"])?;
        self.run_git(&path, &["config", "user.email", "test@example.com"])?;

        // Create initial README.md
        let readme_path = path.join("README.md");
        fs::write(&readme_path, format!("# {}\n", self.name))?;

        // Create initial commit
        self.run_git(&path, &["add", "README.md"])?;
        self.run_git(&path, &["commit", "-m", "Initial commit"])?;

        // Rename branch to main if needed (for older git versions that default to master)
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(&path)
            .output()?;

        if output.status.success() {
            let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if current_branch != "main" {
                self.run_git(&path, &["branch", "-M", "main"])?;
            }
        }

        // Register the repository in context
        context
            .registry()
            .borrow_mut()
            .register_git_repo(self.name.clone(), path);

        Ok(())
    }
}

impl PathDescriptor for GitRepoDescriptor {
    fn path(&self, context: &CreateContext) -> PathBuf {
        context.root_path().join(&self.name)
    }
}
