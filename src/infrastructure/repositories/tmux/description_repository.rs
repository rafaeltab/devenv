use uuid::{uuid, Uuid};

use crate::{
    config::{Config, Session},
    domain::{
        aggregates::tmux::{
            description::{
                session::{PathSessionDescription, SessionDescription, SessionKind},
                window::WindowDescription,
            },
            include_fields_builder::IncludeFieldsBuilder,
        },
        repositories::{
            tmux::{
                description_repository::SessionDescriptionRepository,
                session_repository::TmuxSessionRepository,
            },
            workspace::workspace_repository::WorkspaceRepository,
        },
    },
};

pub struct ImplDescriptionRepository<'a> {
    pub workspace_repository: &'a dyn WorkspaceRepository,
    pub session_repository: &'a dyn TmuxSessionRepository,
    pub config: Config,
}

impl<'a> SessionDescriptionRepository for ImplDescriptionRepository<'a> {
    fn get_session_descriptions(&self) -> Vec<SessionDescription> {
        let workspaces = self.workspace_repository.get_workspaces();
        let mut result: Vec<SessionDescription> = vec![];
        let default_window_descriptions: Vec<WindowDescription> = self
            .config
            .tmux
            .default_windows
            .iter()
            .map(|x| WindowDescription {
                name: x.name.clone(),
                command: x.command.clone(),
            })
            .collect();

        let workspace_namespace = uuid!("dd66ca72-805f-4efb-85cc-f235a925d593");
        let path_namespace = uuid!("3598273a-f7fe-4588-b5a4-fef0ed1ab31b");

        for workspace in workspaces {
            let id = Uuid::new_v5(&workspace_namespace, workspace.id.as_bytes());
            result.push(SessionDescription {
                id: id.to_string(),
                name: workspace.name.clone(),
                windows: default_window_descriptions.clone(),
                kind: SessionKind::Workspace(workspace),
                session: None,
            });
        }

        for session in self.config.tmux.sessions.clone().unwrap_or_default() {
            match session {
                Session::Workspace(workspace) => {
                    let windows: Vec<WindowDescription> = workspace
                        .windows
                        .iter()
                        .map(|x| WindowDescription {
                            name: x.name.clone(),
                            command: x.command.clone(),
                        })
                        .collect();
                    // We already added this to the list so we just need to replace the windows
                    let res_workspace = result
                        .iter_mut()
                        .find(|x| match &x.kind {
                            SessionKind::Path(_) => false,
                            SessionKind::Workspace(w) => w.id == workspace.workspace,
                        })
                        .unwrap();
                    res_workspace.windows = windows;
                }
                Session::Path(path) => {
                    let id = Uuid::new_v5(&path_namespace, path.name.as_bytes());
                    result.push(SessionDescription {
                        id: id.to_string(),
                        name: path.name,
                        kind: SessionKind::Path(PathSessionDescription { path: path.path }),
                        windows: path
                            .windows
                            .iter()
                            .map(|x| WindowDescription {
                                name: x.name.clone(),
                                command: x.command.clone(),
                            })
                            .collect(),
                        session: None,
                    });
                }
            }
        }

        // Find and attach sessions!
        let sessions = self
            .session_repository
            .get_sessions(None, IncludeFieldsBuilder::new().build_session());

        for session in sessions {
            let env = self.session_repository.get_environment(&session.id);
            let id_opt = find_session_id(&env);
            if id_opt.is_none() {
                continue;
            }
            let id = id_opt.unwrap();
            let res_session = result.iter_mut().find(|x| x.id == id);
            if let Some(sess) = res_session {
                sess.session = Some(session)
            }
        }

        result
    }
}

