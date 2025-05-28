use crate::{
    domain::tmux_workspaces::{
        aggregates::workspaces::workspace::{Workspace, WorkspaceTag},
        repositories::workspace::workspace_repository::WorkspaceRepository,
    },
    storage::{self, workspace::WorkspaceStorage},
};

pub struct ImplWorkspaceRepository<'a, TWorkspaceStorage: WorkspaceStorage> {
    pub workspace_storage: &'a TWorkspaceStorage,
}

impl<TWorkspaceStorage> WorkspaceRepository for ImplWorkspaceRepository<'_, TWorkspaceStorage>
where
    TWorkspaceStorage: WorkspaceStorage,
{
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
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository,
        storage::{
            test::mocks::MockWorkspaceStorage,
            workspace::{Workspace, WorkspaceStorage},
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
                },
                Workspace {
                    id: "workspace-2".to_string(),
                    root: "~/home".to_string(),
                    name: "Workspace 2".to_string(),
                    tags: Some(vec!["tag-1".to_string(), "tag-2".to_string()]),
                },
            ],
        }
    }

    #[test]
    fn should_map_all_workspaces() {
        let mut workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository {
            workspace_storage: &mut workspace_storage,
        };

        let result = sut.get_workspaces();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn should_map_root_to_path() {
        let mut workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository {
            workspace_storage: &mut workspace_storage,
        };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().path, "~");
    }

    #[test]
    fn should_map_basic_fields() {
        let mut workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository {
            workspace_storage: &mut workspace_storage,
        };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().name, "Workspace 1");
        assert_eq!(result.first().unwrap().id, "workspace-1");
    }

    #[test]
    fn should_map_none_tags_to_empty_vec() {
        let mut workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository {
            workspace_storage: &mut workspace_storage,
        };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().tags.len(), 0);
    }

    #[test]
    fn should_map_tags_to_workspace_tags() {
        let mut workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository {
            workspace_storage: &mut workspace_storage,
        };

        let result = sut.get_workspaces();
        let tag_result = &result.last().unwrap().tags;

        assert_eq!(tag_result.len(), 2);
        assert_eq!(tag_result.first().unwrap().name, "tag-1");
        assert_eq!(tag_result.last().unwrap().name, "tag-2");
    }
}
