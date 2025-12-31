use serde_json::json;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use test_descriptors::descriptor::{CreateContext, CreateError, Descriptor};

// Shared state for collecting workspaces
thread_local! {
    static WORKSPACES: RefCell<Vec<WorkspaceData>> = RefCell::new(Vec::new());
}

#[derive(Debug, Clone)]
struct WorkspaceData {
    id: String,
    name: String,
    path: PathBuf,
    tags: Vec<String>,
    worktree: Option<WorktreeConfig>,
}

#[derive(Debug, Clone)]
struct WorktreeConfig {
    on_create: Vec<String>,
    symlink_files: Vec<String>,
}

pub struct WorkspaceDescriptor {
    id: String,
    name: String,
    path: PathBuf,
    tags: Vec<String>,
    worktree: Option<WorktreeConfig>,
}

impl WorkspaceDescriptor {
    pub fn new(id: &str, name: &str, path: PathBuf) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            path,
            tags: Vec::new(),
            worktree: None,
        }
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn with_worktree_config(
        mut self,
        on_create: Vec<String>,
        symlink_files: Vec<String>,
    ) -> Self {
        self.worktree = Some(WorktreeConfig {
            on_create,
            symlink_files,
        });
        self
    }
}

impl std::fmt::Debug for WorkspaceDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceDescriptor")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("path", &self.path)
            .finish()
    }
}

impl Descriptor for WorkspaceDescriptor {
    fn create(&self, _context: &CreateContext) -> Result<(), CreateError> {
        // Create the workspace directory
        fs::create_dir_all(&self.path)?;

        // Register this workspace for later use by ConfigDescriptor
        WORKSPACES.with(|workspaces| {
            workspaces.borrow_mut().push(WorkspaceData {
                id: self.id.clone(),
                name: self.name.clone(),
                path: self.path.clone(),
                tags: self.tags.clone(),
                worktree: self.worktree.clone(),
            });
        });

        Ok(())
    }
}

pub struct ConfigDescriptor;

impl ConfigDescriptor {
    pub fn new() -> Self {
        Self
    }
}

impl std::fmt::Debug for ConfigDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigDescriptor").finish()
    }
}

impl Descriptor for ConfigDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        // Collect all workspaces
        let workspaces_data = WORKSPACES.with(|workspaces| workspaces.borrow().clone());

        // Build workspace configurations
        let workspaces: Vec<serde_json::Value> = workspaces_data
            .iter()
            .map(|ws| {
                let mut workspace = json!({
                    "id": ws.id,
                    "name": ws.name,
                    "path": ws.path.to_string_lossy(),
                    "tags": ws.tags,
                });

                if let Some(worktree) = &ws.worktree {
                    workspace["worktree"] = json!({
                        "onCreate": worktree.on_create,
                        "symlinkFiles": worktree.symlink_files,
                    });
                }

                workspace
            })
            .collect();

        // Build the config
        let config = json!({
            "workspaces": workspaces,
            "tmuxSessions": [],
            "defaultWindows": [],
        });

        // Write config to file
        let config_path = context.root_path().join("config.json");
        let config_str = serde_json::to_string_pretty(&config).map_err(|e| {
            CreateError::InvalidDescriptor(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(&config_path, config_str)?;

        // Register config path in context
        context.set_config_path(config_path);

        Ok(())
    }
}
