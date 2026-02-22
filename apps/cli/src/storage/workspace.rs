use serde::{Deserialize, Serialize};
use shaku::Interface;

use super::{storage_interface::Storage, worktree::WorkspaceWorktreeConfig};

pub trait WorkspaceStorage: Storage<Vec<Workspace>> + Interface {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub root: String,
    pub id: String,
    pub name: String,
    pub tags: Option<Vec<String>>,
    /// Optional worktree configuration for this workspace
    pub worktree: Option<WorkspaceWorktreeConfig>,
}
