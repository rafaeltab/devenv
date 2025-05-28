use serde_json::{json, Value};

use crate::{
    domain::tmux_workspaces::aggregates::{tmux::session::TmuxSession, workspaces::workspace::Workspace},
    utils::display::RafaeltabDisplayItem,
};

use super::window::WindowDescription;

pub struct SessionDescription {
    pub id: String,
    pub name: String,
    pub kind: SessionKind,
    pub windows: Vec<WindowDescription>,
    pub session: Option<TmuxSession>,
}

pub enum SessionKind {
    Path(PathSessionDescription),
    Workspace(Workspace),
}

pub struct PathSessionDescription {
    pub path: String,
}

impl RafaeltabDisplayItem for SessionDescription {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "name": self.name,
            "windows": self.windows.iter().map(|x| json!({
                "name": x.name,
                "command": x.command.clone()
            })).collect::<Vec<Value>>(),
            "path": match &self.kind {
                SessionKind::Path(path) => path.path.clone(),
                SessionKind::Workspace(workspace) => workspace.path.clone(),
            },
            "session": self.session.clone().map(|session| session.to_json()),
        })
    }

    fn to_pretty_string(&self) -> String {
        let path_or_name = match &self.kind {
            SessionKind::Path(path) => &path.path,
            SessionKind::Workspace(workspace) => &workspace.name,
        };

        let session_text = match &self.session {
            Some(_) => "attached session",
            None => "no attached session",
        };

        format!(
            "session called '{}' @ '{}' with {}",
            self.name, path_or_name, session_text
        )
    }
}
