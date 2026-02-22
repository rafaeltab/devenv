use std::sync::Arc;

use shaku::{Component, Interface};

use crate::{
    domain::tmux_workspaces::repositories::tmux::description_repository::SessionDescriptionRepository,
    utils::display::{RafaeltabDisplay, ToDynVec},
};

pub trait TmuxListCommandInterface: Interface {
    fn execute(&self, args: TmuxListArgs);
}

pub struct TmuxListArgs<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

#[derive(Component)]
#[shaku(interface = TmuxListCommandInterface)]
pub struct TmuxListCommand {
    #[shaku(inject)]
    session_description_repository: Arc<dyn SessionDescriptionRepository>,
}

impl TmuxListCommandInterface for TmuxListCommand {
    fn execute(&self, args: TmuxListArgs) {
        let descriptions = self
            .session_description_repository
            .get_session_descriptions();
        args.display.display_list(descriptions.to_dyn_vec());
    }
}
