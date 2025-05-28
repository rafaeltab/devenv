use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::repositories::tmux::{
        description_repository::SessionDescriptionRepository,
        session_repository::TmuxSessionRepository,
    },
};

#[derive(Default)]
pub struct TmuxStartCommand;

pub struct TmuxStartOptions<'a> {
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
    pub session_repository: &'a dyn TmuxSessionRepository,
}

impl RafaeltabCommand<TmuxStartOptions<'_>> for TmuxStartCommand {
    fn execute(
        &self,
        TmuxStartOptions {
            session_description_repository,
            session_repository,
        }: TmuxStartOptions,
    ) {
        let descriptions = session_description_repository.get_session_descriptions();

        for description in descriptions {
            if description.session.is_none() {
                session_repository.new_session(&description);

            }
        }
    }
}
