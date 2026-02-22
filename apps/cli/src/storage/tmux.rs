use serde::{Deserialize, Serialize};
use shaku::Interface;

use super::storage_interface::Storage;

pub trait TmuxStorage: Storage<Tmux> + Interface {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tmux {
    pub sessions: Option<Vec<Session>>,
    pub default_windows: Vec<Window>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Session {
    Workspace(WorkspaceSession),
    Path(PathSession),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSession {
    pub windows: Vec<Window>,
    pub workspace: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSession {
    pub windows: Vec<Window>,
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub name: String,
    pub command: Option<String>,
}
