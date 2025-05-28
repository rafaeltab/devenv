use crate::{
    commands::command::RafaeltabCommand,
    domain::tmux_workspaces::repositories::tmux::description_repository::SessionDescriptionRepository,
    utils::display::{RafaeltabDisplay, ToDynVec},
};

#[derive(Default)]
pub struct TmuxListCommand;

pub struct TmuxListOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
}

impl RafaeltabCommand<TmuxListOptions<'_>> for TmuxListCommand {
    fn execute(
        &self,
        TmuxListOptions {
            display,
            session_description_repository,
        }: TmuxListOptions,
    ) {
        let descriptions = session_description_repository.get_session_descriptions();

        display.display_list(descriptions.to_dyn_vec());
    }
}
