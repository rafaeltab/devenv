use crate::{config::Workspace, domain::aggregates::tmux::session::TmuxSession};

use super::window::WindowDescription;

pub struct SessionDescription {
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
    pub description: String,
}
