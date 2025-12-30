//! Command to complete (remove) a git worktree

use std::path::{Path, PathBuf};

use crate::{
    commands::command::RafaeltabCommand,
    domain::{
        tmux_workspaces::{
            aggregates::workspaces::workspace::Workspace,
            repositories::{
                tmux::{
                    client_repository::{SwitchClientTarget, TmuxClientRepository},
                    session_repository::TmuxSessionRepository,
                },
                workspace::workspace_repository::WorkspaceRepository,
            },
        },
        worktree::{
            config::find_most_specific_workspace,
            error::WorktreeError,
        },
    },
    infrastructure::git,
    utils::path::expand_path,
};

#[derive(Default)]
pub struct WorktreeCompleteCommand;

pub struct WorktreeCompleteOptions<'a> {
    /// The branch name of the worktree to complete (optional, defaults to current directory)
    pub branch_name: Option<String>,
    /// Force removal even with uncommitted/unpushed changes
    pub force: bool,
    /// Repository for workspace operations
    pub workspace_repository: &'a dyn WorkspaceRepository,
    /// Repository for tmux session operations
    pub session_repository: &'a dyn TmuxSessionRepository,
    /// Repository for tmux client operations  
    pub client_repository: &'a dyn TmuxClientRepository,
}

/// Result of the worktree complete command
pub enum WorktreeCompleteResult {
    /// Worktree was removed successfully
    Success {
        branch_name: String,
        worktree_path: String,
    },
    /// Operation failed with error
    Failed(WorktreeError),
}

impl RafaeltabCommand<WorktreeCompleteOptions<'_>> for WorktreeCompleteCommand {
    fn execute(&self, options: WorktreeCompleteOptions) {
        match self.execute_internal(options) {
            WorktreeCompleteResult::Success {
                branch_name,
                worktree_path,
            } => {
                println!("✓ Completed worktree for branch '{}'", branch_name);
                println!("✓ Removed: {}", worktree_path);
            }
            WorktreeCompleteResult::Failed(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}

impl WorktreeCompleteCommand {
    fn execute_internal(&self, options: WorktreeCompleteOptions) -> WorktreeCompleteResult {
        // 1. Get current directory
        let current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                return WorktreeCompleteResult::Failed(WorktreeError::GitError(format!(
                    "Failed to get current directory: {}",
                    e
                )));
            }
        };

        // 2. Determine the worktree path
        let (worktree_path, branch_name) = if let Some(ref branch) = options.branch_name {
            // Branch name provided - find the worktree
            match find_worktree_by_branch(&current_dir, branch) {
                Ok((path, name)) => (path, name),
                Err(e) => return WorktreeCompleteResult::Failed(e),
            }
        } else {
            // No branch name - use current directory
            if !git::is_worktree(&current_dir) {
                return WorktreeCompleteResult::Failed(WorktreeError::IsMainRepo(
                    current_dir.clone(),
                ));
            }
            
            let branch = match git::get_current_branch(&current_dir) {
                Ok(b) => b,
                Err(e) => {
                    return WorktreeCompleteResult::Failed(WorktreeError::GitError(e.to_string()));
                }
            };
            
            (current_dir.clone(), branch)
        };

        // 3. Verify this is a worktree (not the main repo)
        if !git::is_worktree(&worktree_path) {
            return WorktreeCompleteResult::Failed(WorktreeError::IsMainRepo(worktree_path));
        }

        // 4. Get the root worktree path (main repo)
        let main_repo_path = match git::get_root_worktree_path(&worktree_path) {
            Ok(path) => path,
            Err(e) => {
                return WorktreeCompleteResult::Failed(WorktreeError::GitError(e.to_string()));
            }
        };

        // 5. Safety checks (unless --force)
        if !options.force {
            // Check for uncommitted changes
            match git::check_clean_status(&worktree_path) {
                Ok(true) => {}
                Ok(false) => {
                    return WorktreeCompleteResult::Failed(WorktreeError::HasUncommittedChanges(
                        worktree_path,
                    ));
                }
                Err(e) => {
                    return WorktreeCompleteResult::Failed(WorktreeError::GitError(e.to_string()));
                }
            }

            // Check for unpushed commits
            match git::check_unpushed_commits(&worktree_path) {
                Ok(false) => {}
                Ok(true) => {
                    return WorktreeCompleteResult::Failed(WorktreeError::HasUnpushedCommits(
                        worktree_path,
                    ));
                }
                Err(e) => {
                    return WorktreeCompleteResult::Failed(WorktreeError::GitError(e.to_string()));
                }
            }
        }

        // 6. Find the workspace this worktree belongs to
        let workspaces = options.workspace_repository.get_workspaces();
        let workspace = find_workspace_for_path(&main_repo_path, &workspaces);

        // 7. Find and kill the tmux session for this worktree
        let session_name = if let Some(ws) = workspace {
            format!("{}-{}", ws.name, branch_name)
        } else {
            format!("worktree-{}", branch_name)
        };

        // Try to find the session by matching the path or name
        // First, try to switch the client to the main workspace session
        if let Some(ws) = workspace {
            // Try to find and switch to main workspace session
            switch_to_main_workspace_session(
                options.session_repository,
                options.client_repository,
                &ws.name,
            );
        }

        // Kill the worktree's session if it exists
        kill_session_by_name(options.session_repository, &session_name);
        println!("✓ Closed tmux session: {}", session_name);

        // 8. Change directory away from the worktree if we're in it
        if current_dir.starts_with(&worktree_path) {
            // We're in the worktree, switch to main repo
            if let Err(e) = std::env::set_current_dir(&main_repo_path) {
                eprintln!("Warning: Could not change directory to main repo: {}", e);
            }
        }

        // 9. Remove the git worktree
        let remove_result = if options.force {
            git::force_remove_worktree(&worktree_path)
        } else {
            git::remove_worktree(&worktree_path)
        };

        if let Err(e) = remove_result {
            return WorktreeCompleteResult::Failed(WorktreeError::GitError(e.to_string()));
        }
        println!("✓ Removed git worktree");

        // 10. Remove empty parent directories
        if worktree_path.parent().is_some() {
            if let Err(e) = git::remove_empty_parent_directories(&worktree_path, main_repo_path.parent().unwrap_or(&main_repo_path)) {
                eprintln!("Warning: Could not clean up empty directories: {}", e);
            }
        }

        WorktreeCompleteResult::Success {
            branch_name,
            worktree_path: worktree_path.display().to_string(),
        }
    }
}

