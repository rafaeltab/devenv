use serde_json::json;

use crate::utils::display::RafaeltabDisplayItem;

#[derive(Debug, Clone)]
pub struct TmuxPane {
    pub id: String,
    pub index: String,
    pub title: String,
    pub window_id: String,
}

impl RafaeltabDisplayItem for TmuxPane {
    fn to_json(&self) -> serde_json::Value {
        let TmuxPane {
            id,
            index,
            title,
            window_id,
        } = self;
        json!({
            "id": id,
            "index": index,
            "title": title,
            "window_id": window_id,
        })
    }

    fn to_pretty_string(&self) -> String {
        let TmuxPane {
            id,
            index,
            title,
            window_id,
        } = self;
        format!("{} {} {} {}", id, index, title, window_id)
    }
}
