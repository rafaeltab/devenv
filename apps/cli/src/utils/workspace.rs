use std::sync::Arc;

use serde_json::{json, Value};

use crate::domain::tmux_workspaces::{
    aggregates::workspaces::workspace::Workspace as DomainWorkspace,
    repositories::workspace::workspace_repository::WorkspaceRepository,
};
use crate::storage::workspace::{Workspace, WorkspaceStorage};

use super::{data_with_path::DataWithPath, display::RafaeltabDisplayItem, path::expand_path};

pub fn get_workspace_paths<TWorkspaceStorage: WorkspaceStorage>(
    workspace_storage: &TWorkspaceStorage,
) -> Vec<DataWithPath<Workspace>> {
    workspace_storage
        .read()
        .iter()
        .map(|x| x.load_path())
        .collect()
}

pub fn get_workspace_paths_from_repo(
    workspace_repository: Arc<dyn WorkspaceRepository>,
) -> Vec<DataWithPath<DomainWorkspace>> {
    workspace_repository
        .get_workspaces()
        .into_iter()
        .map(|ws| {
            let path = expand_path(&ws.path);
            DataWithPath::new(ws, path)
        })
        .collect()
}

pub fn find_workspace<TWorkspaceStorage: WorkspaceStorage>(
    workspace_storage: &TWorkspaceStorage,
    id: &str,
) -> Option<Workspace> {
    workspace_storage
        .read()
        .clone()
        .into_iter()
        .find(|x| x.id == id)
}

impl Workspace {
    pub fn load_path(&self) -> DataWithPath<Workspace> {
        DataWithPath::new(self.clone(), expand_path(&self.root))
    }
}

impl RafaeltabDisplayItem for Workspace {
    fn to_json(&self) -> Value {
        let tags: Vec<String> = match &self.tags {
            Some(tag_list) => tag_list.to_vec(),
            None => vec![],
        };

        json!({
            "name": self.name,
            "root": self.root,
            "id": self.id,
            "tags": tags,
        })
    }

    fn to_pretty_string(&self) -> String {
        match &self.tags {
            Some(tags) if !tags.is_empty() => {
                let tags_formatted = format!(
                    "[{}]",
                    tags.iter()
                        .map(|t| format!("\"{}\"", t))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                format!(
                    "{} ({}): {} {}",
                    self.name, self.id, self.root, tags_formatted
                )
            }
            _ => format!("{} ({}): {}", self.name, self.id, self.root),
        }
    }
}

impl RafaeltabDisplayItem for DataWithPath<Workspace> {
    fn to_json(&self) -> Value {
        let tags: Vec<String> = match &self.data.tags {
            Some(tag_list) => tag_list.to_vec(),
            None => vec![],
        };

        json!({
            "name": self.data.name,
            "root": self.path,
            "id": self.data.id,
            "tags": tags,
        })
    }

    fn to_pretty_string(&self) -> String {
        match &self.data.tags {
            Some(tags) if !tags.is_empty() => {
                let tags_formatted = format!(
                    "[{}]",
                    tags.iter()
                        .map(|t| format!("\"{}\"", t))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                format!(
                    "{} ({}): {} {}",
                    self.data.name, self.data.id, self.path, tags_formatted
                )
            }
            _ => format!("{} ({}): {}", self.data.name, self.data.id, self.path),
        }
    }
}

// Implementations for domain Workspace type
impl RafaeltabDisplayItem for DataWithPath<DomainWorkspace> {
    fn to_json(&self) -> Value {
        let tags: Vec<String> = self.data.tags.iter().map(|t| t.name.clone()).collect();
        json!({
            "name": self.data.name,
            "root": self.path,
            "id": self.data.id,
            "tags": tags,
            "importance": self.data.importance,
        })
    }

    fn to_pretty_string(&self) -> String {
        let tags_str = if self.data.tags.is_empty() {
            String::new()
        } else {
            format!(
                " [{}]",
                self.data
                    .tags
                    .iter()
                    .map(|t| format!("\"{}\"", t.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            "{} ({}): {}{}",
            self.data.name, self.data.id, self.path, tags_str
        )
    }
}
