use crate::{
    commands::command::RafaeltabCommand,
    config::Config,
    utils::{
        display::{RafaeltabDisplay, ToDynVec},
        workspace::get_workspace_paths,
    },
};

#[derive(Default)]
pub struct ListWorkspacesCommand;
pub struct ListWorkspacesCommandArgs<'a> {
    pub config: Config,
    pub display: &'a dyn RafaeltabDisplay,
}

impl<'a> RafaeltabCommand<ListWorkspacesCommandArgs<'a>> for ListWorkspacesCommand {
    fn execute(&self, ListWorkspacesCommandArgs { display, config }: ListWorkspacesCommandArgs) {
        display.display_list(get_workspace_paths(config).to_dyn_vec())
    }
}
