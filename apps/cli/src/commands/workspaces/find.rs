use std::sync::Arc;

use shaku::{Component, Interface};

use crate::{
    storage::workspace::WorkspaceStorage,
    utils::{
        data_with_path::DataWithPath, display::RafaeltabDisplay, path::expand_path,
        workspace::find_workspace,
    },
};

pub trait FindWorkspaceCommandInterface: Interface {
    fn execute(&self, args: FindWorkspaceArgs);
}

pub struct FindWorkspaceArgs<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub id: String,
}

#[derive(Component)]
#[shaku(interface = FindWorkspaceCommandInterface)]
pub struct FindWorkspaceCommand {
    #[shaku(inject)]
    workspace_storage: Arc<dyn WorkspaceStorage>,
}

impl FindWorkspaceCommandInterface for FindWorkspaceCommand {
    fn execute(&self, args: FindWorkspaceArgs) {
        let workspace = find_workspace(&*self.workspace_storage, &args.id);
        if let Some(space) = workspace {
            args.display
                .display(&DataWithPath::new(space.clone(), expand_path(&space.root)))
        }
    }
}
