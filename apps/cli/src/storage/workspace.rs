use serde::{Deserialize, Serialize};

use super::storage_interface::Storage;

pub trait WorkspaceStorage: Storage<Vec<Workspace>> {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub root: String,
    pub id: String,
    pub name: String,
    pub tags: Option<Vec<String>>,
}
