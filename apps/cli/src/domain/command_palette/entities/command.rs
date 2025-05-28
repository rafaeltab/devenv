use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{
    input_step::{BaseInputStep, InputStep, InputStepId},
    value_objects::bash_script::{
        BashScriptPlaceholderName, BashScriptResolveErrors, BashScriptTemplate,
    },
};

pub trait Command {
    /// Gets the unique identifier of this command.
    fn get_id(&self) -> &CommandId;

    /// Gets the usage statistics for this command.
    fn get_usage_stats(&self) -> &UsageStats;

    /// Gets the descriptive details of this command.
    fn get_details(&self) -> &CommandDetails;

    /// Change the usage statistics
    fn update_usage_stats(&mut self, stats: UsageStats);

    /// Gets an ordered list of input steps for this command.
    fn get_ordered_input_steps(&self) -> &Vec<InputStep>;

    /// Gets a specific input step by its ID.
    fn get_input_step_by_id(&self, step_id: &InputStepId) -> Option<&InputStep>;

    /// Get the bash script template
    fn get_bash_script_template(&self) -> &BashScriptTemplate;

    /// Prepares the final executable script string using the provided answers.
    fn prepare_executable_script(
        &self,
        answers: &HashMap<InputStepId, String>,
    ) -> Result<String, Vec<PrepareExecutableScriptError>> {
        let placeholder_values_result: Vec<(
            Result<BashScriptPlaceholderName, PrepareExecutableScriptError>,
            &String,
        )> = answers
            .iter()
            .map(|(step_id, answer)| {
                (
                    match self.get_input_step_by_id(step_id) {
                        Some(input_step) => Ok(input_step.get_script_placeholder_name()),
                        None => Err(PrepareExecutableScriptError::InputStepNotFoundForId(
                            step_id.clone(),
                        )),
                    },
                    answer,
                )
            })
            .collect();

        let mut placeholder_values: HashMap<BashScriptPlaceholderName, String> = HashMap::new();
        let mut errors: Vec<PrepareExecutableScriptError> = vec![];
        for placeholder in placeholder_values_result {
            match placeholder.0 {
                Err(error) => errors.push(error),
                Ok(placeholder_name) => {
                    placeholder_values.insert(placeholder_name, placeholder.1.clone());
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        self.get_bash_script_template()
            .resolve(placeholder_values)
            .map_err(|x| {
                x.into_iter()
                    .map(|y| match y {
                        BashScriptResolveErrors::PlaceholderValueNotFound(
                            bash_script_placeholder_name,
                        ) => PrepareExecutableScriptError::PlaceholderValueNotFound(
                            bash_script_placeholder_name,
                        ),
                    })
                    .collect()
            })
    }
}

#[derive(Debug, Clone)]
pub enum PrepareExecutableScriptError {
    PlaceholderValueNotFound(BashScriptPlaceholderName),
    InputStepNotFoundForId(InputStepId),
}

/// Unique identifier for a CommandAggregate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(pub Uuid);

/// A keyword associated with a command for searchability.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keyword(pub String);

/// Descriptive information for a Command.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandDetails {
    pub title: String,
    pub description: String,
    pub keywords: HashSet<Keyword>,
}

/// Statistics about command usage.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageStats {
    pub execution_count: u32,
    pub last_executed_at: Option<DateTime<Utc>>,
}

impl UsageStats {
    /// Creates a new UsageStats instance with an incremented execution count.
    pub fn record_execution(&self) -> Self {
        UsageStats {
            execution_count: self.execution_count + 1,
            last_executed_at: Some(Utc::now()),
        }
    }
}
