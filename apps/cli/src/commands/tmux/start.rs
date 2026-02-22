use std::sync::Arc;

use crate::{
    commands::command::RafaeltabCommand,
    di::ConfigPathProvider,
    domain::tmux_workspaces::repositories::tmux::{
        description_repository::SessionDescriptionRepository,
        session_repository::TmuxSessionRepository,
    },
};

// Runtime options - None for this command (no CLI arguments)
pub struct TmuxStartRuntimeOptions;

// Command with injected dependencies
pub struct TmuxStartCommand {
    pub session_description_repository: Arc<dyn SessionDescriptionRepository>,
    pub session_repository: Arc<dyn TmuxSessionRepository>,
    pub config_path_provider: Arc<dyn ConfigPathProvider>,
}

impl RafaeltabCommand<TmuxStartRuntimeOptions> for TmuxStartCommand {
    fn execute(
        &self,
        _options: TmuxStartRuntimeOptions,
    ) -> Result<(), crate::commands::command::CommandError> {
        let descriptions = self
            .session_description_repository
            .get_session_descriptions();
        let config_path = self.config_path_provider.path();

        for description in descriptions {
            if description.session.is_none() {
                let session = self.session_repository.new_session(&description);

                // Create worktree sessions if this is a workspace session
                use crate::domain::tmux_workspaces::aggregates::tmux::description::session::SessionKind;
                if let SessionKind::Workspace(workspace) = &description.kind {
                    crate::commands::tmux::session_utils::create_worktree_sessions(
                        workspace,
                        self.session_repository.as_ref(),
                        &config_path,
                    );
                }

                // The created session is returned but we don't need to use it
                let _ = session;
            }
        }
        Ok(())
    }
}
