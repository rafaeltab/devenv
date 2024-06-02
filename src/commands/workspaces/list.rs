use crate::{
    config::Config,
    utils::workspace::{get_workspace_paths, RafaeltabDisplay, ToDynVec},
};

pub struct ListWorkspaceOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

pub fn list_workspaces(config: Config, ListWorkspaceOptions { display }: ListWorkspaceOptions) {
    display.display_list(get_workspace_paths(config).to_dyn_vec())
}
