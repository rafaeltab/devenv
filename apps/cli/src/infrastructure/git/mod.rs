//! Git infrastructure module for worktree management
//!
//! This module provides low-level git operations for managing worktrees,
//! including creating/removing worktrees, checking branch status, and
//! discovering existing worktrees.

use std::path::{Path, PathBuf};

use duct::cmd;

mod error;
pub mod symlink;

pub use error::GitError;

/// Represents the location of a git branch
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchLocation {
    /// Branch exists locally
    Local,
    /// Branch exists on a remote (contains remote name, e.g., "origin")
    Remote(String),
    /// Branch does not exist
    None,
}

/// Information about a git worktree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeInfo {
    /// Absolute path to the worktree
    pub path: PathBuf,
    /// Branch name checked out in this worktree
    pub branch: String,
    /// Whether this is the main worktree (not a linked worktree)
    pub is_main: bool,
}

/// Get the root worktree path (main worktree) from any path within a git repository.
///
/// This works from both the main worktree and any linked worktrees.
///
/// # Arguments
/// * `current_path` - Any path within a git repository
///
/// # Returns
/// The absolute path to the main worktree, or an error if not in a git repository
pub fn get_root_worktree_path(current_path: &Path) -> Result<PathBuf, GitError> {
    // First, get the git common dir which points to the main .git directory
    let output = cmd!(
        "git",
        "rev-parse",
        "--path-format=absolute",
        "--git-common-dir"
    )
    .dir(current_path)
    .stderr_to_stdout()
    .read();

    match output {
        Ok(git_common_dir) => {
            let git_common_dir = git_common_dir.trim();
            // The git common dir is the .git directory of the main worktree
            // We need to get its parent
            let git_path = PathBuf::from(git_common_dir);

            // If it ends with .git, get parent. Otherwise it might be a bare repo
            if git_path.ends_with(".git") {
                git_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .ok_or_else(|| GitError::NotInGitRepo(current_path.to_path_buf()))
            } else {
                // For bare repos or unusual setups, try to get worktree root directly
                let worktree_output = cmd!("git", "worktree", "list", "--porcelain")
                    .dir(current_path)
                    .stderr_to_stdout()
                    .read()
                    .map_err(|_| GitError::NotInGitRepo(current_path.to_path_buf()))?;

                // Parse the first worktree (main worktree) from the porcelain output
                parse_main_worktree_from_porcelain(&worktree_output)
                    .ok_or_else(|| GitError::NotInGitRepo(current_path.to_path_buf()))
            }
        }
        Err(_) => Err(GitError::NotInGitRepo(current_path.to_path_buf())),
    }
}

/// Get the current branch name in the given path.
///
/// # Arguments
/// * `path` - Path within a git repository
///
/// # Returns
/// The current branch name, or an error if in detached HEAD state or not in a repo
pub fn get_current_branch(path: &Path) -> Result<String, GitError> {
    let output = cmd!("git", "rev-parse", "--abbrev-ref", "HEAD")
        .dir(path)
        .stderr_to_stdout()
        .read();

    match output {
        Ok(branch) => {
            let branch = branch.trim().to_string();
            if branch == "HEAD" {
                Err(GitError::DetachedHead)
            } else {
                Ok(branch)
            }
        }
        Err(_) => Err(GitError::NotInGitRepo(path.to_path_buf())),
    }
}

/// Check if a branch exists locally.
///
/// # Arguments
/// * `repo_path` - Path to the git repository
/// * `branch_name` - Name of the branch to check
///
/// # Returns
/// `true` if the branch exists locally, `false` otherwise
pub fn check_branch_exists_locally(repo_path: &Path, branch_name: &str) -> bool {
    cmd!(
        "git",
        "show-ref",
        "--verify",
        "--quiet",
        format!("refs/heads/{}", branch_name)
    )
    .dir(repo_path)
    .run()
    .is_ok()
}

/// Check if a branch exists on a remote.
///
/// # Arguments
/// * `repo_path` - Path to the git repository
/// * `branch_name` - Name of the branch to check
/// * `remote` - Name of the remote (e.g., "origin")
///
/// # Returns
/// `true` if the branch exists on the remote, `false` otherwise
pub fn check_branch_exists_remotely(repo_path: &Path, branch_name: &str, remote: &str) -> bool {
    cmd!(
        "git",
        "show-ref",
        "--verify",
        "--quiet",
        format!("refs/remotes/{}/{}", remote, branch_name)
    )
    .dir(repo_path)
    .run()
    .is_ok()
}

