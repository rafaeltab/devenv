use std::sync::Arc;

use shaku::{Component, Interface};

use crate::{
    commands::tmux::session_utils::SessionUtilsService,
    domain::tmux_workspaces::repositories::tmux::{
        description_repository::SessionDescriptionRepository,
        session_repository::TmuxSessionRepository,
    },
};

pub trait TmuxStartCommandInterface: Interface {
    fn execute(&self);
}

#[derive(Component)]
#[shaku(interface = TmuxStartCommandInterface)]
pub struct TmuxStartCommand {
    #[shaku(inject)]
    session_description_repository: Arc<dyn SessionDescriptionRepository>,
    #[shaku(inject)]
    session_repository: Arc<dyn TmuxSessionRepository>,
    #[shaku(inject)]
    session_utils: Arc<dyn SessionUtilsService>,
}

impl TmuxStartCommandInterface for TmuxStartCommand {
    fn execute(&self) {
        let descriptions = self
            .session_description_repository
            .get_session_descriptions();

        for description in descriptions {
            if description.session.is_none() {
                let session = self.session_repository.new_session(&description);

                // Create worktree sessions if this is a workspace session
                use crate::domain::tmux_workspaces::aggregates::tmux::description::session::SessionKind;
                if let SessionKind::Workspace(workspace) = &description.kind {
                    self.session_utils.create_worktree_sessions(workspace);
                }

                // The created session is returned but we don't need to use it
                let _ = session;
            }
        }
    }
}
