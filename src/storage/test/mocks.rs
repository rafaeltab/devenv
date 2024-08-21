use crate::storage::{
    storage_interface::Storage,
    tmux::{Tmux, TmuxStorage},
    workspace::{Workspace, WorkspaceStorage},
};

pub struct MockWorkspaceStorage {
    pub data: Vec<Workspace>,
}

pub struct MockTmuxStorage {
    pub data: Tmux,
}

impl WorkspaceStorage for MockWorkspaceStorage {}
impl Storage<Vec<Workspace>> for MockWorkspaceStorage {
    fn read(&self) -> Vec<Workspace> {
        self.data.clone()
    }

    fn write(&self, _: &Vec<Workspace>) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl TmuxStorage for MockTmuxStorage {}
impl Storage<Tmux> for MockTmuxStorage {
    fn read(&self) -> Tmux {
        self.data.clone()
    }

    fn write(&self, _: &Tmux) -> Result<(), std::io::Error> {
        Ok(())
    }
}
