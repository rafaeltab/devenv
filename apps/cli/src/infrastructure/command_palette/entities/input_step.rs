use std::collections::HashMap;

use duct::cmd;

use crate::domain::command_palette::entities::{
    input_step::{
        BaseInputStep, InputOption, InputStepDetails, InputStepId, SelectInputStep, TextInputStep,
        ValidationOutcome,
    },
    value_objects::bash_script::{BashScriptPlaceholderName, BashScriptTemplate},
};

pub struct SelectInputStepImpl {
    id: InputStepId,
    script_placeholder: BashScriptPlaceholderName,
    details: InputStepDetails,
    order: u32,
    options: Vec<InputOption>,
}

impl SelectInputStep for SelectInputStepImpl {
    fn get_options(&self) -> &Vec<InputOption> {
        &self.options
    }
}

impl BaseInputStep for SelectInputStepImpl {
    fn get_id(&self) -> &InputStepId {
        &self.id
    }

    fn get_script_placeholder_name(&self) -> &BashScriptPlaceholderName {
        &self.script_placeholder
    }

    fn get_details(&self) -> &InputStepDetails {
        &self.details
    }

    fn get_order(&self) -> u32 {
        self.order
    }
}

pub struct TextInputStepImpl {
    id: InputStepId,
    script_placeholder: BashScriptPlaceholderName,
    details: InputStepDetails,
    order: u32,
    validation_script: Option<BashScriptTemplate>,
}

impl TextInputStep for TextInputStepImpl {
    fn validate_answer(&self, answer: String) -> ValidationOutcome {
        if let Some(validation_script) = &self.validation_script {
            let script_result = validation_script.resolve(HashMap::from([(
                BashScriptPlaceholderName("input".to_string()),
                answer,
            )]));

            return match script_result {
                Ok(script) => {
                    let res = cmd("bash", ["-c", &script]).stdout_capture().run();

                    return match res {
                        Ok(ok) => ValidationOutcome {
                            is_valid: ok.status.success(),
                            feedback_message: if ok.status.success() {
                                None
                            } else {
                                Some(String::from_utf8(ok.stdout).expect(""))
                            },
                        },
                        Err(err) => ValidationOutcome {
                            is_valid: false,
                            feedback_message: Some(err.to_string()),
                        },
                    };
                }
                Err(_) => ValidationOutcome {
                    is_valid: false,
                    feedback_message: Some(
                        "The validation script for this text input is invalid".to_string(),
                    ),
                },
            };
        }

        ValidationOutcome {
            is_valid: true,
            feedback_message: None,
        }
    }
}

impl BaseInputStep for TextInputStepImpl {
    fn get_id(&self) -> &InputStepId {
        &self.id
    }

    fn get_script_placeholder_name(&self) -> &BashScriptPlaceholderName {
        &self.script_placeholder
    }

    fn get_details(&self) -> &InputStepDetails {
        &self.details
    }

    fn get_order(&self) -> u32 {
        self.order
    }
}
