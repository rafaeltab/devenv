use crate::domain::command_palette::entities::{
    command::{Command, CommandDetails, CommandId, UsageStats},
    input_step::{BaseInputStep, InputStep, InputStepId},
    value_objects::bash_script::BashScriptTemplate,
};

pub struct CommandImpl {
    id: CommandId,
    usage_stats: UsageStats,
    details: CommandDetails,
    steps: Vec<InputStep>,
    template: BashScriptTemplate,
}

impl CommandImpl {
    pub fn new(
        id: CommandId,
        usage_stats: UsageStats,
        details: CommandDetails,
        steps: Vec<InputStep>,
        template: BashScriptTemplate,
    ) -> Self {
        let mut steps = steps;
        steps.sort_by_key(|a| a.get_order());

        CommandImpl {
            id,
            usage_stats,
            details,
            steps,
            template,
        }
    }
}

impl Command for CommandImpl {
    fn get_id(&self) -> &CommandId {
        &self.id
    }

    fn get_usage_stats(&self) -> &UsageStats {
        &self.usage_stats
    }

    fn get_details(&self) -> &CommandDetails {
        &self.details
    }

    fn update_usage_stats(&mut self, stats: UsageStats) {
        self.usage_stats = stats;
    }

    fn get_ordered_input_steps(&self) -> &Vec<InputStep> {
        &self.steps
    }

    fn get_input_step_by_id(&self, step_id: &InputStepId) -> Option<&InputStep> {
        self.steps.iter().find(|x| x.get_id() == step_id)
    }

    fn get_bash_script_template(&self) -> &BashScriptTemplate {
        &self.template
    }
}
