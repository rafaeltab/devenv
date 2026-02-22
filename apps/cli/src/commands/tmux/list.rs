use std::sync::Arc;

use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::repositories::tmux::description_repository::SessionDescriptionRepository,
    utils::display::{DisplayFactory, ToDynVec},
};

/// Runtime options for TmuxListCommand - only contains CLI arguments
pub struct TmuxListRuntimeOptions {
    pub json: bool,
    pub json_pretty: bool,
}

/// Command for listing tmux sessions with dependency injection
pub struct TmuxListCommand {
    pub session_description_repository: Arc<dyn SessionDescriptionRepository>,
    pub display_factory: Arc<dyn DisplayFactory>,
}

impl RafaeltabCommand<TmuxListRuntimeOptions> for TmuxListCommand {
    fn execute(
        &self,
        options: TmuxListRuntimeOptions,
    ) -> Result<(), crate::commands::command::CommandError> {
        let display = self
            .display_factory
            .create_display(options.json, options.json_pretty);

        let descriptions = self
            .session_description_repository
            .get_session_descriptions();

        display.display_list(descriptions.to_dyn_vec());
        Ok(())
    }
}
