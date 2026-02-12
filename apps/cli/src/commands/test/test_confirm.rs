use std::env;

use crate::commands::{Command, CommandCtx};

/// Test command for the confirm picker.
///
/// This command displays a confirm picker with prompt from
/// TEST_CONFIRM_PROMPT environment variable.
///
/// Environment variables:
/// - TEST_CONFIRM_PROMPT: The prompt text (default: "Confirm?")
/// - TEST_CONFIRM_DEFAULT: The default selection (default: "true")
#[derive(Debug)]
pub struct TestConfirmCommand;

impl TestConfirmCommand {
    /// Create a new test confirm command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestConfirmCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for TestConfirmCommand {
    fn name(&self) -> &str {
        "test confirm"
    }

    fn description(&self) -> &str {
        "Test the confirm picker"
    }

    fn run(&self, ctx: &mut CommandCtx) {
        // Get prompt and default from environment
        let prompt = env::var("TEST_CONFIRM_PROMPT").unwrap_or_else(|_| "Confirm?".to_string());
        let default = env::var("TEST_CONFIRM_DEFAULT")
            .map(|v| v == "true")
            .unwrap_or(true);

        // Show confirm picker
        let confirmed = ctx.confirm(&prompt, default);

        // Show result in a final confirmation screen before exiting
        // This ensures the result is visible in the PTY buffer
        let result_text = match confirmed {
            Some(true) => "Some(true)",
            Some(false) => "Some(false)",
            None => "None",
        };

        // Display result in a picker that auto-closes after a short delay
        // This ensures the result is captured by PTY before exiting
        ctx.confirm(&format!("Result: {}", result_text), true);

        // Give the test framework time to capture the screen
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
