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

/// Tmux window configuration
#[derive(Debug, Clone)]
pub struct TmuxWindow {
    pub name: String,
    pub command: Option<String>,
}

/// Tmux session configuration
#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub workspace: Option<String>,
    pub path: Option<String>,
    pub name: Option<String>,
    pub windows: Vec<TmuxWindow>,
}

/// Builder for creating rafaeltab configuration files.
///
/// This builder collects all registered workspaces and generates a valid
/// rafaeltab config.json file. It should be added to the test environment
/// using the `RafaeltabRootMixin::rafaeltab_config()` method.
///
/// # Example
/// ```ignore
/// use test_descriptors::TestEnvironment;
///
/// let env = TestEnvironment::describe(|root| {
///     root.rafaeltab_config(|c| {
///         c.defaults();  // Add sensible defaults
///         c.default_window("editor");
///         c.tmux_session("workspace_id", None, &[("shell", None)]);
///     });
///     // Add workspaces...
/// }).create();
/// ```
pub struct ConfigBuilder {
    use_defaults: bool,
    worktree_global: Option<WorktreeGlobalConfig>,
    default_windows: Vec<(String, Option<String>)>,
    tmux_sessions: Vec<TmuxSession>,
}

impl ConfigBuilder {
    pub(crate) fn new() -> Self {
        Self {
            use_defaults: false,
            worktree_global: None,
            default_windows: Vec::new(),
            tmux_sessions: Vec::new(),
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

    /// Add multiple default windows at once
    ///
    /// # Arguments
    /// * `windows` - Array of tuples (name, optional_command)
    ///
    /// # Example
    /// ```ignore
    /// root.rafaeltab_config(|c| {
    ///     c.default_windows(&[
    ///         ("editor", Some("nvim .")),
    ///         ("shell", None),
    ///         ("build", Some("npm run dev")),
    ///     ]);
    /// });
    /// ```
    pub fn default_windows(&mut self, windows: &[(&str, Option<&str>)]) {
        for (name, cmd) in windows {
            self.default_windows
                .push((name.to_string(), cmd.map(|s| s.to_string())));
        }
    }

    /// Add a tmux session for a workspace
    ///
    /// # Arguments
    /// * `workspace_id` - The ID of the workspace this session is for
    /// * `session_name` - Optional custom name for the session (defaults to workspace name)
    /// * `windows` - List of window names (and optional commands)
    ///
    /// # Example
    /// ```ignore
    /// root.rafaeltab_config(|c| {
    ///     c.tmux_session("my_workspace", None, &[("shell", None), ("editor", Some("nvim ."))]);
    /// });
    /// ```
    pub fn tmux_session(
        &mut self,
        workspace_id: &str,
        session_name: Option<&str>,
        windows: &[(&str, Option<&str>)],
    ) {
        let windows = windows
            .iter()
            .map(|(name, cmd)| TmuxWindow {
                name: name.to_string(),
                command: cmd.map(|s| s.to_string()),
            })
            .collect();

        self.tmux_sessions.push(TmuxSession {
            workspace: Some(workspace_id.to_string()),
            path: None,
            name: session_name.map(|s| s.to_string()),
            windows,
        });
    }

    /// Add a tmux session for a specific path (not tied to a workspace)
    ///
    /// # Arguments
    /// * `path` - The directory path for this session
    /// * `session_name` - Name for the session
    /// * `windows` - List of window names (and optional commands)
    pub fn tmux_session_path(
        &mut self,
        path: &str,
        session_name: &str,
        windows: &[(&str, Option<&str>)],
    ) {
        let windows = windows
            .iter()
            .map(|(name, cmd)| TmuxWindow {
                name: name.to_string(),
                command: cmd.map(|s| s.to_string()),
            })
            .collect();

        self.tmux_sessions.push(TmuxSession {
            workspace: None,
            path: Some(path.to_string()),
            name: Some(session_name.to_string()),
            windows,
        });
    }

    pub(crate) fn build(self) -> ConfigDescriptor {
        ConfigDescriptor {
            use_defaults: self.use_defaults,
            worktree_global: self.worktree_global,
            default_windows: self.default_windows,
            tmux_sessions: self.tmux_sessions,
        }
    }
}

/// Descriptor for creating a rafaeltab config file
#[derive(Debug)]
pub struct ConfigDescriptor {
    use_defaults: bool,
    worktree_global: Option<WorktreeGlobalConfig>,
    default_windows: Vec<(String, Option<String>)>,
    tmux_sessions: Vec<TmuxSession>,
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

        // Always add tmux configuration (required by rafaeltab CLI)
        let default_windows: Vec<serde_json::Value> = if self.use_defaults {
            // Use sensible defaults
            vec![
                json!({ "name": "editor", "command": "nvim ." }),
                json!({ "name": "shell" }),
            ]
        } else if !self.default_windows.is_empty() {
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
        } else {
            // Empty default windows
            vec![]
        };

        // Build tmux sessions
        let sessions: Vec<serde_json::Value> = self
            .tmux_sessions
            .iter()
            .map(|session| {
                let mut s = json!({
                    "windows": session.windows.iter().map(|w| {
                        let mut win = json!({ "name": w.name });
                        if let Some(cmd) = &w.command {
                            win["command"] = json!(cmd);
                        }
                        win
                    }).collect::<Vec<_>>()
                });

                if let Some(workspace) = &session.workspace {
                    s["workspace"] = json!(workspace);
                }
                if let Some(path) = &session.path {
                    s["path"] = json!(path);
                }
                if let Some(name) = &session.name {
                    s["name"] = json!(name);
                }

                s
            })
            .collect();

        config["tmux"] = json!({
            "sessions": sessions,
            "defaultWindows": default_windows,
        });

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

/// Mixin trait for `RootBuilder` to add rafaeltab config creation capability.
///
/// This trait allows you to create a rafaeltab configuration file that will
/// include all workspaces registered via `RafaeltabDirMixin` or `RafaeltabGitMixin`.
///
/// The config file path will be stored in the test context and can be accessed
/// via `env.context().config_path()`.
///
/// # Example
/// ```ignore
/// use test_descriptors::TestEnvironment;
///
/// let env = TestEnvironment::describe(|root| {
///     // Create config with defaults
///     root.rafaeltab_config(|c| {
///         c.defaults();
///     });
///
///     // Add workspaces that will be included in the config
///     root.test_dir(|td| {
///         td.dir("workspace-1", |d| {
///             d.rafaeltab_workspace("ws1", "Workspace 1", |_| {});
///         });
///     });
/// }).create();
///
/// let config_path = env.context().config_path().unwrap();
/// // Use config_path with CLI
/// ```
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
