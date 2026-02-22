use std::sync::Arc;

use shaku::Component;
use uuid::{uuid, Uuid};

use crate::{
    di::ConfigPathProvider,
    domain::tmux_workspaces::{
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
    storage::{kinds::json_storage::JsonStorage, storage_interface::Storage, tmux::Session},
};

#[derive(Component)]
#[shaku(interface = SessionDescriptionRepository)]
pub struct ImplDescriptionRepository {
    #[shaku(inject)]
    workspace_repository: Arc<dyn WorkspaceRepository>,
    #[shaku(inject)]
    session_repository: Arc<dyn TmuxSessionRepository>,
    #[shaku(inject)]
    config_path_provider: Arc<dyn ConfigPathProvider>,
}

impl ImplDescriptionRepository {
    fn get_storage(&self) -> JsonStorage {
        let config_path = self.config_path_provider.path().to_string();
        JsonStorage::new(crate::storage::kinds::json_storage::JsonStorageParameters { config_path })
            .expect("Failed to load storage")
    }

    /// Constructor for testing purposes
    #[cfg(test)]
    pub fn new(
        workspace_repository: Arc<dyn WorkspaceRepository>,
        session_repository: Arc<dyn TmuxSessionRepository>,
        config_path_provider: Arc<dyn ConfigPathProvider>,
    ) -> Self {
        Self {
            workspace_repository,
            session_repository,
            config_path_provider,
        }
    }
}

impl SessionDescriptionRepository for ImplDescriptionRepository {
    fn get_session_descriptions(&self) -> Vec<SessionDescription> {
        let storage = self.get_storage();
        let workspaces = self.workspace_repository.get_workspaces();
        let mut result: Vec<SessionDescription> = vec![];
        let tmux_data: crate::storage::tmux::Tmux = storage.read();
        let default_window_descriptions: Vec<WindowDescription> = tmux_data
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

        for session in tmux_data.sessions.clone().unwrap_or_default() {
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
    use std::sync::Arc;

    use crate::{
        domain::tmux_workspaces::{
            aggregates::tmux::description::session::{SessionDescription, SessionKind},
            repositories::tmux::{
                description_repository::SessionDescriptionRepository,
                session_repository::TmuxSessionRepository,
            },
        },
        storage::{
            tmux::{PathSession, Session, Tmux, Window, WorkspaceSession},
            workspace::Workspace,
        },
    };

    use super::ImplDescriptionRepository;
    use crate::di::{ConfigPathOption, ConfigPathProvider};

    fn create_test_config(workspaces: Vec<Workspace>, tmux: Tmux) -> (String, tempfile::TempDir) {
        use crate::storage::kinds::json_storage::JsonData;
        use std::io::Write;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("rafaeltab.json");

        let config = JsonData {
            workspaces,
            tmux,
            worktree: None,
        };

        let config_str = serde_json::to_string(&config).expect("Failed to serialize config");
        let mut file = std::fs::File::create(&config_path).expect("Failed to create config file");
        file.write_all(config_str.as_bytes())
            .expect("Failed to write config");

        (config_path.to_string_lossy().to_string(), temp_dir)
    }

    fn workspaces_factory() -> Vec<Workspace> {
        vec![
            Workspace {
                name: "Home".to_string(),
                id: "home".to_string(),
                root: "~".to_string(),
                tags: Some(vec![]),
                worktree: None,
            },
            Workspace {
                name: "Source".to_string(),
                id: "source".to_string(),
                root: "~/source".to_string(),
                tags: Some(vec![]),
                worktree: None,
            },
        ]
    }

    fn tmux_factory() -> Tmux {
        Tmux {
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
        }
    }

    fn sut_factory(
        workspaces: Vec<Workspace>,
        tmux: Tmux,
    ) -> (ImplDescriptionRepository, tempfile::TempDir) {
        let (config_path, temp_dir) = create_test_config(workspaces, tmux);

        let repo = ImplDescriptionRepository::new(
            Arc::new(crate::infrastructure::tmux_workspaces::repositories::workspace::workspace_repository::ImplWorkspaceRepository::with_config_path(
                Arc::new(ConfigPathOption {
                    path: config_path.clone(),
                }) as Arc<dyn ConfigPathProvider>
            )),
            Arc::new(MockSessionRepo {}),
            Arc::new(ConfigPathOption {
                path: config_path,
            }) as Arc<dyn ConfigPathProvider>,
        );

        (repo, temp_dir)
    }

    #[test]
    fn should_include_all_workspaces() {
        let (sut, _temp_dir) = sut_factory(workspaces_factory(), tmux_factory());

        let result = sut.get_session_descriptions();

        let workspace_sessions: Vec<&SessionDescription> = result
            .iter()
            .filter(|x| matches!(x.kind, SessionKind::Workspace(..)))
            .collect();
        assert_eq!(workspace_sessions.len(), 2);
    }

    #[test]
    fn should_include_all_path_sessions() {
        let (sut, _temp_dir) = sut_factory(workspaces_factory(), tmux_factory());

        let result = sut.get_session_descriptions();

        let binaries_session = result
            .iter()
            .find(|x| !matches!(x.kind, SessionKind::Workspace(..)))
            .unwrap();
        assert_eq!(binaries_session.name, "User binaries");
    }

    #[test]
    fn should_use_session_definition_for_workspace_sessions() {
        let (sut, _temp_dir) = sut_factory(workspaces_factory(), tmux_factory());

        let result = sut.get_session_descriptions();

        let home_session = result.iter().find(|x| x.name == "Home").unwrap();
        assert_eq!(home_session.windows.len(), 1);
    }

    #[test]
    fn should_apply_default_windows_to_workspaces() {
        let (sut, _temp_dir) = sut_factory(workspaces_factory(), tmux_factory());

        let result = sut.get_session_descriptions();

        let source_session = result.iter().find(|x| x.name == "Source").unwrap();
        assert_eq!(source_session.windows.len(), 2);
    }

    #[test]
    fn should_not_apply_workspaces_twice_when_defined_in_sessions() {
        let (sut, _temp_dir) = sut_factory(workspaces_factory(), tmux_factory());

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
        ) -> crate::domain::tmux_workspaces::aggregates::tmux::session::TmuxSession {
            panic!()
        }

        fn kill_session(
            &self,
            _session: Option<
                &crate::domain::tmux_workspaces::aggregates::tmux::session::TmuxSession,
            >,
        ) {
        }

        fn get_environment(&self, _session_id: &str) -> String {
            "".to_string()
        }

        fn get_sessions(
            &self,
            _filter: Option<
                crate::infrastructure::tmux_workspaces::tmux::tmux_format::TmuxFilterNode,
            >,
            _include: crate::domain::tmux_workspaces::aggregates::tmux::session::SessionIncludeFields,
        ) -> Vec<crate::domain::tmux_workspaces::aggregates::tmux::session::TmuxSession> {
            vec![]
        }
    }
}