/// Get the location of a branch (local, remote, or none).
///
/// Checks local first, then remote (origin).
///
/// # Arguments
/// * `repo_path` - Path to the git repository
/// * `branch_name` - Name of the branch to check
///
/// # Returns
/// The location of the branch
pub fn get_branch_location(repo_path: &Path, branch_name: &str) -> BranchLocation {
    if check_branch_exists_locally(repo_path, branch_name) {
        BranchLocation::Local
    } else if check_branch_exists_remotely(repo_path, branch_name, "origin") {
        BranchLocation::Remote("origin".to_string())
    } else {
        BranchLocation::None
    }
}

/// Create a new git worktree.
///
/// # Arguments
/// * `repo_path` - Path to the main git repository
/// * `branch_name` - Name of the branch for the new worktree
/// * `worktree_path` - Path where the worktree should be created
/// * `branch_location` - Whether to create a new branch or use an existing one
/// * `base_branch` - Base branch for creating new branches (only used if branch doesn't exist)
///
/// # Returns
/// `Ok(())` on success, or an error describing what went wrong
pub fn create_worktree(
    repo_path: &Path,
    branch_name: &str,
    worktree_path: &Path,
    branch_location: &BranchLocation,
    base_branch: Option<&str>,
) -> Result<(), GitError> {
    // Check if path already exists
    if worktree_path.exists() {
        return Err(GitError::WorktreePathExists(worktree_path.to_path_buf()));
    }

    // Create parent directories if needed
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| GitError::IoError(e.to_string()))?;
    }

    let result = match branch_location {
        BranchLocation::Local => {
            // Use existing local branch
            cmd!("git", "worktree", "add", worktree_path, branch_name)
                .dir(repo_path)
                .stderr_to_stdout()
                .read()
        }
        BranchLocation::Remote(remote) => {
            // Track remote branch
            cmd!(
                "git",
                "worktree",
                "add",
                "--track",
                "-b",
                branch_name,
                worktree_path,
                format!("{}/{}", remote, branch_name)
            )
            .dir(repo_path)
            .stderr_to_stdout()
            .read()
        }
        BranchLocation::None => {
            // Create new branch from base
            let base = base_branch.unwrap_or("HEAD");
            cmd!(
                "git",
                "worktree",
                "add",
                "-b",
                branch_name,
                worktree_path,
                base
            )
            .dir(repo_path)
            .stderr_to_stdout()
            .read()
        }
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(GitError::WorktreeCreationFailed(e.to_string())),
    }
}

/// Remove a git worktree.
///
/// # Arguments
/// * `worktree_path` - Path to the worktree to remove
///
/// # Returns
/// `Ok(())` on success, or an error describing what went wrong
pub fn remove_worktree(worktree_path: &Path) -> Result<(), GitError> {
    // First verify this is actually a worktree
    if !is_worktree(worktree_path) {
        return Err(GitError::NotAWorktree(worktree_path.to_path_buf()));
    }

    // Run from within the worktree - git will find the root automatically
    let result = cmd!("git", "worktree", "remove", worktree_path)
        .dir(worktree_path)
        .stderr_to_stdout()
        .read();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(GitError::WorktreeRemovalFailed(e.to_string())),
    }
}

/// Force remove a git worktree (even with uncommitted changes).
///
/// # Arguments
/// * `worktree_path` - Path to the worktree to remove
///
/// # Returns
/// `Ok(())` on success, or an error describing what went wrong
pub fn force_remove_worktree(worktree_path: &Path) -> Result<(), GitError> {
    if !is_worktree(worktree_path) {
        return Err(GitError::NotAWorktree(worktree_path.to_path_buf()));
    }

    // Run from within the worktree - git will find the root automatically
    let result = cmd!("git", "worktree", "remove", "--force", worktree_path)
        .dir(worktree_path)
        .stderr_to_stdout()
        .read();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(GitError::WorktreeRemovalFailed(e.to_string())),
    }
}

