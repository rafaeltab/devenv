use shaku::Interface;

use crate::domain::tmux_workspaces::aggregates::tmux::description::session::SessionDescription;

pub trait SessionDescriptionRepository: Interface {
    fn get_session_descriptions(&self) -> Vec<SessionDescription>;
}
