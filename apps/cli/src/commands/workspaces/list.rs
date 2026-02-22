use std::sync::Arc;

use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository,
    utils::{
        data_with_path::DataWithPath,
        display::{DisplayFactory, ToDynVec},
        workspace::get_workspace_paths_from_repo,
    },
};

/// Runtime options for ListWorkspacesCommand - only contains CLI arguments
pub struct ListWorkspacesRuntimeOptions {
    pub json: bool,
    pub json_pretty: bool,
}

/// Command for listing workspaces with dependency injection
pub struct ListWorkspacesCommand {
    pub workspace_repository: Arc<dyn WorkspaceRepository>,
    pub display_factory: Arc<dyn DisplayFactory>,
}

impl RafaeltabCommand<ListWorkspacesRuntimeOptions> for ListWorkspacesCommand {
    fn execute(
        &self,
        options: ListWorkspacesRuntimeOptions,
    ) -> Result<(), crate::commands::command::CommandError> {
        let display = self
            .display_factory
            .create_display(options.json, options.json_pretty);

        let workspaces = get_workspace_paths_from_repo(Arc::clone(&self.workspace_repository))
            .into_iter()
            .map(DataWithPath::from)
            .collect::<Vec<_>>();
        display.display_list(workspaces.to_dyn_vec());
        Ok(())
    }
}
