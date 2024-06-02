use core::panic;
use std::env;

use crate::{
    config::Config,
    utils::workspace::{get_workspace_paths, RafaeltabDisplay},
};

pub struct CurrentWorkspaceOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

pub fn get_current_workspace(
    config: Config,
    CurrentWorkspaceOptions { display }: CurrentWorkspaceOptions,
) {
    let workspaces = get_workspace_paths(config);
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
