//! Command to complete (remove) a git worktree

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use crate::{
    commands::command::RafaeltabCommand,
    domain::{
        tmux_workspaces::{
            aggregates::workspaces::workspace::Workspace,
            repositories::{
                tmux::{
                    client_repository::{SwitchClientTarget, TmuxClientRepository},
                    description_repository::SessionDescriptionRepository,
                    popup_repository::{PopupOptions, TmuxPopupRepository},
                    session_repository::TmuxSessionRepository,
                },
                workspace::workspace_repository::WorkspaceRepository,
            },
        },
        worktree::{config::find_most_specific_workspace, error::WorktreeError},
    },
    infrastructure::{git, tmux_workspaces::tmux::session_detection::get_current_tmux_session},
    utils::path::expand_path,
};

#[derive(Default)]
pub struct WorktreeCompleteCommand;

pub struct WorktreeCompleteOptions<'a> {
    /// The branch name of the worktree to complete (optional, defaults to current directory)
    pub branch_name: Option<String>,
    /// Force removal even with uncommitted/unpushed changes
    pub force: bool,
    /// Skip confirmation prompt
    pub yes: bool,
    /// Repository for workspace operations
    pub workspace_repository: &'a dyn WorkspaceRepository,
    /// Repository for tmux session operations
    pub session_repository: &'a dyn TmuxSessionRepository,
    /// Repository for tmux client operations
    pub client_repository: &'a dyn TmuxClientRepository,
    /// Repository for tmux popup operations
    pub popup_repository: &'a dyn TmuxPopupRepository,
    /// Repository for session descriptions (to create workspace sessions)
    pub description_repository: &'a dyn SessionDescriptionRepository,
}

/// Result of the worktree complete command
pub enum WorktreeCompleteResult {
    /// Worktree was removed successfully
    Success {
        branch_name: String,
        worktree_path: String,
    },
    /// Worktree removal was delegated to a popup
    Delegated {
        branch_name: String,
        target_session: String,
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
                println!("Completed worktree for branch '{}'", branch_name);
                println!("Removed: {}", worktree_path);
            }
            WorktreeCompleteResult::Delegated {
                branch_name,
                target_session,
            } => {
                println!(
                    "Cleanup for '{}' is running in popup on session '{}'",
                    branch_name, target_session
                );
            }
            WorktreeCompleteResult::Failed(err) => {
                eprintln!("Error: {}", err);
                exit(1);
            }
        }
    }
}

impl WorktreeCompleteCommand {
    fn execute_internal(&self, options: WorktreeCompleteOptions) -> WorktreeCompleteResult {
        // ===== PHASE 1: PRE-FLIGHT CHECKS =====

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

        // 2. Determine the worktree path and branch name
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

        // ===== PHASE 2: DETERMINE EXECUTION FLOW =====

        let current_session = get_current_tmux_session();
        let target_session_name = calculate_worktree_session_name(workspace, &branch_name);
        let is_self_deletion = current_session.as_ref() == Some(&target_session_name);

        if is_self_deletion {
            // Flow 1: We're in the worktree session being deleted - delegate to popup
            delegate_to_popup(
                workspace,
                &branch_name,
                options.force,
                options.yes,
                options.session_repository,
                options.popup_repository,
                options.description_repository,
                options.client_repository,
            )
        } else {
            // Flow 2: We're in a different session - execute cleanup directly
            execute_cleanup_directly(
                workspace,
                &worktree_path,
                &main_repo_path,
                &branch_name,
                options.force,
                options.yes,
                &current_dir,
                options.session_repository,
                options.client_repository,
            )
        }
    }
}

