use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::repositories::tmux::{
        description_repository::SessionDescriptionRepository,
        session_repository::TmuxSessionRepository,
    },
    storage::tmux::TmuxStorage,
};

#[derive(Default)]
pub struct TmuxStartCommand;

pub struct TmuxStartOptions<'a> {
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
    pub session_repository: &'a dyn TmuxSessionRepository,
    pub tmux_storage: &'a dyn TmuxStorage,
}

impl RafaeltabCommand<TmuxStartOptions<'_>> for TmuxStartCommand {
    fn execute(
        &self,
        TmuxStartOptions {
            session_description_repository,
            session_repository,
            tmux_storage,
        }: TmuxStartOptions,
    ) {
        let descriptions = session_description_repository.get_session_descriptions();

        for description in descriptions {
            if description.session.is_none() {
                let session = session_repository.new_session(&description);

                // Create worktree sessions if this is a workspace session
                use crate::domain::tmux_workspaces::aggregates::tmux::description::session::SessionKind;
                if let SessionKind::Workspace(workspace) = &description.kind {
                    crate::commands::tmux::session_utils::create_worktree_sessions(
                        workspace,
                        session_repository,
                        tmux_storage,
                    );
                }

                // The created session is returned but we don't need to use it
                let _ = session;
            }
        }
    }
}
