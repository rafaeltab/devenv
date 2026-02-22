//! Utility service for tmux session management

use std::sync::Arc;

use shaku::{Component, Interface};

use crate::{
    domain::tmux_workspaces::aggregates::tmux::description::window::WindowDescription,
    domain::tmux_workspaces::repositories::tmux::session_repository::TmuxSessionRepository,
    storage::tmux::{Session, TmuxStorage},
};

/// Interface for tmux session utility operations.
pub trait SessionUtilsService: Interface {
    /// Create tmux sessions for all worktrees in a workspace.
    /// This runs after the main workspace session has been created.
    fn create_worktree_sessions(
        &self,
        workspace: &crate::domain::tmux_workspaces::aggregates::workspaces::workspace::Workspace,
    );

    /// Get window configuration for a workspace session.
    /// Returns workspace-specific windows if configured, otherwise returns default windows.
    fn get_windows_for_workspace(&self, workspace_id: &str) -> Vec<WindowDescription>;
}

#[derive(Component)]
#[shaku(interface = SessionUtilsService)]
pub struct ImplSessionUtilsService {
    #[shaku(inject)]
    session_repository: Arc<dyn TmuxSessionRepository>,
    #[shaku(inject)]
    tmux_storage: Arc<dyn TmuxStorage>,
}

impl SessionUtilsService for ImplSessionUtilsService {
    fn create_worktree_sessions(
        &self,
        workspace: &crate::domain::tmux_workspaces::aggregates::workspaces::workspace::Workspace,
    ) {
        create_worktree_sessions_impl(
            workspace,
            &*self.session_repository,
            &*self.tmux_storage,
        );
    }

    fn get_windows_for_workspace(&self, workspace_id: &str) -> Vec<WindowDescription> {
        get_windows_for_workspace(workspace_id, &*self.tmux_storage)
    }
}

/// Create tmux sessions for all worktrees in a workspace.
/// This runs after the main workspace session has been created.
/// Errors are silently ignored (TODO: add logging when available).
fn create_worktree_sessions_impl(
    workspace: &crate::domain::tmux_workspaces::aggregates::workspaces::workspace::Workspace,
    session_repository: &dyn TmuxSessionRepository,
    tmux_storage: &dyn TmuxStorage,
) {
    use crate::domain::tmux_workspaces::aggregates::tmux::description::session::{
        PathSessionDescription, SessionDescription, SessionKind,
    };
    use crate::domain::tmux_workspaces::aggregates::tmux::include_fields_builder::IncludeFieldsBuilder;
    use crate::infrastructure::git;
    use crate::utils::path::expand_path;
    use std::path::Path;
    use uuid::{uuid, Uuid};

    let workspace_path = expand_path(&workspace.path);
    let workspace_path = Path::new(&workspace_path);

    // Try to discover worktrees (silently fail if not a git repo)
    let worktrees = match git::discover_worktrees_for_workspace(workspace_path) {
        Ok(wts) => wts,
        Err(_) => {
            // TODO: Log this error when logging infrastructure is available
            return;
        }
    };

    if worktrees.is_empty() {
        return; // No worktrees to create sessions for
    }

    // Get window configuration for this workspace
    let windows = get_windows_for_workspace(&workspace.id, tmux_storage);

    // Create session for each worktree
    for worktree_info in worktrees {
        let session_name = format!("{}-{}", workspace.name, worktree_info.branch);

        // Check if session already exists
        let existing_sessions =
            session_repository.get_sessions(None, IncludeFieldsBuilder::new().build_session());
        let session_exists = existing_sessions.iter().any(|s| s.name == session_name);
        if session_exists {
            continue;
        }

        // Create session description
        let worktree_namespace = uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479");
        let id = Uuid::new_v5(&worktree_namespace, session_name.as_bytes());

        let description = SessionDescription {
            id: id.to_string(),
            name: session_name,
            kind: SessionKind::Path(PathSessionDescription {
                path: worktree_info.path.to_string_lossy().to_string(),
            }),
            windows: windows.clone(),
            session: None,
        };

        // Create the session (ignore errors silently for now)
        // TODO: Log creation errors when logging infrastructure is available
        let _result = session_repository.new_session(&description);
    }
}

