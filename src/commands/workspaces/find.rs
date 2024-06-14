use crate::{
    config::Config,
    utils::{
        data_with_path::DataWithPath, display::RafaeltabDisplay, path::expand_path,
        workspace::find_workspace,
    },
};

pub struct FindWorkspaceOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

pub fn find_workspace_cmd(
    config: Config,
    id: &str,
    FindWorkspaceOptions { display }: FindWorkspaceOptions,
) {
    let workspace = find_workspace(&config, id);
    if let Some(space) = workspace {
        display.display(&DataWithPath::new(space.clone(), expand_path(&space.root)))
    }
}
