use serde_json::{json, Value};

use crate::config::{Config, Workspace};

use super::{data_with_path::DataWithPath, path::expand_path};

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

impl DataWithPath<Workspace> {
    pub fn to_json(&self) -> Value {
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

pub trait ToDynVec<'a> {
    fn to_dyn_vec(&self) -> Vec<&dyn RafaeltabDisplayItem>;
}

impl<'a, T> ToDynVec<'a> for Vec<T>
where
    T: RafaeltabDisplayItem,
{
    fn to_dyn_vec(&self) -> Vec<&dyn RafaeltabDisplayItem> {
        self.iter()
            .map(|x| x as &dyn RafaeltabDisplayItem)
            .collect()
    }
}

pub trait RafaeltabDisplayItem {
    fn to_json(&self) -> Value;
    fn to_pretty_string(&self) -> String;
}

pub trait RafaeltabDisplay {
    fn display_list(&self, list: Vec<&dyn RafaeltabDisplayItem>);
    fn display(&self, element: &dyn RafaeltabDisplayItem);
}

pub struct PrettyDisplay;

impl RafaeltabDisplay for PrettyDisplay {
    fn display_list(&self, list: Vec<&dyn RafaeltabDisplayItem>) {
        for element in list {
            self.display(element);
        }
    }

    fn display(&self, element: &dyn RafaeltabDisplayItem) {
        println!("{}", element.to_pretty_string())
    }
}

pub struct JsonDisplay;

impl RafaeltabDisplay for JsonDisplay {
    fn display_list(&self, list: Vec<&dyn RafaeltabDisplayItem>) {
        let json_arr: Vec<Value> = list.into_iter().map(|x| x.to_json()).collect();
        let json_str = match serde_json::to_string(&json_arr) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert list to json"),
        };
        println!("{}", json_str);
    }

    fn display(&self, element: &dyn RafaeltabDisplayItem) {
        let json_str = match serde_json::to_string(&element.to_json()) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert element to json"),
        };
        println!("{}", json_str);
    }
}

pub struct JsonPrettyDisplay;

impl RafaeltabDisplay for JsonPrettyDisplay {
    fn display_list(&self, list: Vec<&dyn RafaeltabDisplayItem>) {
        let json_arr: Vec<Value> = list.into_iter().map(|x| x.to_json()).collect();
        let json_str = match serde_json::to_string_pretty(&json_arr) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert list to json"),
        };
        println!("{}", json_str);
    }

    fn display(&self, element: &dyn RafaeltabDisplayItem) {
        let json_str = match serde_json::to_string_pretty(&element.to_json()) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert element to json"),
        };
        println!("{}", json_str);
    }
}
