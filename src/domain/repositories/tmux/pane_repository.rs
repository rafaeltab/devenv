use crate::{
    domain::aggregates::tmux::pane::TmuxPane,
    infrastructure::tmux::tmux_format::TmuxFilterNode,
};

#[allow(dead_code)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[allow(dead_code)]
pub enum ListPanesTarget<'a> {
    None,
    Window { id: &'a str },
    Session { id: &'a str },
    All,
}

pub trait TmuxPaneRepository {
    fn list_panes(&self, filter: Option<TmuxFilterNode>, target: ListPanesTarget) -> Vec<TmuxPane>;
    fn kill_pane(&self, pane: Option<&TmuxPane>);
    fn split_window(
        &self,
        pane: Option<&TmuxPane>,
        cwd: Option<&str>,
        direction: SplitDirection,
    ) -> TmuxPane;
}
