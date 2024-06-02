use crate::{
    config::Config,
    utils::workspace::{get_workspace_paths, WorkspaceDisplay},
};

pub struct ListWorkspaceOptions<'a> {
    pub display: &'a dyn WorkspaceDisplay,
}

pub fn list_workspaces(config: Config, ListWorkspaceOptions { display }: ListWorkspaceOptions) {
    display.display_list_with_path(get_workspace_paths(config))
}
