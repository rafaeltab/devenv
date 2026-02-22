use shaku::Interface;

use crate::{
    domain::tmux_workspaces::aggregates::tmux::{
        description::session::SessionDescription,
        session::{SessionIncludeFields, TmuxSession},
    },
    infrastructure::tmux_workspaces::tmux::tmux_format::TmuxFilterNode,
};

pub trait TmuxSessionRepository: Interface {
    fn new_session(&self, description: &SessionDescription) -> TmuxSession;
    fn kill_session(&self, session: Option<&TmuxSession>);
    fn get_environment(&self, session_id: &str) -> String;
    fn get_sessions(
        &self,
        filter: Option<TmuxFilterNode>,
        include: SessionIncludeFields,
    ) -> Vec<TmuxSession>;
}
