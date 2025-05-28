use crate::{
    domain::tmux_workspaces::aggregates::tmux::pane::TmuxPane,
    infrastructure::tmux_workspaces::tmux::tmux_format::TmuxFilterNode,
};

#[allow(dead_code)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[allow(dead_code)]
pub enum GetPanesTarget<'a> {
    None,
    Window { id: &'a str },
    Session { id: &'a str },
    All,
}

pub trait TmuxPaneRepository {
    fn get_panes(&self, filter: Option<TmuxFilterNode>, target: GetPanesTarget) -> Vec<TmuxPane>;
    fn kill_pane(&self, pane: Option<&TmuxPane>);
    fn split_window(
        &self,
        pane: Option<&TmuxPane>,
        cwd: Option<&str>,
        direction: SplitDirection,
    ) -> TmuxPane;
}
