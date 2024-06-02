use std::{
    fs, io,
    path::Path,
};

use serde::Deserialize;

static PATH_LOCATIONS_LINUX: &[&str] = &["$HOME/.rafaeltab.json"];

pub fn load_config(config_path: Option<String>) -> Result<Config, io::Error> {
    let content = read_config(config_path)?;

    let v: Config = serde_json::from_str(content.as_str())?;
    Ok(v)
}

fn read_config(config_path: Option<String>) -> Result<String, io::Error> {
    if let Some(path) = config_path {
        // If config_path is set, read content from the specified file
        fs::read_to_string(path)
    } else {
        // If config_path is not set, loop over PATH_LOCATIONS and find the first existing path
        for &path in PATH_LOCATIONS_LINUX {
            if Path::new(path).exists() {
                return fs::read_to_string(path);
            }
        }
        // If no existing path found, return an error
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No config file found in PATH_LOCATIONS",
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub workspaces: Vec<Workspace>,
    pub tmux: Option<Tmux>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub root: String,
    pub id: String,
    pub name: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tmux {
    pub sessions: Vec<Session>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Session {
    Workspace(WorkspaceSession),
    Path(PathSession),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSession {
    pub windows: Vec<Window>,
    pub workspace: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSession {
    pub windows: Vec<Window>,
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub name: String,
    pub command: Option<String>,
}
