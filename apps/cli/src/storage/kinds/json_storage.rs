use std::{fs, io, path::Path, sync::RwLock};

use serde::{Deserialize, Serialize};

use crate::{
    storage::{
        storage_interface::Storage,
        tmux::{Tmux, TmuxStorage},
        workspace::{Workspace, WorkspaceStorage},
        worktree::{WorktreeConfig, WorktreeStorage},
    },
    utils::path::expand_path,
};

/// Parameters for creating JsonStorage
#[derive(Debug, Clone)]
pub struct JsonStorageParameters {
    pub config_path: String,
}

/// JSON-based storage implementation that handles all data types
pub struct JsonStorage {
    path: String,
    data: RwLock<JsonData>,
}

impl JsonStorage {
    /// Get the config path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Create a new JsonStorage with the given parameters
    pub fn new(parameters: JsonStorageParameters) -> Result<Self, io::Error> {
        let json_data = load_json_data(parameters.config_path.clone())?;

        Ok(JsonStorage {
            path: parameters.config_path,
            data: RwLock::new(json_data),
        })
    }
}

impl WorkspaceStorage for JsonStorage {}
impl Storage<Vec<Workspace>> for JsonStorage {
    fn read(&self) -> Vec<Workspace> {
        self.data.read().unwrap().workspaces.clone()
    }

    fn write(&self, value: &Vec<Workspace>) -> Result<(), io::Error> {
        let new_value = JsonData {
            workspaces: value.clone(),
            tmux: self.data.read().unwrap().tmux.clone(),
            worktree: self.data.read().unwrap().worktree.clone(),
        };
        let _ = write_json_data(self.path.clone(), &new_value);
        // Reload data from disk
        let reloaded_data = load_json_data(self.path.clone())?;
        *self.data.write().unwrap() = reloaded_data;
        Ok(())
    }
}

impl TmuxStorage for JsonStorage {}
impl Storage<Tmux> for JsonStorage {
    fn read(&self) -> Tmux {
        self.data.read().unwrap().tmux.clone()
    }

    fn write(&self, value: &Tmux) -> Result<(), io::Error> {
        let new_value = JsonData {
            workspaces: self.data.read().unwrap().workspaces.clone(),
            tmux: value.clone(),
            worktree: self.data.read().unwrap().worktree.clone(),
        };
        let _ = write_json_data(self.path.clone(), &new_value);
        // Reload data from disk
        let reloaded_data = load_json_data(self.path.clone())?;
        *self.data.write().unwrap() = reloaded_data;
        Ok(())
    }
}

impl WorktreeStorage for JsonStorage {}
impl Storage<Option<WorktreeConfig>> for JsonStorage {
    fn read(&self) -> Option<WorktreeConfig> {
        self.data.read().unwrap().worktree.clone()
    }

    fn write(&self, value: &Option<WorktreeConfig>) -> Result<(), io::Error> {
        let new_value = JsonData {
            workspaces: self.data.read().unwrap().workspaces.clone(),
            tmux: self.data.read().unwrap().tmux.clone(),
            worktree: value.clone(),
        };
        let _ = write_json_data(self.path.clone(), &new_value);
        // Reload data from disk
        let reloaded_data = load_json_data(self.path.clone())?;
        *self.data.write().unwrap() = reloaded_data;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonData {
    pub workspaces: Vec<Workspace>,
    pub tmux: Tmux,
    /// Global worktree configuration (optional)
    pub worktree: Option<WorktreeConfig>,
}

fn load_json_data(path: String) -> Result<JsonData, io::Error> {
    let content = fs::read_to_string(path)?;
    let json_data: JsonData = serde_json::from_str(content.as_str())?;
    Ok(json_data)
}

fn write_json_data(path: String, data: &JsonData) -> Result<(), io::Error> {
    let string_data = serde_json::to_string_pretty(&data)?;
    fs::write(path, string_data)?;

    Ok(())
}

/// Legacy provider for creating storage (used during transition)
pub struct JsonStorageProvider {
    path: String,
}

impl JsonStorageProvider {
    pub fn new(path: Option<String>) -> Result<Self, io::Error> {
        let config_path = get_config_path(path)?;

        Ok(JsonStorageProvider { path: config_path })
    }

    pub fn load(&self) -> Result<JsonStorage, io::Error> {
        JsonStorage::new(JsonStorageParameters {
            config_path: self.path.clone(),
        })
    }
}

fn get_config_path(path: Option<String>) -> Result<String, io::Error> {
    static PATH_LOCATIONS_LINUX: &[&str] = &["~/.rafaeltab.json"];

    if let Some(path) = path {
        Ok(path)
    } else {
        // If config_path is not set, loop over PATH_LOCATIONS and find the first existing path
        for &path in PATH_LOCATIONS_LINUX {
            let full_path = expand_path(path);
            if Path::new(&full_path).exists() {
                return Ok(full_path);
            }
        }

        // If no existing path found, return an error
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No config file found in PATH_LOCATIONS",
        ))
    }
}
