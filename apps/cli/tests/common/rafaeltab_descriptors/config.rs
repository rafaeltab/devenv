use serde_json::json;
use std::fs;
use test_descriptors::descriptor::{CreateContext, CreateError, Descriptor};
use test_descriptors::RootBuilder;

use super::workspace::WORKSPACES;

/// Global worktree configuration
#[derive(Debug, Clone, Default)]
pub struct WorktreeGlobalConfig {
    pub on_create: Vec<String>,
    pub symlink_files: Vec<String>,
}

/// Configuration builder for creating rafaeltab config files
pub struct ConfigBuilder {
    use_defaults: bool,
    worktree_global: Option<WorktreeGlobalConfig>,
    default_windows: Vec<(String, Option<String>)>,
}

impl ConfigBuilder {
    pub(crate) fn new() -> Self {
        Self {
            use_defaults: false,
            worktree_global: None,
            default_windows: Vec::new(),
        }
    }

    /// Use sensible defaults for the config
    pub fn defaults(&mut self) {
        self.use_defaults = true;
    }

    /// Set global worktree configuration
    pub fn worktree_global(&mut self, on_create: &[&str], symlink_files: &[&str]) {
        self.worktree_global = Some(WorktreeGlobalConfig {
            on_create: on_create.iter().map(|s| s.to_string()).collect(),
            symlink_files: symlink_files.iter().map(|s| s.to_string()).collect(),
        });
    }

    /// Add a default window
    pub fn default_window(&mut self, name: &str) {
        self.default_windows.push((name.to_string(), None));
    }

    /// Add a default window with a command
    pub fn default_window_with_command(&mut self, name: &str, command: &str) {
        self.default_windows
            .push((name.to_string(), Some(command.to_string())));
    }

    pub(crate) fn build(self) -> ConfigDescriptor {
        ConfigDescriptor {
            use_defaults: self.use_defaults,
            worktree_global: self.worktree_global,
            default_windows: self.default_windows,
        }
    }
}

/// Descriptor for creating a rafaeltab config file
#[derive(Debug)]
pub struct ConfigDescriptor {
    use_defaults: bool,
    worktree_global: Option<WorktreeGlobalConfig>,
    default_windows: Vec<(String, Option<String>)>,
}

impl Descriptor for ConfigDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        // Collect all registered workspaces
        let workspaces_data = WORKSPACES.with(|workspaces| workspaces.borrow().clone());

        // Build workspace configurations
        let workspaces: Vec<serde_json::Value> = workspaces_data
            .iter()
            .map(|ws| {
                let mut workspace = json!({
                    "id": ws.id,
                    "name": ws.name,
                    "root": ws.path.to_string_lossy(),
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
        let mut config = json!({
            "workspaces": workspaces,
        });

        // Add tmux configuration if we have defaults or default windows
        if self.use_defaults || !self.default_windows.is_empty() {
            let default_windows: Vec<serde_json::Value> = if self.default_windows.is_empty() {
                // Use sensible defaults
                vec![
                    json!({ "name": "editor", "command": "nvim ." }),
                    json!({ "name": "shell" }),
                ]
            } else {
                self.default_windows
                    .iter()
                    .map(|(name, cmd)| {
                        let mut window = json!({ "name": name });
                        if let Some(command) = cmd {
                            window["command"] = json!(command);
                        }
                        window
                    })
                    .collect()
            };

            config["tmux"] = json!({
                "sessions": [],
                "defaultWindows": default_windows,
            });
        }

        // Add global worktree config if set
        if let Some(worktree) = &self.worktree_global {
            config["worktree"] = json!({
                "onCreate": worktree.on_create,
                "symlinkFiles": worktree.symlink_files,
            });
        }

        // Write config to file
        let config_path = context.root_path().join("config.json");
        let config_str = serde_json::to_string_pretty(&config).map_err(|e| {
            CreateError::InvalidDescriptor(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(&config_path, config_str)?;

        // Register config path in context
        context.set_config_path(config_path);

        // Clear workspaces for next test
        WORKSPACES.with(|workspaces| {
            workspaces.borrow_mut().clear();
        });

        Ok(())
    }
}

/// Mixin trait for RootBuilder to add rafaeltab_config support
pub trait RafaeltabRootMixin {
    fn rafaeltab_config<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ConfigBuilder);
}

impl RafaeltabRootMixin for RootBuilder<'_> {
    fn rafaeltab_config<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ConfigBuilder),
    {
        let mut builder = ConfigBuilder::new();
        f(&mut builder);
        // Add the config descriptor to the environment
        // It will be created after all other descriptors, reading from WORKSPACES
        self.add_descriptor(builder.build());
    }
}
