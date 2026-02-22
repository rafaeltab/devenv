mod options;

use std::sync::Arc;

use shaku::{module, HasComponent};

pub use crate::infrastructure::tmux_workspaces::repositories::tmux::tmux_client::TmuxRepository;
pub use options::{ConfigPathOption, ConfigPathProvider, SocketNameOption, SocketNameProvider};

use crate::{
    commands::{
        tmux::{list::TmuxListCommand, start::TmuxStartCommand, switch::TmuxSwitchCommand},
        workspaces::{
            add::WorkspaceAddCommand, current::CurrentWorkspaceCommand, find::FindWorkspaceCommand,
            find_tag::FindTagWorkspaceCommand, list::ListWorkspacesCommand,
            tmux::ListTmuxWorkspacesCommand,
        },
        worktree::{complete::WorktreeCompleteCommand, start::WorktreeStartCommand},
    },
    domain::tmux_workspaces::repositories::{
        tmux::{
            client_repository::TmuxClientRepository,
            description_repository::SessionDescriptionRepository,
            popup_repository::TmuxPopupRepository, session_repository::TmuxSessionRepository,
        },
        workspace::workspace_repository::WorkspaceRepository,
    },
    infrastructure::tmux_workspaces::{
        repositories::{
            tmux::{
                description_repository::ImplDescriptionRepository,
                popup_repository::ImplPopupRepository,
            },
            workspace::workspace_repository::ImplWorkspaceRepository,
        },
        tmux::connection::TmuxConnectionImpl,
    },
    utils::display::DisplayFactoryImpl,
};

// Define the module with all components
module! {
    AppModule {
        components = [
            ConfigPathOption,
            SocketNameOption,
            TmuxConnectionImpl,
            TmuxRepository,
            ImplWorkspaceRepository,
            ImplDescriptionRepository,
            ImplPopupRepository,
            DisplayFactoryImpl,
        ],
        providers = []
    }
}

/// Container for dependency injection
pub struct AppContainer {
    config_path: String,
    module: Arc<AppModule>,
    /// Store the TmuxRepository separately so we can return it as different traits
    /// This avoids unsafe transmutation and ensures we use the same instance
    tmux_repository: Arc<TmuxRepository>,
    /// Command instances created with injected dependencies
    tmux_switch_command: Arc<TmuxSwitchCommand>,
    tmux_list_command: Arc<TmuxListCommand>,
    tmux_start_command: Arc<TmuxStartCommand>,
    list_workspaces_command: Arc<ListWorkspacesCommand>,
    workspace_add_command: Arc<WorkspaceAddCommand>,
    current_workspace_command: Arc<CurrentWorkspaceCommand>,
    find_workspace_command: Arc<FindWorkspaceCommand>,
    find_tag_workspace_command: Arc<FindTagWorkspaceCommand>,
    list_tmux_workspaces_command: Arc<ListTmuxWorkspacesCommand>,
    worktree_start_command: Arc<WorktreeStartCommand>,
    worktree_complete_command: Arc<WorktreeCompleteCommand>,
}

impl AppContainer {
    /// Create a new container with the given config path
    pub fn new(config_path: Option<String>) -> Result<Self, std::io::Error> {
        let resolved_path = resolve_config_path(config_path)?;

        let config_path_option = ConfigPathOption {
            path: resolved_path.clone(),
        };

        let module = AppModule::builder()
            .with_component_override::<dyn ConfigPathProvider>(Box::new(config_path_option))
            .build();

        // Resolve the TmuxRepository once and store it
        let tmux_repository: Arc<TmuxRepository> = Arc::new(TmuxRepository {
            connection: module.resolve(),
            config_path_provider: module.resolve(),
        });

        // Manually create command instances with injected dependencies
        let tmux_switch_command = Arc::new(TmuxSwitchCommand {
            session_description_repository: module.resolve(),
            session_repository: Arc::clone(&tmux_repository) as Arc<dyn TmuxSessionRepository>,
            client_repository: Arc::clone(&tmux_repository) as Arc<dyn TmuxClientRepository>,
            config_path_provider: module.resolve(),
        });

        let tmux_list_command = Arc::new(TmuxListCommand {
            session_description_repository: module.resolve(),
            display_factory: module.resolve(),
        });

        let tmux_start_command = Arc::new(TmuxStartCommand {
            session_description_repository: module.resolve(),
            session_repository: Arc::clone(&tmux_repository) as Arc<dyn TmuxSessionRepository>,
            config_path_provider: module.resolve(),
        });

        let list_workspaces_command = Arc::new(ListWorkspacesCommand {
            workspace_repository: module.resolve(),
            display_factory: module.resolve(),
        });

        let workspace_add_command = Arc::new(WorkspaceAddCommand {
            workspace_repository: module.resolve(),
            display_factory: module.resolve(),
        });

        let current_workspace_command = Arc::new(CurrentWorkspaceCommand {
            workspace_repository: module.resolve(),
            display_factory: module.resolve(),
        });

        let find_workspace_command = Arc::new(FindWorkspaceCommand {
            workspace_repository: module.resolve(),
            display_factory: module.resolve(),
        });

        let find_tag_workspace_command = Arc::new(FindTagWorkspaceCommand {
            workspace_repository: module.resolve(),
            display_factory: module.resolve(),
        });

        let list_tmux_workspaces_command = Arc::new(ListTmuxWorkspacesCommand {
            workspace_repository: module.resolve(),
            display_factory: module.resolve(),
        });

        let worktree_start_command = Arc::new(WorktreeStartCommand {
            workspace_repository: module.resolve(),
            session_repository: Arc::clone(&tmux_repository) as Arc<dyn TmuxSessionRepository>,
            client_repository: Arc::clone(&tmux_repository) as Arc<dyn TmuxClientRepository>,
            config_path_provider: module.resolve(),
        });

        let worktree_complete_command = Arc::new(WorktreeCompleteCommand {
            workspace_repository: module.resolve(),
            session_repository: Arc::clone(&tmux_repository) as Arc<dyn TmuxSessionRepository>,
            client_repository: Arc::clone(&tmux_repository) as Arc<dyn TmuxClientRepository>,
            popup_repository: module.resolve(),
            description_repository: module.resolve(),
        });

        Ok(Self {
            config_path: resolved_path,
            module: Arc::new(module),
            tmux_repository,
            tmux_switch_command,
            tmux_list_command,
            tmux_start_command,
            list_workspaces_command,
            workspace_add_command,
            current_workspace_command,
            find_workspace_command,
            find_tag_workspace_command,
            list_tmux_workspaces_command,
            worktree_start_command,
            worktree_complete_command,
        })
    }