/// Check if the working directory is clean (no uncommitted changes).
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// `true` if clean, `false` if there are uncommitted changes
pub fn check_clean_status(path: &Path) -> Result<bool, GitError> {
    let result = cmd!("git", "status", "--porcelain")
        .dir(path)
        .stderr_to_stdout()
        .read();

    match result {
        Ok(output) => Ok(output.trim().is_empty()),
        Err(_) => Err(GitError::NotInGitRepo(path.to_path_buf())),
    }
}

/// Check if there are unpushed commits on the current branch.
///
/// # Arguments
/// * `path` - Path to the repository
///
/// # Returns
/// `true` if there are unpushed commits, `false` if all commits are pushed
pub fn check_unpushed_commits(path: &Path) -> Result<bool, GitError> {
    // Get the current branch
    let branch = get_current_branch(path)?;

    // Check if there's an upstream branch
    let upstream_result = cmd!(
        "git",
        "rev-parse",
        "--abbrev-ref",
        format!("{}@{{upstream}}", branch)
    )
    .dir(path)
    .stderr_null()
    .read();

    match upstream_result {
        Ok(_) => {
            // Has upstream, check for unpushed commits
            let result = cmd!("git", "log", "@{upstream}..HEAD", "--oneline")
                .dir(path)
                .stderr_to_stdout()
                .read();

            match result {
                Ok(output) => Ok(!output.trim().is_empty()),
                Err(_) => Ok(false), // If command fails, assume no unpushed
            }
        }
        Err(_) => {
            // No upstream branch set - this means we have a local-only branch
            // Consider this as having "unpushed" commits since the branch hasn't been pushed
            Ok(true)
        }
    }
}

/// List all worktrees for a repository.
///
/// # Arguments
/// * `repo_path` - Path to any location within the git repository
///
/// # Returns
/// A vector of `WorktreeInfo` for all worktrees
pub fn list_worktrees(repo_path: &Path) -> Result<Vec<WorktreeInfo>, GitError> {
    let output = cmd!("git", "worktree", "list", "--porcelain")
        .dir(repo_path)
        .stderr_to_stdout()
        .read()
        .map_err(|_| GitError::NotInGitRepo(repo_path.to_path_buf()))?;

    Ok(parse_worktrees_from_porcelain(&output))
}

/// Discover worktrees that belong to a specific workspace.
///
/// # Arguments
/// * `workspace_root` - The root path of the workspace (main worktree)
///
/// # Returns
/// A vector of `WorktreeInfo` for worktrees belonging to this workspace
pub fn discover_worktrees_for_workspace(
    workspace_root: &Path,
) -> Result<Vec<WorktreeInfo>, GitError> {
    let worktrees = list_worktrees(workspace_root)?;

    // Filter to only include non-main worktrees
    // All worktrees for this workspace share the same git directory
    Ok(worktrees.into_iter().filter(|wt| !wt.is_main).collect())
}

/// Check if a path is a git worktree (not the main repository).
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// `true` if this is a linked worktree, `false` if it's the main repo or not a git repo
pub fn is_worktree(path: &Path) -> bool {
    // A worktree has a .git file (not directory) that points to the main repo's worktree directory
    let git_path = path.join(".git");

    if git_path.is_file() {
        // Read the .git file to verify it points to a worktree
        if let Ok(contents) = std::fs::read_to_string(&git_path) {
            return contents.starts_with("gitdir:");
        }
    }

    false
}

/// Remove empty parent directories up to (but not including) a stop path.
///
/// # Arguments
/// * `path` - Starting path (will remove parents of this path)
/// * `stop_at` - Stop removing when reaching this path
///
/// # Returns
/// `Ok(())` on success, or an error if removal fails
pub fn remove_empty_parent_directories(path: &Path, stop_at: &Path) -> Result<(), GitError> {
    let mut current = path.parent();

    while let Some(parent) = current {
        // Stop if we've reached the stop path
        if parent == stop_at || !parent.starts_with(stop_at) {
            break;
        }

        // Check if directory is empty
        let is_empty = match std::fs::read_dir(parent) {
            Ok(mut entries) => entries.next().is_none(),
            Err(_) => break,
        };

        if is_empty {
            if let Err(e) = std::fs::remove_dir(parent) {
                return Err(GitError::IoError(format!(
                    "Failed to remove directory {:?}: {}",
                    parent, e
                )));
            }
            current = parent.parent();
        } else {
            // Directory is not empty, stop
            break;
        }
    }

    Ok(())
}