fn find_session_id(input: &str) -> Option<String> {
    // Define the target identifier
    let target = "RAFAELTAB_SESSION_ID=";

    // Find the position of the target identifier
    if let Some(start_index) = input.find(target) {
        // Calculate the start index of the UUID
        let start_index = start_index + target.len();

        // Extract the substring that starts right after the target identifier
        let substring = &input[start_index..];

        // Find the end of the UUID by looking for the next newline or the end of the string
        let end_index = substring.find('\n').unwrap_or(substring.len());

        // Extract the UUID and trim any extraneous whitespace
        let uuid = &substring[..end_index].trim();

        // Return the UUID as a String
        return Some(uuid.to_string());
    }

    // Return None if the target identifier is not found
    None
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{Config, PathSession, Session, Tmux, Window, Workspace, WorkspaceSession},
        domain::{
            aggregates::tmux::description::session::{SessionDescription, SessionKind},
            repositories::{
                tmux::{
                    description_repository::SessionDescriptionRepository,
                    session_repository::TmuxSessionRepository,
                },
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
                shell: "zsh -c".to_string(),
            },
        }
    }

    fn workspace_repo_factory(config: Config) -> impl WorkspaceRepository {
        ImplWorkspaceRepository { config }
    }

    fn session_repo_factory() -> impl TmuxSessionRepository {
        MockSessionRepo {}
    }

    fn sut_factory<'a>(
        config: Config,
        workspace_repository: &'a dyn WorkspaceRepository,
        session_repository: &'a dyn TmuxSessionRepository,
    ) -> ImplDescriptionRepository<'a> {
        ImplDescriptionRepository {
            config,
            workspace_repository,
            session_repository,
        }
    }

    #[test]
    fn should_include_all_workspaces() {
        let config = config_factory();
        let workspace_repo = workspace_repo_factory(config.clone());
        let session_repository = session_repo_factory();
        let sut = sut_factory(config.clone(), &workspace_repo, &session_repository);

        let result = sut.get_session_descriptions();

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
        let session_repository = session_repo_factory();
        let sut = sut_factory(config.clone(), &workspace_repo, &session_repository);

        let result = sut.get_session_descriptions();

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
        let session_repository = session_repo_factory();
        let sut = sut_factory(config.clone(), &workspace_repo, &session_repository);

        let result = sut.get_session_descriptions();

        let home_session = result.iter().find(|x| x.name == "Home").unwrap();
        assert_eq!(home_session.windows.len(), 1);
    }

    #[test]
    fn should_apply_default_windows_to_workspaces() {
        let config = config_factory();
        let workspace_repo = workspace_repo_factory(config.clone());
        let session_repository = session_repo_factory();
        let sut = sut_factory(config.clone(), &workspace_repo, &session_repository);

        let result = sut.get_session_descriptions();

        let source_session = result.iter().find(|x| x.name == "Source").unwrap();
        assert_eq!(source_session.windows.len(), 2);
    }

    #[test]
    fn should_not_apply_workspaces_twice_when_defined_in_sessions() {
        let config = config_factory();
        let workspace_repo = workspace_repo_factory(config.clone());
        let session_repository = session_repo_factory();
        let sut = sut_factory(config.clone(), &workspace_repo, &session_repository);

        let result = sut.get_session_descriptions();

        let home_session: Vec<&SessionDescription> =
            result.iter().filter(|x| x.name == "Home").collect();
        assert_eq!(home_session.len(), 1);
    }

    struct MockSessionRepo {}

    impl TmuxSessionRepository for MockSessionRepo {
        fn new_session(
            &self,
            _description: &SessionDescription,
        ) -> crate::domain::aggregates::tmux::session::TmuxSession {
            todo!()
        }

        fn kill_session(
            &self,
            _session: Option<&crate::domain::aggregates::tmux::session::TmuxSession>,
        ) {
        }

        fn get_environment(&self, _session_id: &str) -> String {
            "".to_string()
        }

        fn get_sessions(
            &self,
            _filter: Option<crate::infrastructure::tmux::tmux_format::TmuxFilterNode>,
            _include: crate::domain::aggregates::tmux::session::SessionIncludeFields,
        ) -> Vec<crate::domain::aggregates::tmux::session::TmuxSession> {
            vec![]
        }
    }
}
