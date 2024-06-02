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

impl Workspace {
    pub fn to_json(&self) -> Value {
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
}
pub trait WorkspaceDisplay {
    fn display_list_with_path(&self, workspaces: Vec<DataWithPath<Workspace>>);
    fn display_list(&self, workspaces: Vec<Workspace>);
    fn display_with_path(&self, workspace: DataWithPath<Workspace>);
    fn display(&self, workspace: Workspace);
}

pub struct PrettyWorkspaceDisplay;

impl WorkspaceDisplay for PrettyWorkspaceDisplay {
    fn display_list_with_path(&self, workspaces: Vec<DataWithPath<Workspace>>) {
        for workspace in workspaces {
            self.display_with_path(workspace);
        }
    }

    fn display_list(&self, workspaces: Vec<Workspace>) {
        for workspace in workspaces {
            self.display(workspace);
        }
    }

    fn display_with_path(&self, workspace: DataWithPath<Workspace>) {
        match &workspace.data.tags {
            Some(tags) if !tags.is_empty() => {
                println!(
                    "{} ({}): {} {:?}",
                    workspace.data.name, workspace.data.id, workspace.path, tags
                )
            }
            _ => println!(
                "{} ({}): {}",
                workspace.data.name, workspace.data.id, workspace.path
            ),
        }
    }

    fn display(&self, workspace: Workspace) {
        match &workspace.tags {
            Some(tags) if !tags.is_empty() => {
                println!(
                    "{} ({}): {} {:?}",
                    workspace.name, workspace.id, workspace.root, tags
                )
            }
            _ => println!("{} ({}): {}", workspace.name, workspace.id, workspace.root),
        }
    }
}

pub struct JsonWorkspaceDisplay;

impl WorkspaceDisplay for JsonWorkspaceDisplay {
    fn display_list_with_path(&self, workspaces: Vec<DataWithPath<Workspace>>) {
        let json_arr: Vec<Value> = workspaces.into_iter().map(|x| x.to_json()).collect();
        let json_str = match serde_json::to_string(&json_arr) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert workspaces to json"),
        };
        println!("{}", json_str);
    }

    fn display_list(&self, workspaces: Vec<Workspace>) {
        let json_arr: Vec<Value> = workspaces.into_iter().map(|x| x.to_json()).collect();
        let json_str = match serde_json::to_string(&json_arr) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert workspaces to json"),
        };
        println!("{}", json_str);
    }

    fn display_with_path(&self, workspace: DataWithPath<Workspace>) {
        let json_str = match serde_json::to_string(&workspace.to_json()) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert workspace to json"),
        };
        println!("{}", json_str);
    }

    fn display(&self, workspace: Workspace) {
        let json_str = match serde_json::to_string(&workspace.to_json()) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert workspace to json"),
        };
        println!("{}", json_str);
    }
}

pub struct JsonPrettyWorkspaceDisplay;

impl WorkspaceDisplay for JsonPrettyWorkspaceDisplay {
    fn display_list_with_path(&self, workspaces: Vec<DataWithPath<Workspace>>) {
        let json_arr: Vec<Value> = workspaces.into_iter().map(|x| x.to_json()).collect();
        let json_str = match serde_json::to_string_pretty(&json_arr) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert workspaces to json"),
        };
        println!("{}", json_str);
    }

    fn display_list(&self, workspaces: Vec<Workspace>) {
        let json_arr: Vec<Value> = workspaces.into_iter().map(|x| x.to_json()).collect();
        let json_str = match serde_json::to_string_pretty(&json_arr) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert workspaces to json"),
        };
        println!("{}", json_str);
    }

    fn display_with_path(&self, workspace: DataWithPath<Workspace>) {
        let json_str = match serde_json::to_string_pretty(&workspace.to_json()) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert workspace to json"),
        };
        println!("{}", json_str);
    }

    fn display(&self, workspace: Workspace) {
        let json_str = match serde_json::to_string_pretty(&workspace.to_json()) {
            Ok(str) => str,
            Err(_) => panic!("Failed to convert workspace to json"),
        };
        println!("{}", json_str);
    }
}
