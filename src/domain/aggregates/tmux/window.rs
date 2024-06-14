use super::pane::TmuxPane;

#[derive(Debug, Clone)]
pub struct TmuxWindow {
    pub id: String,
    pub index: String,
    pub name: String,
    pub panes: Option<Vec<TmuxPane>>,
    pub include_fields: WindowIncludeFields,
}

#[derive(Debug, Clone)]
pub struct WindowIncludeFields {
    pub panes: Option<()>,
}
