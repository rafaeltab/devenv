use crate::{
    domain::tmux_workspaces::aggregates::tmux::{
        client::{ClientIncludeFields, TmuxClient},
        pane::TmuxPane,
        session::TmuxSession,
        window::TmuxWindow,
    },
    infrastructure::tmux_workspaces::tmux::tmux_format::TmuxFilterNode,
};

#[allow(dead_code)]
pub enum SwitchClientTarget<'a> {
    Session(&'a TmuxSession),
    Window(&'a TmuxWindow),
    Pane(&'a TmuxPane),
}

pub trait TmuxClientRepository {
    fn get_clients(
        &self,
        filter: Option<TmuxFilterNode>,
        include: ClientIncludeFields,
    ) -> Vec<TmuxClient>;
    fn switch_client(&self, client: Option<&TmuxClient>, target: SwitchClientTarget);
}
