use super::window::{TmuxWindow, WindowIncludeFields};

#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub id: String,
    pub name: String,
    pub windows: Option<Vec<TmuxWindow>>,
    pub include_fields: SessionIncludeFields,
}

#[derive(Clone, Debug)]
pub struct SessionIncludeFields {
    pub windows: Option<WindowIncludeFields>
}
