use crate::{
    commands::command::RafaeltabCommand,
    storage::workspace::WorkspaceStorage,
    utils::{
        display::{RafaeltabDisplay, ToDynVec},
        workspace::get_workspace_paths,
    },
};

#[derive(Default)]
pub struct ListWorkspacesCommand;
pub struct ListWorkspacesCommandArgs<'a, TWorkspaceStorage: WorkspaceStorage> {
    pub workspace_storage: &'a TWorkspaceStorage,
    pub display: &'a dyn RafaeltabDisplay,
}

impl<'a, TWorkspaceStorage: WorkspaceStorage>
    RafaeltabCommand<ListWorkspacesCommandArgs<'a, TWorkspaceStorage>> for ListWorkspacesCommand
{
    fn execute(
        &self,
        ListWorkspacesCommandArgs {
            display,
            workspace_storage,
        }: ListWorkspacesCommandArgs<'a, TWorkspaceStorage>,
    ) {
        display.display_list(get_workspace_paths(workspace_storage).to_dyn_vec())
    }
}