/// Find a worktree by its branch name
fn find_worktree_by_branch(current_dir: &Path, branch_name: &str) -> Result<(PathBuf, String), WorktreeError> {
    // First, get the root worktree to find all worktrees
    let root = git::get_root_worktree_path(current_dir)
        .map_err(|e| WorktreeError::GitError(e.to_string()))?;

    // List all worktrees
    let worktrees = git::list_worktrees(&root)
        .map_err(|e| WorktreeError::GitError(e.to_string()))?;

    // Find the one with matching branch name
    for wt in worktrees {
        if wt.branch == branch_name {
            return Ok((wt.path, wt.branch));
        }
    }

    Err(WorktreeError::WorktreeNotFound(branch_name.to_string()))
}

/// Find the workspace that contains the given path.
/// When workspaces are nested, returns the most specific (longest path) match.
fn find_workspace_for_path<'a>(path: &Path, workspaces: &'a [Workspace]) -> Option<&'a Workspace> {
    let path_str = path.to_string_lossy();
    
    // Build a list of (workspace_id, expanded_path) for lookup
    let workspace_paths: Vec<(&str, String)> = workspaces
        .iter()
        .map(|ws| (ws.id.as_str(), expand_path(&ws.path)))
        .collect();
    
    // Find the most specific workspace ID
    let found_id = find_most_specific_workspace(
        &path_str,
        workspace_paths.iter().map(|(id, path)| (*id, path.as_str())),
    )?;
    
    // Return the workspace with that ID
    workspaces.iter().find(|ws| ws.id == found_id)
}

/// Switch the current tmux client to the main workspace session
fn switch_to_main_workspace_session(
    session_repository: &dyn TmuxSessionRepository,
    client_repository: &dyn TmuxClientRepository,
    workspace_name: &str,
) {
    use crate::domain::tmux_workspaces::aggregates::tmux::include_fields_builder::IncludeFieldsBuilder;

    // Get all sessions
    let sessions = session_repository.get_sessions(None, IncludeFieldsBuilder::new().build_session());

    // Find the main workspace session (the one without branch suffix)
    if let Some(main_session) = sessions.iter().find(|s| s.name == workspace_name) {
        client_repository.switch_client(None, SwitchClientTarget::Session(main_session));
    }
}

/// Kill a tmux session by name
fn kill_session_by_name(
    session_repository: &dyn TmuxSessionRepository,
    session_name: &str,
) {
    use crate::domain::tmux_workspaces::aggregates::tmux::include_fields_builder::IncludeFieldsBuilder;

    // Get all sessions
    let sessions = session_repository.get_sessions(None, IncludeFieldsBuilder::new().build_session());

    // Find and kill the session
    if let Some(session) = sessions.iter().find(|s| s.name == session_name) {
        session_repository.kill_session(Some(session));
    }
}