    /// Get the config path
    pub fn config_path(&self) -> String {
        self.config_path.clone()
    }

    /// Get the workspace repository
    pub fn workspace_repository(&self) -> Arc<dyn WorkspaceRepository> {
        self.module.resolve()
    }

    /// Get the session repository (also implements TmuxClientRepository)
    pub fn session_repository(&self) -> Arc<dyn TmuxSessionRepository> {
        Arc::clone(&self.tmux_repository) as Arc<dyn TmuxSessionRepository>
    }

    /// Get the client repository
    /// Returns the same TmuxRepository instance as session_repository
    /// Since TmuxRepository implements both TmuxSessionRepository and TmuxClientRepository,
    /// we return the same Arc instance through different trait interfaces
    pub fn client_repository(&self) -> Arc<dyn TmuxClientRepository> {
        Arc::clone(&self.tmux_repository) as Arc<dyn TmuxClientRepository>
    }

    /// Get the description repository
    pub fn description_repository(&self) -> Arc<dyn SessionDescriptionRepository> {
        self.module.resolve()
    }

    /// Get the popup repository
    pub fn popup_repository(&self) -> Arc<dyn TmuxPopupRepository> {
        self.module.resolve()
    }

    /// Get the tmux switch command
    pub fn tmux_switch_command(&self) -> Arc<TmuxSwitchCommand> {
        Arc::clone(&self.tmux_switch_command)
    }

    /// Get the tmux list command
    pub fn tmux_list_command(&self) -> Arc<TmuxListCommand> {
        Arc::clone(&self.tmux_list_command)
    }

    /// Get the tmux start command
    pub fn tmux_start_command(&self) -> Arc<TmuxStartCommand> {
        Arc::clone(&self.tmux_start_command)
    }

    /// Get the list workspaces command
    pub fn list_workspaces_command(&self) -> Arc<ListWorkspacesCommand> {
        Arc::clone(&self.list_workspaces_command)
    }

    /// Get the workspace add command
    pub fn workspace_add_command(&self) -> Arc<WorkspaceAddCommand> {
        Arc::clone(&self.workspace_add_command)
    }

    /// Get the current workspace command
    pub fn current_workspace_command(&self) -> Arc<CurrentWorkspaceCommand> {
        Arc::clone(&self.current_workspace_command)
    }

    /// Get the find workspace command
    pub fn find_workspace_command(&self) -> Arc<FindWorkspaceCommand> {
        Arc::clone(&self.find_workspace_command)
    }

    /// Get the find tag workspace command
    pub fn find_tag_workspace_command(&self) -> Arc<FindTagWorkspaceCommand> {
        Arc::clone(&self.find_tag_workspace_command)
    }

    /// Get the list tmux workspaces command
    pub fn list_tmux_workspaces_command(&self) -> Arc<ListTmuxWorkspacesCommand> {
        Arc::clone(&self.list_tmux_workspaces_command)
    }

    /// Get the worktree start command
    pub fn worktree_start_command(&self) -> Arc<WorktreeStartCommand> {
        Arc::clone(&self.worktree_start_command)
    }

    /// Get the worktree complete command
    pub fn worktree_complete_command(&self) -> Arc<WorktreeCompleteCommand> {
        Arc::clone(&self.worktree_complete_command)
    }
}

fn resolve_config_path(path: Option<String>) -> Result<String, std::io::Error> {
    use crate::utils::path::expand_path;
    use std::path::Path;

    static DEFAULT_CONFIG_PATHS: &[&str] = &["~/.rafaeltab.json"];

    if let Some(path) = path {
        Ok(path)
    } else {
        // If config_path is not set, loop over DEFAULT_CONFIG_PATHS and find the first existing path
        for &path in DEFAULT_CONFIG_PATHS {
            let full_path = expand_path(path);
            if Path::new(&full_path).exists() {
                return Ok(full_path);
            }
        }

        // If no existing path found, return an error
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No config file found in default locations",
        ))
    }
}
