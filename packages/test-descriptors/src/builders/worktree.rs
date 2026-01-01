use crate::descriptor::{CommitDescriptor, CreateContext, CreateError, Descriptor};
use std::path::PathBuf;
use std::process::Command;

use super::git::CommitBuilder;

pub struct WorktreeBuilder {
    repo_name: String,
    base_branch: String,
    branch: String,
    parent_path: PathBuf,
    commits: Vec<CommitDescriptor>,
}

impl WorktreeBuilder {
    pub(crate) fn new(
        repo_name: &str,
        base_branch: &str,
        branch: &str,
        parent_path: PathBuf,
    ) -> Self {
        Self {
            repo_name: repo_name.to_string(),
            base_branch: base_branch.to_string(),
            branch: branch.to_string(),
            parent_path,
            commits: Vec::new(),
        }
    }

    /// Add a commit to this worktree
    pub fn commit<F>(&mut self, message: &str, f: F)
    where
        F: FnOnce(&mut CommitBuilder),
    {
        let mut builder = CommitBuilder::new(message);
        f(&mut builder);
        self.commits.push(builder.build());
    }

    pub(crate) fn build(self) -> HierarchicalWorktreeDescriptor {
        HierarchicalWorktreeDescriptor {
            repo_name: self.repo_name,
            base_branch: self.base_branch,
            branch: self.branch,
            parent_path: self.parent_path,
            commits: self.commits,
        }
    }
}

/// Hierarchical git worktree descriptor
#[derive(Debug)]
pub struct HierarchicalWorktreeDescriptor {
    repo_name: String,
    base_branch: String,
    branch: String,
    parent_path: PathBuf,
    commits: Vec<CommitDescriptor>,
}

impl HierarchicalWorktreeDescriptor {
    /// Convert branch name to a valid directory name (e.g., "feature/test" -> "feature-test")
    fn branch_to_dirname(branch: &str) -> String {
        branch.replace('/', "-")
    }

    fn run_git(path: &PathBuf, args: &[&str]) -> Result<String, CreateError> {
        let output = Command::new("git").args(args).current_dir(path).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!(
                "Git command failed: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Descriptor for HierarchicalWorktreeDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        // Find the repo path from the registry
        let repo_path = context
            .registry()
            .borrow()
            .get_git_repo(&self.repo_name)
            .cloned()
            .ok_or_else(|| {
                CreateError::InvalidDescriptor(format!(
                    "Git repository '{}' not found. Worktrees must be created after their parent repository.",
                    self.repo_name
                ))
            })?;

        // Calculate worktree directory path
        let worktree_dirname = Self::branch_to_dirname(&self.branch);
        let worktree_path = self.parent_path.join(&worktree_dirname);

        // Create the worktree from the base branch
        // git worktree add -b <new-branch> <path> <base-branch>
        Self::run_git(
            &repo_path,
            &[
                "worktree",
                "add",
                "-b",
                &self.branch,
                &worktree_path.to_string_lossy(),
                &self.base_branch,
            ],
        )?;

        // Apply commits to the worktree
        for commit in &self.commits {
            commit.apply(&worktree_path, context)?;
        }

        // Register the worktree in context
        context.registry().borrow_mut().register_worktree(
            self.repo_name.clone(),
            self.branch.clone(),
            worktree_path,
        );

        Ok(())
    }
}
