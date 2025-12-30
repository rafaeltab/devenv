//! Domain errors for worktree management

use std::path::PathBuf;

/// Errors that can occur during worktree domain operations
#[derive(Debug, Clone)]
pub enum WorktreeError {
    /// The current directory is not within a known workspace
    NotInWorkspace,
    /// The workspace does not have worktree configuration
    WorktreeConfigMissing {
        workspace_name: String,
    },
    /// The current directory is not inside a git repository
    NotInGitRepo(PathBuf),
    /// The worktree has uncommitted changes
    HasUncommittedChanges(PathBuf),
    /// The worktree has unpushed commits
    HasUnpushedCommits(PathBuf),
    /// The specified worktree was not found
    WorktreeNotFound(String),
    /// The worktree path would conflict with an existing directory
    PathConflict(PathBuf),
    /// Cannot perform operation on the main repository (not a worktree)
    IsMainRepo(PathBuf),
    /// An onCreate command failed during worktree setup
    OnCreateCommandFailed {
        command: String,
        error: String,
    },
    /// Symlink creation failed
    SymlinkFailed {
        path: PathBuf,
        error: String,
    },
    /// Git operation failed
    GitError(String),
    /// User cancelled the operation
    UserCancelled,
}

impl std::fmt::Display for WorktreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorktreeError::NotInWorkspace => {
                write!(f, "Current directory is not within a known workspace")
            }
            WorktreeError::WorktreeConfigMissing { workspace_name } => {
                write!(
                    f,
                    "Workspace '{}' does not have worktree configuration. Use --force to continue with defaults.",
                    workspace_name
                )
            }
            WorktreeError::NotInGitRepo(path) => {
                write!(f, "Not in a git repository: {}", path.display())
            }
            WorktreeError::HasUncommittedChanges(path) => {
                write!(
                    f,
                    "Worktree has uncommitted changes: {}. Use --force to remove anyway.",
                    path.display()
                )
            }
            WorktreeError::HasUnpushedCommits(path) => {
                write!(
                    f,
                    "Worktree has unpushed commits: {}. Use --force to remove anyway.",
                    path.display()
                )
            }
            WorktreeError::WorktreeNotFound(branch) => {
                write!(f, "Worktree for branch '{}' not found", branch)
            }
            WorktreeError::PathConflict(path) => {
                write!(f, "Path already exists: {}", path.display())
            }
            WorktreeError::IsMainRepo(path) => {
                write!(
                    f,
                    "Cannot perform this operation on the main repository: {}",
                    path.display()
                )
            }
            WorktreeError::OnCreateCommandFailed { command, error } => {
                write!(f, "onCreate command '{}' failed: {}", command, error)
            }
            WorktreeError::SymlinkFailed { path, error } => {
                write!(f, "Failed to create symlink for {:?}: {}", path, error)
            }
            WorktreeError::GitError(msg) => {
                write!(f, "Git error: {}", msg)
            }
            WorktreeError::UserCancelled => {
                write!(f, "Operation cancelled by user")
            }
        }
    }
}

impl std::error::Error for WorktreeError {}

impl From<crate::infrastructure::git::GitError> for WorktreeError {
    fn from(err: crate::infrastructure::git::GitError) -> Self {
        WorktreeError::GitError(err.to_string())
    }
}
