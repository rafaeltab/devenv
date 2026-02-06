//! Utility functions for tmux session management

use crate::{
    domain::tmux_workspaces::aggregates::tmux::description::window::WindowDescription,
    storage::tmux::{Session, TmuxStorage},
};

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
            if let Session::Workspace(ws_session) = session {
                if ws_session.workspace == workspace_id {
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
            .map_or(false, |c| c.contains("vim")));
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
            .map_or(false, |c| c.contains("nvim .")));
        assert_eq!(result[1].name, "build");
        assert!(result[1]
            .command
            .as_ref()
            .map_or(false, |c| c.contains("npm run dev")));
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
