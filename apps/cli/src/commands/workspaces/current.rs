use std::env;
use std::sync::Arc;

use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository,
    domain::worktree::config::find_most_specific_workspace,
    utils::{display::DisplayFactory, workspace::get_workspace_paths_from_repo},
};

pub struct CurrentWorkspaceRuntimeOptions {
    pub json: bool,
    pub json_pretty: bool,
}

pub struct CurrentWorkspaceCommand {
    pub workspace_repository: Arc<dyn WorkspaceRepository>,
    pub display_factory: Arc<dyn DisplayFactory>,
}

impl RafaeltabCommand<CurrentWorkspaceRuntimeOptions> for CurrentWorkspaceCommand {
    fn execute(
        &self,
        options: CurrentWorkspaceRuntimeOptions,
    ) -> Result<(), crate::commands::command::CommandError> {
        let display = self
            .display_factory
            .create_display(options.json, options.json_pretty);

        let workspaces = get_workspace_paths_from_repo(Arc::clone(&self.workspace_repository));
        let cwd = match env::current_dir() {
            Ok(cwd_path) => cwd_path.to_string_lossy().to_string(),
            Err(_) => panic!("Failed to read cwd"),
        };

        // Build iterator of (workspace_id, path) tuples
        let workspace_paths_iter = workspaces
            .iter()
            .map(|ws| (ws.data.id.as_str(), ws.path.as_str()));

        // Find the most specific (deepest nested) workspace
        if let Some(workspace_id) = find_most_specific_workspace(&cwd, workspace_paths_iter) {
            // Find the full workspace data and display it
            if let Some(workspace) = workspaces.iter().find(|ws| ws.data.id == workspace_id) {
                display.display(workspace);
            }
        }

        // If nothing is found we do empty output
        Ok(())
    }
}
