//! Git operation errors

use std::path::PathBuf;

/// Errors that can occur during git operations
#[derive(Debug, Clone)]
pub enum GitError {
    /// The path is not inside a git repository
    NotInGitRepo(PathBuf),
    /// The repository is in a detached HEAD state
    DetachedHead,
    /// The path is not a git worktree
    NotAWorktree(PathBuf),
    /// The worktree path already exists
    WorktreePathExists(PathBuf),
    /// Failed to create a worktree
    WorktreeCreationFailed(String),
    /// Failed to remove a worktree
    WorktreeRemovalFailed(String),
    /// The worktree has uncommitted changes
    WorktreeHasUncommittedChanges(PathBuf),
    /// The worktree has unpushed commits
    WorktreeHasUnpushedCommits(PathBuf),
    /// Generic I/O error
    IoError(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::NotInGitRepo(path) => {
                write!(f, "Not in a git repository: {}", path.display())
            }
            GitError::DetachedHead => {
                write!(f, "Repository is in detached HEAD state")
            }
            GitError::NotAWorktree(path) => {
                write!(f, "Path is not a git worktree: {}", path.display())
            }
            GitError::WorktreePathExists(path) => {
                write!(f, "Worktree path already exists: {}", path.display())
            }
            GitError::WorktreeCreationFailed(msg) => {
                write!(f, "Failed to create worktree: {}", msg)
            }
            GitError::WorktreeRemovalFailed(msg) => {
                write!(f, "Failed to remove worktree: {}", msg)
            }
            GitError::WorktreeHasUncommittedChanges(path) => {
                write!(f, "Worktree has uncommitted changes: {}", path.display())
            }
            GitError::WorktreeHasUnpushedCommits(path) => {
                write!(f, "Worktree has unpushed commits: {}", path.display())
            }
            GitError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
        }
    }
}

impl std::error::Error for GitError {}
