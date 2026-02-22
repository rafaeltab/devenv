use std::sync::Arc;

use shaku::{Component, Interface};

use crate::{
    storage::workspace::WorkspaceStorage,
    utils::{
        display::{RafaeltabDisplay, ToDynVec},
        workspace::get_workspace_paths,
    },
};

pub trait ListWorkspacesCommandInterface: Interface {
    fn execute(&self, args: ListWorkspacesArgs);
}

pub struct ListWorkspacesArgs<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

#[derive(Component)]
#[shaku(interface = ListWorkspacesCommandInterface)]
pub struct ListWorkspacesCommand {
    #[shaku(inject)]
    workspace_storage: Arc<dyn WorkspaceStorage>,
}

impl ListWorkspacesCommandInterface for ListWorkspacesCommand {
    fn execute(&self, args: ListWorkspacesArgs) {
        args.display
            .display_list(get_workspace_paths(&*self.workspace_storage).to_dyn_vec())
    }
}
