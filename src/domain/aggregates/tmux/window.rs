use serde_json::json;

use crate::utils::display::RafaeltabDisplayItem;

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

impl RafaeltabDisplayItem for TmuxWindow {
    fn to_json(&self) -> serde_json::Value {
        let TmuxWindow {
            id,
            index,
            name,
            panes: _,
            include_fields: _,
        } = self;
        let mut value = json!({
            "id": id,
            "index": index,
            "name": name,
        });

        if let Some(panes) = &self.panes {
            value["panes"] = panes.iter().map(|pane| pane.to_json()).collect();
        }
        value
    }

    fn to_pretty_string(&self) -> String {
        let TmuxWindow {
            id,
            index,
            name,
            panes: _,
            include_fields: _,
        } = self;
        if let Some(panes) = &self.panes {
            format!(
                "Window {} with id {} and index {} with {} panes",
                name,
                id,
                index,
                panes.len()
            )
        } else {
            format!("Window {} with id {} and index {}", name, id, index,)
        }
    }
}
