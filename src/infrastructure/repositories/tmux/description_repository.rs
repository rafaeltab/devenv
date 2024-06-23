use crate::{
    config::Config,
    domain::{
        aggregates::tmux::description::session::SessionDescription,
        repositories::{
            tmux::description_repository::SessionDescriptionRepository,
            workspace::workspace_repository::WorkspaceRepository,
        },
    },
};

pub struct ImplDescriptionRepository<'a> {
    workspace_repository: &'a dyn WorkspaceRepository,
    config: Config,
}

impl<'a> SessionDescriptionRepository for ImplDescriptionRepository<'a> {
    fn get_session_descriptions(&self) -> Vec<SessionDescription> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{Config, PathSession, Session, Tmux, Window, Workspace, WorkspaceSession},
        domain::{
            aggregates::tmux::description::session::{SessionDescription, SessionKind},
            repositories::{
                tmux::description_repository::SessionDescriptionRepository,
                workspace::workspace_repository::WorkspaceRepository,
            },
        },
        infrastructure::repositories::workspace::workspace_repository::ImplWorkspaceRepository,
    };

    use super::ImplDescriptionRepository;

    fn config_factory() -> Config {
        Config {
            workspaces: vec![
                Workspace {
                    name: "Home".to_string(),
                    id: "home".to_string(),
                    root: "~".to_string(),
                    tags: Some(vec![]),
                },
                Workspace {
                    name: "Source".to_string(),
                    id: "source".to_string(),
                    root: "~/source".to_string(),
                    tags: Some(vec![]),
                },
            ],
            tmux: Tmux {
                sessions: Some(vec![
                    Session::Path(PathSession {
                        windows: vec![Window {
                            name: "zsh".to_string(),
                            command: None,
                        }],
                        path: "/usr/bin".to_string(),
                        name: "User binaries".to_string(),
                    }),
                    Session::Workspace(WorkspaceSession {
                        windows: vec![Window {
                            name: "zsh".to_string(),
                            command: None,
                        }],
                        workspace: "home".to_string(),
                        name: None,
                    }),
                ]),
                default_windows: vec![
                    Window {
                        name: "Neovim".to_string(),
                        command: Some("nvim".to_string()),
                    },
                    Window {
                        name: "zsh".to_string(),
                        command: None,
                    },
                ],
            },
        }
    }

    fn workspace_repo_factory(config: Config) -> impl WorkspaceRepository {
        ImplWorkspaceRepository { config }
    }

    fn sut_factory(
        config: Config,
        workspace_repository: &dyn WorkspaceRepository,
    ) -> ImplDescriptionRepository {
        ImplDescriptionRepository {
            config,
            workspace_repository,
        }
    }

    #[test]
    fn should_include_all_workspaces() {
        let config = config_factory();
        let workspace_repo = workspace_repo_factory(config.clone());
        let sut = sut_factory(config.clone(), &workspace_repo);

        let result = sut.get_session_descriptions();

        assert_eq!(result.len(), 3);
        let workspace_sessions: Vec<&SessionDescription> = result
            .iter()
            .filter(|x| matches!(x.kind, SessionKind::Workspace(..)))
            .collect();
        assert_eq!(workspace_sessions.len(), 2);
    }

    #[test]
    fn should_include_all_path_sessions() {
        let config = config_factory();
        let workspace_repo = workspace_repo_factory(config.clone());
        let sut = sut_factory(config.clone(), &workspace_repo);

        let result = sut.get_session_descriptions();

        assert_eq!(result.len(), 3);
        let binaries_session = result
            .iter()
            .find(|x| !matches!(x.kind, SessionKind::Workspace(..)))
            .unwrap();
        assert_eq!(binaries_session.name, "User binaries");
    }

    #[test]
    fn should_use_session_definition_for_workspace_sessions() {
        let config = config_factory();
        let workspace_repo = workspace_repo_factory(config.clone());
        let sut = sut_factory(config.clone(), &workspace_repo);

        let result = sut.get_session_descriptions();

        assert_eq!(result.len(), 3);
        let home_session = result.iter().find(|x| x.name == "Home").unwrap();
        assert_eq!(home_session.windows.len(), 1);
    }

    #[test]
    fn should_apply_default_windows_to_workspaces() {
        let config = config_factory();
        let workspace_repo = workspace_repo_factory(config.clone());
        let sut = sut_factory(config.clone(), &workspace_repo);

        let result = sut.get_session_descriptions();

        assert_eq!(result.len(), 3);
        let source_session = result.iter().find(|x| x.name == "Source").unwrap();
        assert_eq!(source_session.windows.len(), 2);
    }

    #[test]
    fn should_not_apply_workspaces_twice_when_defined_in_sessions() {
        let config = config_factory();
        let workspace_repo = workspace_repo_factory(config.clone());
        let sut = sut_factory(config.clone(), &workspace_repo);

        let result = sut.get_session_descriptions();

        assert_eq!(result.len(), 3);
        let home_session: Vec<&SessionDescription> =
            result.iter().filter(|x| x.name == "Home").collect();
        assert_eq!(home_session.len(), 1);
    }
}
