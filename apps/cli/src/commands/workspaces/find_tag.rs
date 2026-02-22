use std::sync::Arc;

use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::{
        aggregates::workspaces::workspace::Workspace,
        repositories::workspace::workspace_repository::WorkspaceRepository,
    },
    utils::{
        data_with_path::DataWithPath,
        display::{DisplayFactory, ToDynVec},
        path::expand_path,
    },
};

// Runtime options - CLI arguments only
pub struct FindTagWorkspaceRuntimeOptions {
    pub tag: String,
    pub json: bool,
    pub json_pretty: bool,
}

// Command with injected dependencies
pub struct FindTagWorkspaceCommand {
    pub workspace_repository: Arc<dyn WorkspaceRepository>,
    pub display_factory: Arc<dyn DisplayFactory>,
}

impl RafaeltabCommand<FindTagWorkspaceRuntimeOptions> for FindTagWorkspaceCommand {
    fn execute(
        &self,
        options: FindTagWorkspaceRuntimeOptions,
    ) -> Result<(), crate::commands::command::CommandError> {
        let display = self
            .display_factory
            .create_display(options.json, options.json_pretty);

        let workspaces: Vec<Workspace> = self
            .workspace_repository
            .get_workspaces()
            .into_iter()
            .filter(|x| x.tags.iter().any(|t| t.name == options.tag))
            .collect();

        let workspaces_with_path: Vec<DataWithPath<Workspace>> = workspaces
            .into_iter()
            .map(|ws| {
                let path = expand_path(&ws.path);
                DataWithPath::new(ws, path)
            })
            .collect();

        display.display_list(workspaces_with_path.to_dyn_vec());

        Ok(())
    }
}