/// Delegate worktree cleanup to a popup in the main workspace session.
/// This is used when running from within the worktree session that's being deleted.
#[allow(clippy::too_many_arguments)]
fn delegate_to_popup(
    workspace: Option<&Workspace>,
    branch_name: &str,
    force: bool,
    yes: bool,
    session_repository: &dyn TmuxSessionRepository,
    popup_repository: &dyn TmuxPopupRepository,
    description_repository: &dyn SessionDescriptionRepository,
    client_repository: &dyn TmuxClientRepository,
) -> WorktreeCompleteResult {
    // 1. Get main workspace session name
    let main_session_name = if let Some(ws) = workspace {
        ws.name.clone()
    } else {
        return WorktreeCompleteResult::Failed(WorktreeError::GitError(
            "Cannot create popup: no workspace found for this worktree".to_string(),
        ));
    };

    // 2. Ask for confirmation (unless --yes)
    if !yes {
        println!("About to delete worktree for branch '{}'", branch_name);
        print!("Continue? [y/N] ");

        // Flush stdout to ensure prompt is displayed
        if io::stdout().flush().is_err() {
            return WorktreeCompleteResult::Failed(WorktreeError::GitError(
                "Failed to flush stdout".to_string(),
            ));
        }

        let stdin = io::stdin();
        let mut response = String::new();
        if stdin.lock().read_line(&mut response).is_err() {
            return WorktreeCompleteResult::Failed(WorktreeError::GitError(
                "Failed to read user input".to_string(),
            ));
        }

        if !response.trim().eq_ignore_ascii_case("y") {
            return WorktreeCompleteResult::Failed(WorktreeError::GitError(
                "Cancelled by user".to_string(),
            ));
        }
    }

    // 3. Ensure main workspace session exists
    if let Err(e) = ensure_workspace_session_exists(
        workspace.unwrap(),
        session_repository,
        description_repository,
    ) {
        return WorktreeCompleteResult::Failed(e);
    }

    // 4. Switch client to main workspace session (so user sees the popup there)
    switch_to_main_workspace_session(session_repository, client_repository, &main_session_name);

    // 5. Build cleanup command
    let mut command_parts = vec![
        "rafaeltab".to_string(),
        "worktree".to_string(),
        "complete".to_string(),
        branch_name.to_string(),
    ];

    if force {
        command_parts.push("--force".to_string());
    }

    // Always add --yes when delegating to avoid double confirmation
    command_parts.push("--yes".to_string());

    let command = command_parts.join(" ");

    // 6. Display popup
    let popup_options = PopupOptions {
        target_session: main_session_name.clone(),
        command,
        width: Some("80%".to_string()),
        height: Some("80%".to_string()),
        title: Some(format!("Completing worktree: {}", branch_name)),
    };

    if let Err(e) = popup_repository.display_popup(&popup_options) {
        return WorktreeCompleteResult::Failed(WorktreeError::GitError(format!(
            "Failed to create popup: {}",
            e
        )));
    }

    WorktreeCompleteResult::Delegated {
        branch_name: branch_name.to_string(),
        target_session: main_session_name,
    }
}

/// Execute worktree cleanup directly in the current session.
/// This is used when running from a different session than the worktree being deleted.
#[allow(clippy::too_many_arguments)]
fn execute_cleanup_directly(
    workspace: Option<&Workspace>,
    worktree_path: &Path,
    main_repo_path: &Path,
    branch_name: &str,
    force: bool,
    yes: bool,
    current_dir: &Path,
    session_repository: &dyn TmuxSessionRepository,
    client_repository: &dyn TmuxClientRepository,
) -> WorktreeCompleteResult {
    // 1. Confirmation (unless --yes)
    if !yes {
        println!("About to delete worktree for branch '{}'", branch_name);
        println!("Location: {}", worktree_path.display());
        print!("Continue? [y/N] ");

        // Flush stdout to ensure prompt is displayed
        if io::stdout().flush().is_err() {
            return WorktreeCompleteResult::Failed(WorktreeError::GitError(
                "Failed to flush stdout".to_string(),
            ));
        }

        let stdin = io::stdin();
        let mut response = String::new();
        if stdin.lock().read_line(&mut response).is_err() {
            return WorktreeCompleteResult::Failed(WorktreeError::GitError(
                "Failed to read user input".to_string(),
            ));
        }

        if !response.trim().eq_ignore_ascii_case("y") {
            return WorktreeCompleteResult::Failed(WorktreeError::GitError(
                "Cancelled by user".to_string(),
            ));
        }
    }

    // 2. Determine if we need to switch the client
    let should_switch_client = current_dir.starts_with(worktree_path);

    // 3. If we're in the worktree being deleted, switch client first
    if should_switch_client {
        if let Some(ws) = workspace {
            switch_to_main_workspace_session(session_repository, client_repository, &ws.name);
            println!("Switched to main workspace session");
        }
    }

    // 4. Kill the worktree's tmux session
    let session_name = calculate_worktree_session_name(workspace, branch_name);
    kill_session_by_name(session_repository, &session_name);
    println!("Closed tmux session: {}", session_name);

    // 5. Change directory away from worktree if needed
    if current_dir.starts_with(worktree_path) {
        if let Err(e) = std::env::set_current_dir(main_repo_path) {
            eprintln!("Warning: Could not change directory: {}", e);
        }
    }

    // 6. Remove the git worktree
    let remove_result = if force {
        git::force_remove_worktree(worktree_path)
    } else {
        git::remove_worktree(worktree_path)
    };

    if let Err(e) = remove_result {
        return WorktreeCompleteResult::Failed(WorktreeError::GitError(e.to_string()));
    }
    println!("Removed git worktree");

    // 7. Clean up empty parent directories
    if worktree_path.parent().is_some() {
        let stop_at = main_repo_path.parent().unwrap_or(main_repo_path);
        if let Err(e) = git::remove_empty_parent_directories(worktree_path, stop_at) {
            eprintln!("Warning: Could not clean up empty directories: {}", e);
        }
    }

    WorktreeCompleteResult::Success {
        branch_name: branch_name.to_string(),
        worktree_path: worktree_path.display().to_string(),
    }
}

