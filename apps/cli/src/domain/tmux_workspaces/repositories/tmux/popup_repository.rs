/// Options for displaying a tmux popup
pub struct PopupOptions {
    /// Session to attach popup to
    pub target_session: String,
    /// Command to execute in popup
    pub command: String,
    /// Width of the popup (e.g., "80%")
    pub width: Option<String>,
    /// Height of the popup (e.g., "80%")
    pub height: Option<String>,
    /// Title for the popup
    pub title: Option<String>,
}

pub trait TmuxPopupRepository {
    /// Display a popup running the given command
    fn display_popup(&self, options: &PopupOptions) -> Result<(), String>;
}
