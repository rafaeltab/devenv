use uuid::Uuid;

use super::value_objects::bash_script::BashScriptPlaceholderName;

/// Unique identifier for an InputStep.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputStepId(pub Uuid);

/// Descriptive information for an InputStep.
#[derive(Debug, Clone, PartialEq)]
pub struct InputStepDetails {
    pub title: String,
    pub description: String,
}

/// An option for a SelectInputStep.
#[derive(Debug, Clone, PartialEq)]
pub struct InputOption {
    pub value: String,
    pub display_label: String,
}

/// Defines a validation rule for a TextInputStep.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationRule {
    pub script_body: String,
    pub language: String,
}

/// Outcome of a validation attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationOutcome {
    pub is_valid: bool,
    pub feedback_message: Option<String>,
}

/// Represents the base properties of a step in a command that requires user input.
pub trait BaseInputStep: Send + Sync {
    fn get_id(&self) -> &InputStepId;
    fn get_script_placeholder_name(&self) -> &BashScriptPlaceholderName;
    fn get_details(&self) -> &InputStepDetails;
    /// Gets the order of this input step within the command.
    fn get_order(&self) -> u32;
}

/// Enum representing the different types of input steps.
pub enum InputStep {
    /// An InputStep that allows selection from a predefined list of options.
    Select(Box<dyn SelectInputStep>),
    /// An InputStep that accepts free-form text input, with optional validation.
    Text(Box<dyn TextInputStep>),
}

pub trait TextInputStep: BaseInputStep {
    /// Validates a given answer for this input step.
    fn validate_answer(&self, answer: String) -> ValidationOutcome;
}

pub trait SelectInputStep: BaseInputStep {
    fn get_options(&self) -> &Vec<InputOption>;
}

impl BaseInputStep for InputStep {
    fn get_id(&self) -> &InputStepId {
        match &self {
            InputStep::Select(select_input_step) => select_input_step.get_id(),
            InputStep::Text(text_input_step) => text_input_step.get_id(),
        }
    }

    fn get_details(&self) -> &InputStepDetails {
        match &self {
            InputStep::Select(select_input_step) => select_input_step.get_details(),
            InputStep::Text(text_input_step) => text_input_step.get_details(),
        }
    }

    fn get_order(&self) -> u32 {
        match &self {
            InputStep::Select(select_input_step) => select_input_step.get_order(),
            InputStep::Text(text_input_step) => text_input_step.get_order(),
        }
    }

    fn get_script_placeholder_name(&self) -> &BashScriptPlaceholderName {
        match &self {
            InputStep::Select(select_input_step) => select_input_step.get_script_placeholder_name(),
            InputStep::Text(text_input_step) => text_input_step.get_script_placeholder_name(),
        }
    }
}
