use crate::storage::{
    storage_interface::Storage,
    tmux::{Tmux, TmuxStorage},
    workspace::{Workspace, WorkspaceStorage},
    worktree::{WorktreeConfig, WorktreeStorage},
};

pub struct MockWorkspaceStorage {
    pub data: Vec<Workspace>,
}

pub struct MockTmuxStorage {
    pub data: Tmux,
}

pub struct MockWorktreeStorage {
    pub data: Option<WorktreeConfig>,
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

impl WorktreeStorage for MockWorktreeStorage {}
impl Storage<Option<WorktreeConfig>> for MockWorktreeStorage {
    fn read(&self) -> Option<WorktreeConfig> {
        self.data.clone()
    }

    fn write(&self, _: &Option<WorktreeConfig>) -> Result<(), std::io::Error> {
        Ok(())
    }
}
