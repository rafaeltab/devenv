use crate::{
    domain::{
        aggregates::workspaces::workspace::{Workspace, WorkspaceTag},
        repositories::workspace::workspace_repository::WorkspaceRepository,
    },
    storage::workspace::WorkspaceStorage,
};

pub struct ImplWorkspaceRepository<'a, TWorkspaceStorage: WorkspaceStorage> {
    pub workspace_storage: &'a TWorkspaceStorage,
}

impl<'a, TWorkspaceStorage: WorkspaceStorage> WorkspaceRepository
    for ImplWorkspaceRepository<'a, TWorkspaceStorage>
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
}

#[cfg(test)]
mod test {
    use crate::{
        domain::repositories::workspace::workspace_repository::WorkspaceRepository,
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
        let workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository {
            workspace_storage: &workspace_storage,
        };

        let result = sut.get_workspaces();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn should_map_root_to_path() {
        let workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository { workspace_storage: &workspace_storage };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().path, "~");
    }

    #[test]
    fn should_map_basic_fields() {
        let workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository { workspace_storage: &workspace_storage };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().name, "Workspace 1");
        assert_eq!(result.first().unwrap().id, "workspace-1");
    }

    #[test]
    fn should_map_none_tags_to_empty_vec() {
        let workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository { workspace_storage: &workspace_storage };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().tags.len(), 0);
    }

    #[test]
    fn should_map_tags_to_workspace_tags() {
        let workspace_storage = storage_factory();

        let sut = ImplWorkspaceRepository { workspace_storage: &workspace_storage };

        let result = sut.get_workspaces();
        let tag_result = &result.last().unwrap().tags;

        assert_eq!(tag_result.len(), 2);
        assert_eq!(tag_result.first().unwrap().name, "tag-1");
        assert_eq!(tag_result.last().unwrap().name, "tag-2");
    }
}
