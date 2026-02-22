use std::sync::Arc;

use shaku::Component;

use crate::{
    di::ConfigPathProvider,
    domain::tmux_workspaces::{
        aggregates::workspaces::workspace::{Workspace, WorkspaceTag},
        repositories::workspace::workspace_repository::WorkspaceRepository,
    },
    storage::{self, kinds::json_storage::JsonStorage, storage_interface::Storage},
};

#[derive(Component)]
#[shaku(interface = WorkspaceRepository)]
pub struct ImplWorkspaceRepository {
    #[shaku(inject)]
    config_path_provider: Arc<dyn ConfigPathProvider>,
}

impl ImplWorkspaceRepository {
    fn get_storage(&self) -> JsonStorage {
        let config_path = self.config_path_provider.path().to_string();
        JsonStorage::new(crate::storage::kinds::json_storage::JsonStorageParameters { config_path })
            .expect("Failed to load storage")
    }

    /// Constructor for testing purposes
    #[cfg(test)]
    pub fn with_config_path(config_path_provider: Arc<dyn ConfigPathProvider>) -> Self {
        Self {
            config_path_provider,
        }
    }
}

impl WorkspaceRepository for ImplWorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace> {
        let storage = self.get_storage();
        let workspaces_data: Vec<crate::storage::workspace::Workspace> = storage.read();
        workspaces_data
            .iter()
            .map(|workspace| Workspace {
                id: workspace.id.clone(),
                tags: workspace
                    .tags
                    .clone()
                    .map(|x| {
                        x.iter()
                            .map(|tag| WorkspaceTag {
                                name: tag.to_string(),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                name: workspace.name.clone(),
                path: workspace.root.clone(),
                importance: 0,
                worktree: workspace.worktree.clone(),
            })
            .collect()
    }

    fn create_workspace(
        &self,
        name: String,
        tags: Vec<String>,
        root: String,
        id: String,
    ) -> Workspace {
        let storage = self.get_storage();
        let workspace = storage::workspace::Workspace {
            id,
            name,
            tags: Some(tags),
            root,
            worktree: None,
        };

        let mut workspaces: Vec<crate::storage::workspace::Workspace> = storage.read();
        workspaces.push(workspace.clone());
        storage.write(&workspaces).expect("");

        Workspace {
            id: workspace.id.clone(),
            tags: workspace
                .tags
                .clone()
                .map(|x| {
                    x.iter()
                        .map(|tag| WorkspaceTag {
                            name: tag.to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            name: workspace.name.clone(),
            path: workspace.root.clone(),
            importance: 0,
            worktree: workspace.worktree.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{
        di::{ConfigPathOption, ConfigPathProvider},
        domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository,
        storage::{workspace::Workspace, worktree::WorkspaceWorktreeConfig},
    };

    use super::ImplWorkspaceRepository;

    fn create_test_repo(
        workspaces: Vec<Workspace>,
    ) -> (ImplWorkspaceRepository, tempfile::TempDir) {
        use crate::storage::kinds::json_storage::JsonData;
        use std::io::Write;

        // Create a temp directory with the workspace data
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("rafaeltab.json");

        // Create full config structure
        let config = JsonData {
            workspaces: workspaces.clone(),
            tmux: crate::storage::tmux::Tmux {
                sessions: None,
                default_windows: vec![],
            },
            worktree: None,
        };

        let config_str = serde_json::to_string(&config).expect("Failed to serialize config");
        let mut file = std::fs::File::create(&config_path).expect("Failed to create config file");
        file.write_all(config_str.as_bytes())
            .expect("Failed to write config");

        let config_path_option = Arc::new(ConfigPathOption {
            path: config_path.to_string_lossy().to_string(),
        }) as Arc<dyn ConfigPathProvider>;

        (
            ImplWorkspaceRepository::with_config_path(config_path_option),
            temp_dir,
        )
    }

    fn workspaces_factory() -> Vec<Workspace> {
        vec![
            Workspace {
                id: "workspace-1".to_string(),
                root: "~".to_string(),
                name: "Workspace 1".to_string(),
                tags: None,
                worktree: None,
            },
            Workspace {
                id: "workspace-2".to_string(),
                root: "~/home".to_string(),
                name: "Workspace 2".to_string(),
                tags: Some(vec!["tag-1".to_string(), "tag-2".to_string()]),
                worktree: None,
            },
        ]
    }

    #[test]
    fn should_map_all_workspaces() {
        let (sut, _temp_dir) = create_test_repo(workspaces_factory());

        let result = sut.get_workspaces();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn should_map_root_to_path() {
        let (sut, _temp_dir) = create_test_repo(workspaces_factory());

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().path, "~");
    }

    #[test]
    fn should_map_basic_fields() {
        let (sut, _temp_dir) = create_test_repo(workspaces_factory());

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().name, "Workspace 1");
        assert_eq!(result.first().unwrap().id, "workspace-1");
    }

    #[test]
    fn should_map_none_tags_to_empty_vec() {
        let (sut, _temp_dir) = create_test_repo(workspaces_factory());

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().tags.len(), 0);
    }

    #[test]
    fn should_map_tags_to_workspace_tags() {
        let (sut, _temp_dir) = create_test_repo(workspaces_factory());

        let result = sut.get_workspaces();
        let tag_result = &result.last().unwrap().tags;

        assert_eq!(tag_result.len(), 2);
        assert_eq!(tag_result.first().unwrap().name, "tag-1");
        assert_eq!(tag_result.last().unwrap().name, "tag-2");
    }

    #[test]
    fn should_map_worktree_config_when_present() {
        let worktree_config = WorkspaceWorktreeConfig {
            symlink_files: vec![".env".to_string(), "config.json".to_string()],
            on_create: vec!["npm install".to_string()],
        };

        let workspaces = vec![Workspace {
            id: "workspace-with-config".to_string(),
            root: "~/test".to_string(),
            name: "Test Workspace".to_string(),
            tags: None,
            worktree: Some(worktree_config.clone()),
        }];

        let (sut, _temp_dir) = create_test_repo(workspaces);

        let result = sut.get_workspaces();
        let workspace = result.first().unwrap();

        assert!(workspace.worktree.is_some());
        let config = workspace.worktree.as_ref().unwrap();
        assert_eq!(config.symlink_files.len(), 2);
        assert_eq!(config.symlink_files[0], ".env");
        assert_eq!(config.symlink_files[1], "config.json");
        assert_eq!(config.on_create.len(), 1);
        assert_eq!(config.on_create[0], "npm install");
    }

    #[test]
    fn should_map_none_worktree_to_none() {
        let (sut, _temp_dir) = create_test_repo(workspaces_factory());

        let result = sut.get_workspaces();

        assert!(result.first().unwrap().worktree.is_none());
        assert!(result.last().unwrap().worktree.is_none());
    }
}
