use crate::{
    config::{Config, Workspace},
    utils::{data_with_path::DataWithPath, workspace::WorkspaceDisplay},
};

pub struct FindTagWorkspaceOptions<'a> {
    pub display: &'a dyn WorkspaceDisplay,
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

    display.display_list_with_path(workspaces);
}
