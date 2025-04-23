use core::panic;
use std::env;

use crate::{
    storage::workspace::WorkspaceStorage, utils::{display::RafaeltabDisplay, workspace::get_workspace_paths}
};

pub struct CurrentWorkspaceOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

pub fn get_current_workspace< TWorkspaceStorage: WorkspaceStorage>(
    workspace_storage: &TWorkspaceStorage,
    CurrentWorkspaceOptions { display }: CurrentWorkspaceOptions,
) {
    let workspaces = get_workspace_paths(workspace_storage);
    let cwd = match env::current_dir() {
        Ok(cwd_path) => cwd_path.to_string_lossy().to_string(),
        Err(_) => panic!("Failed to read cwd"),
    };

    for workspace in workspaces {
        if path_matches(&workspace.path, &cwd) {
            display.display(&workspace);
            break;
        }
    }

    // If nothing is found we do empty output
}

fn path_matches(path: &str, cwd: &str) -> bool {
    cwd.starts_with(path)
}
