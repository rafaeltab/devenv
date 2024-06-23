use crate::{domain::aggregates::tmux::session::{SessionIncludeFields, TmuxSession}, infrastructure::tmux::tmux_format::TmuxFilterNode};

pub trait TmuxSessionRepository {
    fn new_session(&self) -> TmuxSession;
    fn kill_session(&self, session: Option<&TmuxSession>);
    fn get_environment(&self, session_id: &str) -> String;
    fn get_sessions(
        &self,
        filter: Option<TmuxFilterNode>,
        include: SessionIncludeFields,
    ) -> Vec<TmuxSession>;
}
