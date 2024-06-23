use serde_json::json;

use crate::utils::display::RafaeltabDisplayItem;

use super::window::{TmuxWindow, WindowIncludeFields};

#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub id: String,
    pub name: String,
    pub path: String,
    pub windows: Option<Vec<TmuxWindow>>,
    pub environment: Option<String>,
    pub include_fields: SessionIncludeFields,
}

#[derive(Clone, Debug)]
pub struct SessionIncludeFields {
    pub windows: Option<WindowIncludeFields>,
    pub environment: Option<()>,
}

impl RafaeltabDisplayItem for TmuxSession {
    fn to_json(&self) -> serde_json::Value {
        let mut value = json!({
            "id": self.id,
            "name": self.name,
            "path": self.path,
        });
        if let Some(windows) = &self.windows {
            value["windows"] = windows.iter().map(|window| window.to_json()).collect();
        }
        value
    }

    fn to_pretty_string(&self) -> String {
        let TmuxSession { id, name, path, .. } = self;
        let windows_part = if let Some(windows) = &self.windows {
            format!(" and {} windows", windows.len())
        } else {
            "".to_string()
        };

        format!(
            "Session {} with id {} at {}{}",
            name, id, path, windows_part
        )
    }
}
