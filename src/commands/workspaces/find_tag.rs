use crate::{
    config::{Config, Workspace},
    utils::{
        data_with_path::DataWithPath,
        display::{RafaeltabDisplay, ToDynVec},
    },
};

pub struct FindTagWorkspaceOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

pub fn find_tag_workspace(
    config: Config,
    tag: &str,
    FindTagWorkspaceOptions { display }: FindTagWorkspaceOptions,
) {
    let workspaces: Vec<DataWithPath<Workspace>> = config
        .workspaces
        .into_iter()
        .filter(|x| match &x.tags {
            Some(tags) => tags.contains(&tag.to_string()),
            None => false,
        })
        .map(|x| x.load_path())
        .collect();

    display.display_list(workspaces.to_dyn_vec());
}
