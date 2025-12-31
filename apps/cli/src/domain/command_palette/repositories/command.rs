use crate::domain::command_palette::{
    aggregates::command::CommandAggregate, entities::command::CommandId,
};

pub trait CommandRepository {
    fn get_by_id(&self, id: CommandId) -> impl CommandAggregate;
    fn save(&self, agg: impl CommandAggregate);
    fn delete(&self, id: CommandId);
}
