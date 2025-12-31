use super::branch::BranchDescriptor;
use super::context::CreateContext;
use super::error::CreateError;
use super::remote::RemoteDescriptor;
use super::traits::{Descriptor, PathDescriptor};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct GitRepoDescriptor {
    name: String,
    branches: Vec<BranchDescriptor>,
    remotes: Vec<RemoteDescriptor>,
    initial_branch: String,
}

impl GitRepoDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            branches: Vec::new(),
            remotes: Vec::new(),
            initial_branch: "main".to_string(),
        }
    }

    pub fn with_branch(mut self, branch: BranchDescriptor) -> Self {
        self.branches.push(branch);
        self
    }

    pub fn with_remote(mut self, remote: RemoteDescriptor) -> Self {
        self.remotes.push(remote);
        self
    }

    pub fn with_initial_branch(mut self, branch: &str) -> Self {
        self.initial_branch = branch.to_string();
        self
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

        // Rename branch to desired initial branch if needed
        let output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(&path)
            .output()?;

        if output.status.success() {
            let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if current_branch != self.initial_branch {
                self.run_git(&path, &["branch", "-M", &self.initial_branch])?;
            }
        }

        // Setup remotes
        for remote in &self.remotes {
            let bare_path = remote.create_bare_repo(context)?;
            let bare_url = bare_path.to_string_lossy();
            self.run_git(&path, &["remote", "add", remote.name(), &bare_url])?;
        }

        // Create additional branches
        for branch in &self.branches {
            branch.apply(&path, context)?;
        }

        // Return to initial branch
        self.run_git(&path, &["checkout", &self.initial_branch])?;

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
