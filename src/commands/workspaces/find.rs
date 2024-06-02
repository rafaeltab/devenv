use crate::{
    config::Config,
    utils::{data_with_path::DataWithPath, path::expand_path, workspace::WorkspaceDisplay},
};

pub struct FindWorkspaceOptions<'a> {
    pub display: &'a dyn WorkspaceDisplay,
}

pub fn find_workspace(
    config: Config,
    id: &str,
    FindWorkspaceOptions { display }: FindWorkspaceOptions,
) {
    let workspace = config.workspaces.into_iter().find(|x| x.id == id);
    if let Some(space) = workspace {
        display.display_with_path(DataWithPath::new(space.clone(), expand_path(&space.root)))
    }
}
