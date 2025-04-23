use crate::{
    commands::command::RafaeltabCommand,
    domain::repositories::tmux::description_repository::SessionDescriptionRepository,
    utils::display::{RafaeltabDisplay, ToDynVec},
};

#[derive(Default)]
pub struct TmuxListCommand;

pub struct TmuxListOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
}

impl<'a> RafaeltabCommand<TmuxListOptions<'a>> for TmuxListCommand {
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