/// Get window configuration for a workspace session.
/// Returns workspace-specific windows if configured, otherwise returns default windows.
pub fn get_windows_for_workspace(
    workspace_id: &str,
    tmux_storage: &dyn TmuxStorage,
) -> Vec<WindowDescription> {
    let tmux_config = tmux_storage.read();

    // Check if workspace has custom session config
    if let Some(sessions) = &tmux_config.sessions {
        for session in sessions {
            if let Session::Workspace(ws_session) = session
                && ws_session.workspace == workspace_id {
                    return ws_session
                        .windows
                        .iter()
                        .map(|w| WindowDescription {
                            name: w.name.clone(),
                            command: w.command.clone(),
                        })
                        .collect();
                }
        }
    }

    // Fall back to default windows
    tmux_config
        .default_windows
        .iter()
        .map(|w| WindowDescription {
            name: w.name.clone(),
            command: w.command.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{
        test::mocks::MockTmuxStorage,
        tmux::{Tmux, Window, WorkspaceSession},
    };

    #[test]
    fn test_returns_default_windows_when_no_workspace_config() {
        let storage = MockTmuxStorage {
            data: Tmux {
                sessions: None,
                default_windows: vec![
                    Window {
                        name: "editor".to_string(),
                        command: Some("vim".to_string()),
                    },
                    Window {
                        name: "shell".to_string(),
                        command: None,
                    },
                ],
            },
        };

        let result = get_windows_for_workspace("test-workspace", &storage);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "editor");
        assert!(result[0]
            .command
            .as_ref()
            .is_some_and(|c| c.contains("vim")));
        assert_eq!(result[1].name, "shell");
        assert_eq!(result[1].command, None);
    }

    #[test]
    fn test_returns_workspace_config_when_exists() {
        let storage = MockTmuxStorage {
            data: Tmux {
                sessions: Some(vec![Session::Workspace(WorkspaceSession {
                    workspace: "my-workspace".to_string(),
                    name: None,
                    windows: vec![
                        Window {
                            name: "nvim".to_string(),
                            command: Some("nvim .".to_string()),
                        },
                        Window {
                            name: "build".to_string(),
                            command: Some("npm run dev".to_string()),
                        },
                    ],
                })]),
                default_windows: vec![Window {
                    name: "default".to_string(),
                    command: None,
                }],
            },
        };

        let result = get_windows_for_workspace("my-workspace", &storage);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "nvim");
        assert!(result[0]
            .command
            .as_ref()
            .is_some_and(|c| c.contains("nvim .")));
        assert_eq!(result[1].name, "build");
        assert!(result[1]
            .command
            .as_ref()
            .is_some_and(|c| c.contains("npm run dev")));
    }

    #[test]
    fn test_returns_default_for_different_workspace() {
        let storage = MockTmuxStorage {
            data: Tmux {
                sessions: Some(vec![Session::Workspace(WorkspaceSession {
                    workspace: "workspace-a".to_string(),
                    name: None,
                    windows: vec![Window {
                        name: "custom".to_string(),
                        command: None,
                    }],
                })]),
                default_windows: vec![Window {
                    name: "default".to_string(),
                    command: None,
                }],
            },
        };

        let result = get_windows_for_workspace("workspace-b", &storage);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "default");
    }

    #[test]
    fn test_handles_empty_default_windows() {
        let storage = MockTmuxStorage {
            data: Tmux {
                sessions: None,
                default_windows: vec![],
            },
        };

        let result = get_windows_for_workspace("test-workspace", &storage);

        assert_eq!(result.len(), 0);
    }
}
