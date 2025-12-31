//! Command to start a new git worktree

use std::path::Path;

use duct::cmd;
use inquire::Confirm;

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
            config::{
                calculate_worktree_path, find_most_specific_workspace, BranchStatus,
                MergedWorktreeConfig, WorktreeCreationInfo,
            },
            error::WorktreeError,
        },
    },
    infrastructure::git::{self, symlink::create_symlinks, BranchLocation, GitError},
    storage::worktree::WorktreeStorage,
    utils::path::expand_path,
};

#[derive(Default)]
pub struct WorktreeStartCommand;

pub struct WorktreeStartOptions<'a> {
    /// The branch name for the new worktree
    pub branch_name: String,
    /// Force creation even without worktree config
    pub force: bool,
    /// Skip confirmation prompt
    pub yes: bool,
    /// Repository for workspace operations
    pub workspace_repository: &'a dyn WorkspaceRepository,
    /// Storage for global worktree config
    pub worktree_storage: &'a dyn WorktreeStorage,
    /// Repository for tmux session operations
    pub session_repository: &'a dyn TmuxSessionRepository,
    /// Repository for tmux client operations
    pub client_repository: &'a dyn TmuxClientRepository,
}

/// Result of the worktree start command
pub enum WorktreeStartResult {
    /// Worktree was created successfully and tmux session was started
    Success {
        worktree_path: String,
        session_name: String,
    },
    /// Worktree was created but onCreate commands failed
    PartialSuccess {
        worktree_path: String,
        session_name: String,
        failed_command: String,
        error: String,
    },
    /// Operation was cancelled by user
    Cancelled,
    /// Operation failed with error
    Failed(WorktreeError),
}

