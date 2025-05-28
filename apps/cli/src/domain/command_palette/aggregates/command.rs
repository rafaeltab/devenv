use crate::domain::command_palette::entities::command::Command;

pub trait CommandAggregate {
    /// Records an execution of this command, updating usage statistics.
    fn record_execution(&mut self) {
        let new_stats = self.get_root().get_usage_stats().record_execution();
        self.get_root_mut().update_usage_stats(new_stats);
    }

    /// Get the root of the aggregate
    fn get_root(&self) -> &dyn Command;

    /// Get the root of the aggregate
    fn get_root_mut(&mut self) -> &mut dyn Command;
}
