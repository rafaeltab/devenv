use std::sync::Arc;

use shaku::Component;

use crate::{
    domain::tmux_workspaces::{
        aggregates::workspaces::workspace::{Workspace, WorkspaceTag},
        repositories::workspace::workspace_repository::WorkspaceRepository,
    },
    storage::{self, workspace::WorkspaceStorage},
};

#[derive(Component)]
#[shaku(interface = WorkspaceRepository)]
pub struct ImplWorkspaceRepository {
    #[shaku(inject)]
    pub workspace_storage: Arc<dyn WorkspaceStorage>,
}

impl WorkspaceRepository for ImplWorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace> {
        self.workspace_storage
            .read()
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
        let workspace = storage::workspace::Workspace {
            id,
            name,
            tags: Some(tags),
            root,
            worktree: None,
        };

        let mut workspaces = self.workspace_storage.read().clone();
        workspaces.push(workspace.clone());
        self.workspace_storage.write(&workspaces).expect("");

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
        domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository,
        storage::{
            test::mocks::MockWorkspaceStorage,
            workspace::{Workspace, WorkspaceStorage},
            worktree::WorkspaceWorktreeConfig,
        },
    };

    use super::ImplWorkspaceRepository;

    fn storage_factory() -> impl WorkspaceStorage {
        MockWorkspaceStorage {
            data: vec![
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
            ],
        }
    }

    #[test]
    fn should_map_all_workspaces() {
        let workspace_storage: Arc<dyn WorkspaceStorage> = Arc::new(storage_factory());

        let sut = ImplWorkspaceRepository { workspace_storage };

        let result = sut.get_workspaces();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn should_map_root_to_path() {
        let workspace_storage: Arc<dyn WorkspaceStorage> = Arc::new(storage_factory());

        let sut = ImplWorkspaceRepository { workspace_storage };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().path, "~");
    }

    #[test]
    fn should_map_basic_fields() {
        let workspace_storage: Arc<dyn WorkspaceStorage> = Arc::new(storage_factory());

        let sut = ImplWorkspaceRepository { workspace_storage };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().name, "Workspace 1");
        assert_eq!(result.first().unwrap().id, "workspace-1");
    }

    #[test]
    fn should_map_none_tags_to_empty_vec() {
        let workspace_storage: Arc<dyn WorkspaceStorage> = Arc::new(storage_factory());

        let sut = ImplWorkspaceRepository { workspace_storage };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().tags.len(), 0);
    }

    #[test]
    fn should_map_tags_to_workspace_tags() {
        let workspace_storage: Arc<dyn WorkspaceStorage> = Arc::new(storage_factory());

        let sut = ImplWorkspaceRepository { workspace_storage };

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

        let workspace_storage: Arc<dyn WorkspaceStorage> = Arc::new(MockWorkspaceStorage {
            data: vec![Workspace {
                id: "workspace-with-config".to_string(),
                root: "~/test".to_string(),
                name: "Test Workspace".to_string(),
                tags: None,
                worktree: Some(worktree_config.clone()),
            }],
        });

        let sut = ImplWorkspaceRepository { workspace_storage };

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
        let workspace_storage: Arc<dyn WorkspaceStorage> = Arc::new(storage_factory());

        let sut = ImplWorkspaceRepository { workspace_storage };

        let result = sut.get_workspaces();

        assert!(result.first().unwrap().worktree.is_none());
        assert!(result.last().unwrap().worktree.is_none());
    }
}
