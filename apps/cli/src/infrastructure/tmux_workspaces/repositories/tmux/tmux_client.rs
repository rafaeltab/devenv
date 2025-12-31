use crate::infrastructure::tmux_workspaces::tmux::connection::TmuxConnection;
use crate::storage::tmux::TmuxStorage;

pub struct TmuxRepository<'a, TTmuxStorage: TmuxStorage> {
    pub tmux_storage: &'a TTmuxStorage,
    pub connection: &'a TmuxConnection,
}
