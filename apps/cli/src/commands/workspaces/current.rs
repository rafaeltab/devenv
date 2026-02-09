use std::env;

use crate::{
    domain::worktree::config::find_most_specific_workspace,
    storage::workspace::WorkspaceStorage,
    utils::{display::RafaeltabDisplay, workspace::get_workspace_paths},
};

pub struct CurrentWorkspaceOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

pub fn get_current_workspace<TWorkspaceStorage: WorkspaceStorage>(
    workspace_storage: &TWorkspaceStorage,
    CurrentWorkspaceOptions { display }: CurrentWorkspaceOptions,
) {
    let workspaces = get_workspace_paths(workspace_storage);
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
}
