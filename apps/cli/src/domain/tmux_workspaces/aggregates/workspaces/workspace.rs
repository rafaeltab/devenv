use serde::Serialize;
use serde_json::json;

use crate::{storage::worktree::WorkspaceWorktreeConfig, utils::display::RafaeltabDisplayItem};

#[derive(Clone, Serialize)]
pub struct Workspace {
    /// A plaintext string that represents the unique identifier of this workspace
    pub id: String,
    /// A name that describes this workspace
    pub name: String,
    /// The path where this workspace resides
    pub path: String,
    /// Tags that provide additional information about this workspace
    pub tags: Vec<WorkspaceTag>,
    /// How important this workspace is, a higher value means this workspace is more important
    pub importance: i32,
    /// Optional worktree configuration for this workspace
    pub worktree: Option<WorkspaceWorktreeConfig>,
}

#[derive(Clone, Serialize)]
pub struct WorkspaceTag {
    /// The name of this tag
    pub name: String,
}

impl RafaeltabDisplayItem for Workspace {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "name": self.name,
            "path": self.path,
            "tags": self.tags
        })
    }

    fn to_pretty_string(&self) -> String {
        format!(
            "Workspace {} @{} with id {} [{}] ",
            self.name,
            self.path,
            self.id,
            self.tags
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
