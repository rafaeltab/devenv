use std::sync::Arc;

use shaku::{Component, Interface};

use crate::{
    storage::workspace::{Workspace, WorkspaceStorage},
    utils::{
        data_with_path::DataWithPath,
        display::{RafaeltabDisplay, ToDynVec},
    },
};

pub trait FindTagWorkspaceCommandInterface: Interface {
    fn execute(&self, args: FindTagWorkspaceArgs);
}

pub struct FindTagWorkspaceArgs<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub tag: String,
}

#[derive(Component)]
#[shaku(interface = FindTagWorkspaceCommandInterface)]
pub struct FindTagWorkspaceCommand {
    #[shaku(inject)]
    workspace_storage: Arc<dyn WorkspaceStorage>,
}

impl FindTagWorkspaceCommandInterface for FindTagWorkspaceCommand {
    fn execute(&self, args: FindTagWorkspaceArgs) {
        let workspaces: Vec<DataWithPath<Workspace>> = self
            .workspace_storage
            .read()
            .iter()
            .filter(|x| match &x.tags {
                Some(tags) => tags.contains(&args.tag.to_string()),
                None => false,
            })
            .map(|x| x.load_path())
            .collect();

        args.display.display_list(workspaces.to_dyn_vec());
    }
}