/// Parse worktree information from `git worktree list --porcelain` output.
fn parse_worktrees_from_porcelain(output: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;
    let mut is_main = true; // First worktree is main

    for line in output.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if complete
            if let (Some(path), Some(branch)) = (current_path.take(), current_branch.take()) {
                worktrees.push(WorktreeInfo {
                    path,
                    branch,
                    is_main,
                });
                is_main = false; // Subsequent worktrees are not main
            }
            current_path = Some(PathBuf::from(line.strip_prefix("worktree ").unwrap()));
        } else if line.starts_with("branch ") {
            let branch = line
                .strip_prefix("branch refs/heads/")
                .unwrap_or(line.strip_prefix("branch ").unwrap_or(""));
            current_branch = Some(branch.to_string());
        } else if line == "detached" {
            current_branch = Some("HEAD".to_string());
        }
    }

    // Don't forget the last worktree
    if let (Some(path), Some(branch)) = (current_path, current_branch) {
        worktrees.push(WorktreeInfo {
            path,
            branch,
            is_main,
        });
    }

    worktrees
}

/// Parse the main worktree path from porcelain output.
fn parse_main_worktree_from_porcelain(output: &str) -> Option<PathBuf> {
    for line in output.lines() {
        if line.starts_with("worktree ") {
            return Some(PathBuf::from(line.strip_prefix("worktree ").unwrap()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use test_utils::assert_path_equals;

    use super::*;
    use std::fs;
    use std::process::Command;

    /// Helper to create a temporary git repository for testing
    fn create_temp_git_repo() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to init git repo");

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set git email");

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to set git name");

        // Create initial commit
        fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to add files");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to commit");

        temp_dir
    }

    #[test]
    fn test_get_root_worktree_path_from_main_worktree() {
        let temp_dir = create_temp_git_repo();

        let result = get_root_worktree_path(temp_dir.path());

        assert!(result.is_ok());
        assert_path_equals!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_get_root_worktree_path_from_nested_worktree() {
        let temp_dir = create_temp_git_repo();
        let worktree_path = temp_dir.path().parent().unwrap().join("test-worktree");

        // Create a worktree
        Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                "test-branch",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create worktree");

        let result = get_root_worktree_path(&worktree_path);

        assert!(result.is_ok());
        assert_path_equals!(result.unwrap(), temp_dir.path());

        // Cleanup worktree
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .ok();
    }

    #[test]
    fn test_get_root_worktree_path_fails_when_not_in_git_repo() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = get_root_worktree_path(temp_dir.path());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitError::NotInGitRepo(_)));
    }

    #[test]
    fn test_get_current_branch_returns_branch_name() {
        let temp_dir = create_temp_git_repo();

        // Create and switch to a new branch
        Command::new("git")
            .args(["checkout", "-b", "feature-branch"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create branch");

        let result = get_current_branch(temp_dir.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "feature-branch");
    }

    #[test]
    fn test_check_branch_exists_locally_returns_true() {
        let temp_dir = create_temp_git_repo();

        Command::new("git")
            .args(["branch", "test-branch"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create branch");

        let result = check_branch_exists_locally(temp_dir.path(), "test-branch");

        assert!(result);
    }

    #[test]
    fn test_check_branch_exists_locally_returns_false() {
        let temp_dir = create_temp_git_repo();

        let result = check_branch_exists_locally(temp_dir.path(), "nonexistent-branch");

        assert!(!result);
    }

    #[test]
    fn test_get_branch_location_returns_local_when_exists_locally() {
        let temp_dir = create_temp_git_repo();

        Command::new("git")
            .args(["branch", "local-branch"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create branch");

        let result = get_branch_location(temp_dir.path(), "local-branch");

        assert_eq!(result, BranchLocation::Local);
    }

    #[test]
    fn test_get_branch_location_returns_none_when_branch_not_found() {
        let temp_dir = create_temp_git_repo();

        let result = get_branch_location(temp_dir.path(), "nonexistent-branch");

        assert_eq!(result, BranchLocation::None);
    }

    #[test]
    fn test_create_worktree_creates_directory_and_branch() {
        let temp_dir = create_temp_git_repo();
        let worktree_path = temp_dir.path().parent().unwrap().join("new-worktree");

        let result = create_worktree(
            temp_dir.path(),
            "new-feature",
            &worktree_path,
            &BranchLocation::None,
            None,
        );

        assert!(result.is_ok());
        assert!(worktree_path.exists());
        assert!(check_branch_exists_locally(temp_dir.path(), "new-feature"));

        // Cleanup
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .ok();
    }

    #[test]
    fn test_create_worktree_with_nested_path_creates_parent_dirs() {
        let temp_dir = create_temp_git_repo();
        let worktree_path = temp_dir
            .path()
            .parent()
            .unwrap()
            .join("feat")
            .join("user")
            .join("login");

        let result = create_worktree(
            temp_dir.path(),
            "feat/user/login",
            &worktree_path,
            &BranchLocation::None,
            None,
        );

        assert!(result.is_ok());
        assert!(worktree_path.exists());

        // Cleanup
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .ok();
    }

    #[test]
    fn test_create_worktree_fails_when_path_exists() {
        let temp_dir = create_temp_git_repo();
        let worktree_path = temp_dir.path().parent().unwrap().join("existing-dir");

        fs::create_dir_all(&worktree_path).unwrap();

        let result = create_worktree(
            temp_dir.path(),
            "new-branch",
            &worktree_path,
            &BranchLocation::None,
            None,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GitError::WorktreePathExists(_)
        ));

        // Cleanup
        fs::remove_dir_all(&worktree_path).ok();
    }

    #[test]
    fn test_create_worktree_uses_existing_local_branch() {
        let temp_dir = create_temp_git_repo();
        let worktree_path = temp_dir.path().parent().unwrap().join("existing-branch-wt");

        // Create branch first
        Command::new("git")
            .args(["branch", "existing-branch"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create branch");

        let result = create_worktree(
            temp_dir.path(),
            "existing-branch",
            &worktree_path,
            &BranchLocation::Local,
            None,
        );

        assert!(result.is_ok());
        assert!(worktree_path.exists());

        // Cleanup
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .ok();
    }

    #[test]
    fn test_check_clean_status_returns_true_when_clean() {
        let temp_dir = create_temp_git_repo();

        let result = check_clean_status(temp_dir.path());

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_check_clean_status_returns_false_with_uncommitted_changes() {
        let temp_dir = create_temp_git_repo();

        // Create uncommitted changes
        fs::write(temp_dir.path().join("new-file.txt"), "content").unwrap();

        let result = check_clean_status(temp_dir.path());

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_check_clean_status_returns_false_with_staged_changes() {
        let temp_dir = create_temp_git_repo();

        // Create and stage changes
        fs::write(temp_dir.path().join("staged-file.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "staged-file.txt"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to stage file");

        let result = check_clean_status(temp_dir.path());

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_check_unpushed_commits_handles_branch_with_no_upstream() {
        let temp_dir = create_temp_git_repo();

        // Create a new branch with no upstream
        Command::new("git")
            .args(["checkout", "-b", "local-only-branch"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create branch");

        // Add a commit
        fs::write(temp_dir.path().join("new-file.txt"), "content").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to stage");
        Command::new("git")
            .args(["commit", "-m", "New commit"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to commit");

        let result = check_unpushed_commits(temp_dir.path());

        assert!(result.is_ok());
        // Branch with no upstream should be considered as having unpushed commits
        assert!(result.unwrap());
    }

    #[test]
    fn test_list_worktrees_returns_all_worktrees() {
        let temp_dir = create_temp_git_repo();
        let worktree_path = temp_dir.path().parent().unwrap().join("test-wt");

        // Create a worktree
        Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                "test-branch",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create worktree");

        let result = list_worktrees(temp_dir.path());

        assert!(result.is_ok());
        let worktrees = result.unwrap();
        assert_eq!(worktrees.len(), 2);

        // First should be main
        assert!(worktrees[0].is_main);
        // Second should not be main
        assert!(!worktrees[1].is_main);
        assert_eq!(worktrees[1].branch, "test-branch");

        // Cleanup
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .ok();
    }

    #[test]
    fn test_list_worktrees_returns_single_for_no_worktrees() {
        let temp_dir = create_temp_git_repo();

        let result = list_worktrees(temp_dir.path());

        assert!(result.is_ok());
        let worktrees = result.unwrap();
        // Should only have the main worktree
        assert_eq!(worktrees.len(), 1);
        assert!(worktrees[0].is_main);
    }

    #[test]
    fn test_is_worktree_returns_true_for_worktree() {
        let temp_dir = create_temp_git_repo();
        let worktree_path = temp_dir.path().parent().unwrap().join("is-wt-test");

        Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                "wt-test",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create worktree");

        let result = is_worktree(&worktree_path);

        assert!(result);

        // Cleanup
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .ok();
    }

    #[test]
    fn test_is_worktree_returns_false_for_main_repo() {
        let temp_dir = create_temp_git_repo();

        let result = is_worktree(temp_dir.path());

        assert!(!result);
    }

    #[test]
    fn test_is_worktree_returns_false_for_non_git_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = is_worktree(temp_dir.path());

        assert!(!result);
    }

    #[test]
    fn test_remove_worktree_removes_directory_and_worktree() {
        let temp_dir = create_temp_git_repo();
        // Use a unique suffix based on temp_dir name to avoid collisions
        let unique_suffix = temp_dir.path().file_name().unwrap().to_str().unwrap();
        let worktree_path = temp_dir
            .path()
            .parent()
            .unwrap()
            .join(format!("remove-test-{}", unique_suffix));

        Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                "remove-branch",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create worktree");

        assert!(
            worktree_path.exists(),
            "Worktree should exist after creation"
        );

        let result = remove_worktree(&worktree_path);

        assert!(result.is_ok(), "Expected Ok but got: {:?}", result.err());
        assert!(
            !worktree_path.exists(),
            "Worktree should not exist after removal"
        );
    }

    #[test]
    fn test_remove_worktree_fails_when_not_a_worktree() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = remove_worktree(temp_dir.path());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitError::NotAWorktree(_)));
    }

    #[test]
    fn test_discover_worktrees_filters_main() {
        let temp_dir = create_temp_git_repo();
        let worktree_path = temp_dir.path().parent().unwrap().join("discover-test");

        Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                "discover-branch",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to create worktree");

        let result = discover_worktrees_for_workspace(temp_dir.path());

        assert!(result.is_ok());
        let worktrees = result.unwrap();
        // Should only include non-main worktrees
        assert_eq!(worktrees.len(), 1);
        assert!(!worktrees[0].is_main);

        // Cleanup
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                worktree_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .output()
            .ok();
    }

    #[test]
    fn test_remove_empty_parent_directories_single_level() {
        let temp_dir = tempfile::tempdir().unwrap();
        let parent = temp_dir.path().join("parent");
        let child = parent.join("child");

        fs::create_dir_all(&child).unwrap();
        fs::remove_dir(&child).unwrap(); // Remove child, parent is now empty

        let result = remove_empty_parent_directories(&child, temp_dir.path());

        assert!(result.is_ok());
        assert!(!parent.exists());
    }

    #[test]
    fn test_remove_empty_parent_directories_stops_at_non_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let parent = temp_dir.path().join("parent");
        let sibling = parent.join("sibling");
        let child = parent.join("child");

        fs::create_dir_all(&child).unwrap();
        fs::create_dir_all(&sibling).unwrap();
        fs::remove_dir(&child).unwrap(); // Remove child, but sibling still exists

        let result = remove_empty_parent_directories(&child, temp_dir.path());

        assert!(result.is_ok());
        // Parent should still exist because sibling is there
        assert!(parent.exists());
    }

    #[test]
    fn test_remove_empty_parent_directories_nested() {
        let temp_dir = tempfile::tempdir().unwrap();
        let level1 = temp_dir.path().join("level1");
        let level2 = level1.join("level2");
        let level3 = level2.join("level3");

        fs::create_dir_all(&level3).unwrap();
        fs::remove_dir(&level3).unwrap();

        let result = remove_empty_parent_directories(&level3, temp_dir.path());

        assert!(result.is_ok());
        assert!(!level2.exists());
        assert!(!level1.exists());
    }
}
