use serde_json::{json, Value};

use crate::config::{Config, Workspace};

use super::{data_with_path::DataWithPath, display::RafaeltabDisplayItem, path::expand_path};

pub fn get_workspace_paths(config: Config) -> Vec<DataWithPath<Workspace>> {
    config
        .workspaces
        .into_iter()
        .map(|x| x.load_path())
        .collect()
}

pub fn find_workspace(config: &Config, id: &str) -> Option<Workspace> {
    config.workspaces.clone().into_iter().find(|x| x.id == id)
}

impl Workspace {
    pub fn load_path(&self) -> DataWithPath<Workspace> {
        DataWithPath::new(self.clone(), expand_path(&self.root))
    }
}

impl RafaeltabDisplayItem for Workspace {
    fn to_json(&self) -> Value {
        let tags: Vec<String> = match &self.tags {
            Some(tag_list) => tag_list.to_vec(),
            None => vec![],
        };

        json!({
            "name": self.name,
            "root": self.root,
            "id": self.id,
            "tags": tags,
        })
    }

    fn to_pretty_string(&self) -> String {
        match &self.tags {
            Some(tags) if !tags.is_empty() => {
                format!("{} ({}): {} {:?}", self.name, self.id, self.root, tags)
            }
            _ => format!("{} ({}): {}", self.name, self.id, self.root),
        }
    }
}

impl RafaeltabDisplayItem for DataWithPath<Workspace> {
    fn to_json(&self) -> Value {
        let tags: Vec<String> = match &self.data.tags {
            Some(tag_list) => tag_list.to_vec(),
            None => vec![],
        };

        json!({
            "name": self.data.name,
            "root": self.path,
            "id": self.data.id,
            "tags": tags,
        })
    }

    fn to_pretty_string(&self) -> String {
        match &self.data.tags {
            Some(tags) if !tags.is_empty() => {
                format!(
                    "{} ({}): {} {:?}",
                    self.data.name, self.data.id, self.path, tags
                )
            }
            _ => format!("{} ({}): {}", self.data.name, self.data.id, self.path),
        }
    }
}

