use crate::storage::tmux::TmuxStorage;

pub struct TmuxRepository<'a, TTmuxStorage: TmuxStorage> {
    pub tmux_storage: &'a TTmuxStorage,
}