/// Calculate the expected tmux session name for a worktree
fn calculate_worktree_session_name(workspace: Option<&Workspace>, branch_name: &str) -> String {
    if let Some(ws) = workspace {
        format!("{}-{}", ws.name, branch_name)
    } else {
        format!("worktree-{}", branch_name)
    }
}

/// Ensure that the workspace session exists, creating it if necessary
fn ensure_workspace_session_exists(
    workspace: &Workspace,
    session_repository: &dyn TmuxSessionRepository,
    description_repository: &dyn SessionDescriptionRepository,
) -> Result<(), WorktreeError> {
    use crate::domain::tmux_workspaces::aggregates::tmux::include_fields_builder::IncludeFieldsBuilder;

    // Check if session already exists
    let sessions =
        session_repository.get_sessions(None, IncludeFieldsBuilder::new().build_session());

    if sessions.iter().any(|s| s.name == workspace.name) {
        return Ok(()); // Session already exists
    }

    // Create the session using description repository
    let descriptions = description_repository.get_session_descriptions();
    let workspace_description = descriptions
        .iter()
        .find(|d| d.name == workspace.name)
        .ok_or_else(|| {
            WorktreeError::GitError(format!(
                "Could not find session description for workspace '{}'",
                workspace.name
            ))
        })?;

    session_repository.new_session(workspace_description);

    Ok(())
}

/// Find a worktree by its branch name
fn find_worktree_by_branch(
    current_dir: &Path,
    branch_name: &str,
) -> Result<(PathBuf, String), WorktreeError> {
    // First, get the root worktree to find all worktrees
    let root = git::get_root_worktree_path(current_dir)
        .map_err(|e| WorktreeError::GitError(e.to_string()))?;

    // List all worktrees
    let worktrees =
        git::list_worktrees(&root).map_err(|e| WorktreeError::GitError(e.to_string()))?;

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
        workspace_paths
            .iter()
            .map(|(id, path)| (*id, path.as_str())),
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
    let sessions =
        session_repository.get_sessions(None, IncludeFieldsBuilder::new().build_session());

    // Find the main workspace session (the one without branch suffix)
    if let Some(main_session) = sessions.iter().find(|s| s.name == workspace_name) {
        client_repository.switch_client(None, SwitchClientTarget::Session(main_session));
    }
}

/// Kill a tmux session by name
fn kill_session_by_name(session_repository: &dyn TmuxSessionRepository, session_name: &str) {
    use crate::domain::tmux_workspaces::aggregates::tmux::include_fields_builder::IncludeFieldsBuilder;

    // Get all sessions
    let sessions =
        session_repository.get_sessions(None, IncludeFieldsBuilder::new().build_session());

    // Find and kill the session
    if let Some(session) = sessions.iter().find(|s| s.name == session_name) {
        session_repository.kill_session(Some(session));
    }
}
