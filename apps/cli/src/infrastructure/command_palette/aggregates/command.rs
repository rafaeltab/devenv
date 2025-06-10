use crate::{
    domain::command_palette::{aggregates::command::CommandAggregate, entities::command::Command},
    infrastructure::command_palette::entities::command::CommandImpl,
};

pub struct CommandAggregateImpl {
    root: CommandImpl,
}

impl CommandAggregate for CommandAggregateImpl {
    fn get_root(&self) -> &dyn Command {
        &self.root
    }

    fn get_root_mut(&mut self) -> &mut dyn Command {
        &mut self.root
    }
}
