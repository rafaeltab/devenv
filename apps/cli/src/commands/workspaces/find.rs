use crate::{
    storage::workspace::WorkspaceStorage,
    utils::{
        data_with_path::DataWithPath, display::RafaeltabDisplay, path::expand_path,
        workspace::find_workspace,
    },
};

pub struct FindWorkspaceOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

pub fn find_workspace_cmd<TWorkspaceStorage: WorkspaceStorage>(
    workspace_storage: &TWorkspaceStorage,
    id: &str,
    FindWorkspaceOptions { display }: FindWorkspaceOptions,
) {
    let workspace = find_workspace(workspace_storage, id);
    if let Some(space) = workspace {
        display.display(&DataWithPath::new(space.clone(), expand_path(&space.root)))
    }
}
