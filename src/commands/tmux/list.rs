use crate::{
    commands::command::RafaeltabCommand,
    domain::{
        aggregates::tmux::include_fields_builder::IncludeFieldsBuilder,
        repositories::tmux::session_repository::TmuxSessionRepository,
    },
    utils::display::{RafaeltabDisplay, ToDynVec},
};

#[derive(Default)]
pub struct TmuxListCommand;

pub struct TmuxListOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub session_repository: &'a dyn TmuxSessionRepository,
}

impl<'a> RafaeltabCommand<TmuxListOptions<'a>> for TmuxListCommand {
    fn execute(
        &self,
        TmuxListOptions {
            display,
            session_repository,
        }: TmuxListOptions,
    ) {
        // First get tmux sessions
        let sessions = session_repository.get_sessions(
            None,
            IncludeFieldsBuilder::default()
                .with_windows(true)
                .build_session(),
        );

        display.display_list(sessions.to_dyn_vec());

        // Then get the possible tmux sessions

        // Eliminate those that are already open

        // Print both together 
    }
}