impl RafaeltabCommand<WorktreeStartOptions<'_>> for WorktreeStartCommand {
    fn execute(&self, options: WorktreeStartOptions) {
        let branch_name = options.branch_name.clone();
        match self.execute_internal(options) {
            WorktreeStartResult::Success {
                worktree_path,
                session_name,
            } => {
                println!("✓ Created worktree at {}", worktree_path);
                println!("✓ Started tmux session: {}", session_name);
            }
            WorktreeStartResult::PartialSuccess {
                worktree_path,
                session_name,
                failed_command,
                error,
            } => {
                println!("✓ Created worktree at {}", worktree_path);
                println!("✓ Created tmux session: {} (not switched)", session_name);
                println!();
                println!("⚠ onCreate command failed: {}", failed_command);
                println!("  Error: {}", error);
                println!();
                println!("The worktree was created but setup is incomplete.");
                println!("Fix the issue and run the remaining commands manually,");
                println!(
                    "or use 'rafaeltab worktree complete {}' to remove it.",
                    branch_name
                );
            }
            WorktreeStartResult::Cancelled => {
                println!("Operation cancelled.");
            }
            WorktreeStartResult::Failed(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}

impl WorktreeStartCommand {
    fn execute_internal(&self, options: WorktreeStartOptions) -> WorktreeStartResult {
        // 1. Get current directory
        let current_dir = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                return WorktreeStartResult::Failed(WorktreeError::GitError(format!(
                    "Failed to get current directory: {}",
                    e
                )));
            }
        };

        // 2. Find the workspace for the current directory
        let workspaces = options.workspace_repository.get_workspaces();
        let workspace = match find_workspace_for_path(&current_dir, &workspaces) {
            Some(ws) => ws,
            None => {
                return WorktreeStartResult::Failed(WorktreeError::NotInWorkspace);
            }
        };

        // 3. Get the workspace root path
        let workspace_root = expand_path(&workspace.path);
        let workspace_root_path = Path::new(&workspace_root);

        // 4. Verify we're in a git repository
        let git_root = match git::get_root_worktree_path(workspace_root_path) {
            Ok(root) => root,
            Err(GitError::NotInGitRepo(path)) => {
                return WorktreeStartResult::Failed(WorktreeError::NotInGitRepo(path));
            }
            Err(e) => {
                return WorktreeStartResult::Failed(WorktreeError::GitError(e.to_string()));
            }
        };

        // 5. Check for worktree configuration
        let global_config = options.worktree_storage.read();
        let workspace_config =
            find_workspace_worktree_config(&workspace.id, options.workspace_repository);

        let has_config = global_config.is_some() || workspace_config.is_some();

        if !has_config && !options.force {
            return WorktreeStartResult::Failed(WorktreeError::WorktreeConfigMissing {
                workspace_name: workspace.name.clone(),
            });
        }

        // 6. Merge configurations
        let merged_config =
            MergedWorktreeConfig::merge(global_config.as_ref(), workspace_config.as_ref());

        // 7. Get current branch (base branch)
        let base_branch = match git::get_current_branch(&git_root) {
            Ok(branch) => branch,
            Err(GitError::DetachedHead) => {
                return WorktreeStartResult::Failed(WorktreeError::GitError(
                    "Cannot create worktree from detached HEAD state".to_string(),
                ));
            }
            Err(e) => {
                return WorktreeStartResult::Failed(WorktreeError::GitError(e.to_string()));
            }
        };

        // 8. Check if branch already exists
        let branch_location = git::get_branch_location(&git_root, &options.branch_name);
        let branch_status = match &branch_location {
            BranchLocation::Local => BranchStatus::ExistsLocally,
            BranchLocation::Remote(remote) => BranchStatus::ExistsRemotely(remote.clone()),
            BranchLocation::None => BranchStatus::New,
        };

        // 9. Calculate worktree path
        let worktree_path = calculate_worktree_path(&git_root, &options.branch_name);

        // 10. Check if path already exists
        if worktree_path.exists() {
            return WorktreeStartResult::Failed(WorktreeError::PathConflict(worktree_path));
        }

        // 11. Build creation info for confirmation
        let creation_info = WorktreeCreationInfo {
            branch_name: options.branch_name.clone(),
            base_branch: base_branch.clone(),
            branch_status,
            worktree_path: worktree_path.clone(),
            config: merged_config.clone(),
            workspace_name: workspace.name.clone(),
        };

        // 12. Show confirmation prompt (unless --yes)
        if !options.yes {
            println!();
            println!("Creating worktree:");
            println!("  Branch: {}", creation_info.branch_name);
            println!("  Base branch: {} (current)", creation_info.base_branch);
            println!("  Status: {}", creation_info.branch_status);
            println!("  Path: {}", creation_info.worktree_path.display());
            println!(
                "  Symlinks: {} patterns ({})",
                creation_info.config.symlink_files.len(),
                creation_info.config.symlink_files.join(", ")
            );
            println!(
                "  onCreate: {} commands ({})",
                creation_info.config.on_create.len(),
                creation_info.config.on_create.join(", ")
            );
            println!();

            let confirmed = Confirm::new("Continue?").with_default(true).prompt();

            match confirmed {
                Ok(true) => {}
                Ok(false) | Err(_) => {
                    return WorktreeStartResult::Cancelled;
                }
            }
        }

        // 13. Create the worktree
        if let Err(e) = git::create_worktree(
            &git_root,
            &options.branch_name,
            &worktree_path,
            &branch_location,
            Some(&base_branch),
        ) {
            return WorktreeStartResult::Failed(WorktreeError::GitError(e.to_string()));
        }
        println!("✓ Created git worktree");

        // 14. Create symlinks
        if !merged_config.symlink_files.is_empty() {
            match create_symlinks(&git_root, &worktree_path, &merged_config.symlink_files) {
                Ok(result) => {
                    if !result.created.is_empty() {
                        println!(
                            "✓ Created {} symlinks: {}",
                            result.created.len(),
                            result
                                .created
                                .iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                    if !result.skipped.is_empty() {
                        println!(
                            "  Skipped {} files (already exist or not found)",
                            result.skipped.len()
                        );
                    }
                }
                Err(e) => {
                    // Symlink failure is not fatal - continue but warn
                    println!("⚠ Symlink creation had issues: {}", e);
                }
            }
        }

        // 15. Run onCreate commands
        let mut on_create_failed: Option<(String, String)> = None;
        for command in &merged_config.on_create {
            println!("  Running: {}", command);
            let result = cmd!("sh", "-c", command)
                .dir(&worktree_path)
                .stderr_to_stdout()
                .read();

            match result {
                Ok(output) => {
                    if !output.trim().is_empty() {
                        // Print output indented
                        for line in output.lines() {
                            println!("    {}", line);
                        }
                    }
                    println!("  ✓ Completed: {}", command);
                }
                Err(e) => {
                    on_create_failed = Some((command.clone(), e.to_string()));
                    break;
                }
            }
        }

        // 16. Create tmux session
        let session_name = format!("{}-{}", workspace.name, options.branch_name);

        // Create a session description for the worktree
        let session =
            create_tmux_session(options.session_repository, &session_name, &worktree_path);

        // 17. If onCreate failed, don't switch to session
        if let Some((failed_cmd, error)) = on_create_failed {
            return WorktreeStartResult::PartialSuccess {
                worktree_path: worktree_path.display().to_string(),
                session_name,
                failed_command: failed_cmd,
                error,
            };
        }

        // 18. Switch to the new tmux session
        if let Some(ref sess) = session {
            options
                .client_repository
                .switch_client(None, SwitchClientTarget::Session(sess));
        }

        WorktreeStartResult::Success {
            worktree_path: worktree_path.display().to_string(),
            session_name,
        }
    }
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

/// Find the worktree config for a workspace by ID
fn find_workspace_worktree_config(
    workspace_id: &str,
    workspace_repository: &dyn WorkspaceRepository,
) -> Option<crate::storage::worktree::WorkspaceWorktreeConfig> {
    workspace_repository
        .get_workspaces()
        .iter()
        .find(|ws| ws.id == workspace_id)
        .and_then(|ws| ws.worktree.clone())
}

/// Create a tmux session for the worktree
fn create_tmux_session(
    session_repository: &dyn TmuxSessionRepository,
    session_name: &str,
    worktree_path: &Path,
) -> Option<crate::domain::tmux_workspaces::aggregates::tmux::session::TmuxSession> {
    use crate::domain::tmux_workspaces::aggregates::tmux::description::{
        session::{PathSessionDescription, SessionDescription, SessionKind},
        window::WindowDescription,
    };
    use uuid::{uuid, Uuid};

    let worktree_namespace = uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479");
    let id = Uuid::new_v5(&worktree_namespace, session_name.as_bytes());

    let description = SessionDescription {
        id: id.to_string(),
        name: session_name.to_string(),
        kind: SessionKind::Path(PathSessionDescription {
            path: worktree_path.to_string_lossy().to_string(),
        }),
        windows: vec![
            WindowDescription {
                name: "neovim".to_string(),
                command: Some("nvim .".to_string()),
            },
            WindowDescription {
                name: "shell".to_string(),
                command: None,
            },
        ],
        session: None,
    };

    Some(session_repository.new_session(&description))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::worktree::config::MergedWorktreeConfig;
    use crate::infrastructure::tmux_workspaces::repositories::workspace::workspace_repository::ImplWorkspaceRepository;
    use crate::storage::{
        test::mocks::MockWorkspaceStorage,
        workspace::Workspace,
        worktree::{WorkspaceWorktreeConfig, WorktreeConfig},
    };

    #[test]
    fn test_find_workspace_worktree_config_returns_config() {
        let worktree_config = WorkspaceWorktreeConfig {
            symlink_files: vec![".env.local".to_string()],
            on_create: vec!["pnpm install".to_string()],
        };

        let workspace_storage = MockWorkspaceStorage {
            data: vec![
                Workspace {
                    id: "workspace-1".to_string(),
                    root: "~/test1".to_string(),
                    name: "Test 1".to_string(),
                    tags: None,
                    worktree: None,
                },
                Workspace {
                    id: "workspace-with-config".to_string(),
                    root: "~/test2".to_string(),
                    name: "Test 2".to_string(),
                    tags: None,
                    worktree: Some(worktree_config.clone()),
                },
            ],
        };

        let workspace_repository = ImplWorkspaceRepository {
            workspace_storage: &workspace_storage,
        };

        let result = find_workspace_worktree_config("workspace-with-config", &workspace_repository);

        assert!(result.is_some());
        let config = result.unwrap();
        assert_eq!(config.symlink_files.len(), 1);
        assert_eq!(config.symlink_files[0], ".env.local");
        assert_eq!(config.on_create.len(), 1);
        assert_eq!(config.on_create[0], "pnpm install");
    }

    #[test]
    fn test_find_workspace_worktree_config_returns_none_when_missing() {
        let workspace_storage = MockWorkspaceStorage {
            data: vec![Workspace {
                id: "workspace-no-config".to_string(),
                root: "~/test".to_string(),
                name: "Test".to_string(),
                tags: None,
                worktree: None,
            }],
        };

        let workspace_repository = ImplWorkspaceRepository {
            workspace_storage: &workspace_storage,
        };

        let result = find_workspace_worktree_config("workspace-no-config", &workspace_repository);

        assert!(result.is_none());
    }

    #[test]
    fn test_find_workspace_worktree_config_returns_none_for_nonexistent_workspace() {
        let workspace_storage = MockWorkspaceStorage {
            data: vec![Workspace {
                id: "workspace-1".to_string(),
                root: "~/test".to_string(),
                name: "Test".to_string(),
                tags: None,
                worktree: None,
            }],
        };

        let workspace_repository = ImplWorkspaceRepository {
            workspace_storage: &workspace_storage,
        };

        let result = find_workspace_worktree_config("nonexistent-workspace", &workspace_repository);

        assert!(result.is_none());
    }

    #[test]
    fn test_workspace_config_merges_correctly_with_global_config() {
        // Setup workspace with specific config
        let workspace_config = WorkspaceWorktreeConfig {
            symlink_files: vec![".env.local".to_string()],
            on_create: vec!["pnpm install".to_string()],
        };

        let workspace_storage = MockWorkspaceStorage {
            data: vec![Workspace {
                id: "test-workspace".to_string(),
                root: "~/test".to_string(),
                name: "Test Workspace".to_string(),
                tags: None,
                worktree: Some(workspace_config),
            }],
        };

        let workspace_repository = ImplWorkspaceRepository {
            workspace_storage: &workspace_storage,
        };

        // Setup global config
        let global_config = WorktreeConfig {
            symlink_files: vec![".env".to_string(), "config.json".to_string()],
            on_create: vec!["npm ci".to_string()],
        };

        // Get workspace config and merge with global
        let ws_config = find_workspace_worktree_config("test-workspace", &workspace_repository);
        let merged = MergedWorktreeConfig::merge(Some(&global_config), ws_config.as_ref());

        // Verify merged config has both global and workspace items
        assert_eq!(merged.symlink_files.len(), 3);
        assert!(merged.symlink_files.contains(&".env".to_string()));
        assert!(merged.symlink_files.contains(&"config.json".to_string()));
        assert!(merged.symlink_files.contains(&".env.local".to_string()));

        assert_eq!(merged.on_create.len(), 2);
        assert!(merged.on_create.contains(&"npm ci".to_string()));
        assert!(merged.on_create.contains(&"pnpm install".to_string()));
    }
}
