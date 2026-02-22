use std::sync::Arc;

use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository,
    utils::{data_with_path::DataWithPath, display::DisplayFactory, path::expand_path},
};

// Runtime options - CLI arguments only
pub struct FindWorkspaceRuntimeOptions {
    pub id: String,
    pub json: bool,
    pub json_pretty: bool,
}

// Command with injected dependencies
pub struct FindWorkspaceCommand {
    pub workspace_repository: Arc<dyn WorkspaceRepository>,
    pub display_factory: Arc<dyn DisplayFactory>,
}

impl RafaeltabCommand<FindWorkspaceRuntimeOptions> for FindWorkspaceCommand {
    fn execute(
        &self,
        options: FindWorkspaceRuntimeOptions,
    ) -> Result<(), crate::commands::command::CommandError> {
        let display = self
            .display_factory
            .create_display(options.json, options.json_pretty);

        let workspace = self
            .workspace_repository
            .get_workspaces()
            .into_iter()
            .find(|x| x.id == options.id);

        if let Some(space) = workspace {
            let path = expand_path(&space.path);
            display.display(&DataWithPath::new(space, path))
        }

        Ok(())
    }
}
