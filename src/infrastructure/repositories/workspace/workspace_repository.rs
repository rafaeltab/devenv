use crate::{
    config::Config,
    domain::{
        aggregates::workspaces::workspace::{Workspace, WorkspaceTag},
        repositories::workspace::workspace_repository::WorkspaceRepository,
    },
};

pub struct ImplWorkspaceRepository {
    pub config: Config,
}

impl WorkspaceRepository for ImplWorkspaceRepository {
    fn get_workspaces(&self) -> Vec<Workspace> {
        self.config
            .workspaces
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
        config::{Config, Tmux, Workspace},
        domain::repositories::workspace::workspace_repository::WorkspaceRepository,
    };

    use super::ImplWorkspaceRepository;

    fn config_factory() -> Config {
        Config {
            workspaces: vec![
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
            tmux: Tmux {
                sessions: None,
                default_windows: vec![],
                shell: "zsh -c".to_string(),
            },
        }
    }

    #[test]
    fn should_map_all_workspaces() {
        let config = config_factory();

        let sut = ImplWorkspaceRepository { config };

        let result = sut.get_workspaces();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn should_map_root_to_path() {
        let config = config_factory();

        let sut = ImplWorkspaceRepository { config };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().path, "~");
    }

    #[test]
    fn should_map_basic_fields() {
        let config = config_factory();

        let sut = ImplWorkspaceRepository { config };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().name, "Workspace 1");
        assert_eq!(result.first().unwrap().id, "workspace-1");
    }

    #[test]
    fn should_map_none_tags_to_empty_vec() {
        let config = config_factory();

        let sut = ImplWorkspaceRepository { config };

        let result = sut.get_workspaces();

        assert_eq!(result.first().unwrap().tags.len(), 0);
    }

    #[test]
    fn should_map_tags_to_workspace_tags() {
        let config = config_factory();

        let sut = ImplWorkspaceRepository { config };

        let result = sut.get_workspaces();
        let tag_result = &result.last().unwrap().tags;

        assert_eq!(tag_result.len(), 2);
        assert_eq!(tag_result.first().unwrap().name, "tag-1");
        assert_eq!(tag_result.last().unwrap().name, "tag-2");
    }
}
