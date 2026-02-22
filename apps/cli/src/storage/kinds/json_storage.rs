use std::{
    fs, io,
    path::Path,
    sync::{Arc, RwLock},
};

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

pub struct JsonStorageProvider {
    path: String,
}
pub struct JsonStorage {
    path: String,
    data: RwLock<JsonData>,
}

impl JsonStorageProvider {
    pub fn new(path: Option<String>) -> Result<Self, io::Error> {
        let config_path = get_config_path(path)?;

        Ok(JsonStorageProvider { path: config_path })
    }

    pub fn load(&self) -> Result<JsonStorage, io::Error> {
        let json_data = load_json_data(self.path.clone())?;

        Ok(JsonStorage {
            path: self.path.clone(),
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
        *self.data.write().unwrap() = load_json_data(self.path.clone())?;
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
        *self.data.write().unwrap() = load_json_data(self.path.clone())?;
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
        *self.data.write().unwrap() = load_json_data(self.path.clone())?;
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

/// Wrapper to provide `JsonStorage` as `WorkspaceStorage` via shaku `with_component_override`.
/// Shares the underlying `JsonStorage` instance via `Arc`.
///
/// Listed as a component in the shaku module, but MUST be overridden via
/// `with_component_override` at module-build time (the default panics).
#[derive(shaku::Component)]
#[shaku(interface = WorkspaceStorage)]
pub struct JsonWorkspaceStorage {
    #[shaku(default)]
    inner: Option<Arc<JsonStorage>>,
}

impl JsonWorkspaceStorage {
    pub fn new(storage: Arc<JsonStorage>) -> Self {
        Self {
            inner: Some(storage),
        }
    }

    fn storage(&self) -> &JsonStorage {
        self.inner
            .as_ref()
            .expect("JsonWorkspaceStorage must be provided via with_component_override")
    }
}

impl Storage<Vec<Workspace>> for JsonWorkspaceStorage {
    fn read(&self) -> Vec<Workspace> {
        Storage::<Vec<Workspace>>::read(self.storage())
    }
    fn write(&self, value: &Vec<Workspace>) -> Result<(), io::Error> {
        Storage::<Vec<Workspace>>::write(self.storage(), value)
    }
}

impl WorkspaceStorage for JsonWorkspaceStorage {}

/// Wrapper to provide `JsonStorage` as `TmuxStorage` via shaku `with_component_override`.
#[derive(shaku::Component)]
#[shaku(interface = TmuxStorage)]
pub struct JsonTmuxStorage {
    #[shaku(default)]
    inner: Option<Arc<JsonStorage>>,
}

impl JsonTmuxStorage {
    pub fn new(storage: Arc<JsonStorage>) -> Self {
        Self {
            inner: Some(storage),
        }
    }

    fn storage(&self) -> &JsonStorage {
        self.inner
            .as_ref()
            .expect("JsonTmuxStorage must be provided via with_component_override")
    }
}

impl Storage<Tmux> for JsonTmuxStorage {
    fn read(&self) -> Tmux {
        Storage::<Tmux>::read(self.storage())
    }
    fn write(&self, value: &Tmux) -> Result<(), io::Error> {
        Storage::<Tmux>::write(self.storage(), value)
    }
}

impl TmuxStorage for JsonTmuxStorage {}

/// Wrapper to provide `JsonStorage` as `WorktreeStorage` via shaku `with_component_override`.
#[derive(shaku::Component)]
#[shaku(interface = WorktreeStorage)]
pub struct JsonWorktreeStorage {
    #[shaku(default)]
    inner: Option<Arc<JsonStorage>>,
}

impl JsonWorktreeStorage {
    pub fn new(storage: Arc<JsonStorage>) -> Self {
        Self {
            inner: Some(storage),
        }
    }

    fn storage(&self) -> &JsonStorage {
        self.inner
            .as_ref()
            .expect("JsonWorktreeStorage must be provided via with_component_override")
    }
}

impl Storage<Option<WorktreeConfig>> for JsonWorktreeStorage {
    fn read(&self) -> Option<WorktreeConfig> {
        Storage::<Option<WorktreeConfig>>::read(self.storage())
    }
    fn write(&self, value: &Option<WorktreeConfig>) -> Result<(), io::Error> {
        Storage::<Option<WorktreeConfig>>::write(self.storage(), value)
    }
}

impl WorktreeStorage for JsonWorktreeStorage {}

static PATH_LOCATIONS_LINUX: &[&str] = &["~/.rafaeltab.json"];

fn get_config_path(path: Option<String>) -> Result<String, io::Error> {
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
