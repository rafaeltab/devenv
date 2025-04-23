use crate::domain::aggregates::tmux::description::session::SessionDescription;

pub trait SessionDescriptionRepository {
    fn get_session_descriptions(&self) -> Vec<SessionDescription>;
}
