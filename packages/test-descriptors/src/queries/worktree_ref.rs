use crate::environment::TestEnvironment;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct WorktreeRef<'a> {
    pub(crate) repo_name: String,
    pub(crate) branch: String,
    pub(crate) path: PathBuf,
    pub(crate) env: &'a TestEnvironment,
}

impl<'a> WorktreeRef<'a> {
    /// Get the repository name this worktree belongs to
    pub fn repo_name(&self) -> &str {
        &self.repo_name
    }

    /// Get the branch name of this worktree
    pub fn branch(&self) -> &str {
        &self.branch
    }

    /// Get the path to this worktree
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if the worktree directory exists
    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.join(".git").exists()
    }

    /// Get the current branch of this worktree
    pub fn current_branch(&self) -> String {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git branch");

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    /// Check if the worktree has uncommitted changes
    pub fn is_clean(&self) -> bool {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git status");

        output.stdout.is_empty()
    }

    /// Check if there are staged changes
    pub fn has_staged_changes(&self) -> bool {
        let output = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git diff --cached");

        !output.stdout.is_empty()
    }

    /// Check if there are unstaged changes
    pub fn has_unstaged_changes(&self) -> bool {
        let output = Command::new("git")
            .args(["diff", "--name-only"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git diff");

        !output.stdout.is_empty()
    }

    /// Check if there are untracked files
    pub fn has_untracked_files(&self) -> bool {
        let output = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git ls-files");

        !output.stdout.is_empty()
    }

    /// Get the commit count on this branch
    pub fn commit_count(&self) -> usize {
        let output = Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git rev-list");

        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(0)
    }

    /// Run an arbitrary git command in this worktree
    pub fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git command");

        String::from_utf8_lossy(&output.stdout).to_string()
    }
}
