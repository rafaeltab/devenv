use std::env;

use crate::commands::{Command, CommandCtx};
use crate::tui::picker_ctx::StaticSuggestionProvider;

/// Test command for the text input picker with suggestions.
///
/// This command displays a text input picker with suggestions from
/// TEST_SUGGESTIONS environment variable.
///
/// Environment variables:
/// - TEST_SUGGESTIONS: Comma-separated list of suggestions (default: "apple,application,apply")
#[derive(Debug)]
pub struct TestTextInputSuggestionsCommand;

impl TestTextInputSuggestionsCommand {
    /// Create a new test text input suggestions command.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TestTextInputSuggestionsCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for TestTextInputSuggestionsCommand {
    fn name(&self) -> &str {
        "test text input suggestions"
    }

    fn description(&self) -> &str {
        "Test text input with suggestions"
    }

    fn run(&self, ctx: &mut CommandCtx) {
        // Get suggestions from environment or use defaults
        let suggestions_str =
            env::var("TEST_SUGGESTIONS").unwrap_or_else(|_| "apple,application,apply".to_string());
        let suggestions: Vec<String> = suggestions_str.split(',').map(|s| s.to_string()).collect();

        let provider = StaticSuggestionProvider::new(suggestions);

        // Show picker with suggestions
        let input = ctx.input_with_suggestions("Query", Box::new(provider));

        // Show result in a confirmation screen before exiting
        let result_text = match input {
            Some(text) => format!("Some({})", text),
            None => "None".to_string(),
        };

        // Display result in the picker so it's captured by PTY
        ctx.confirm(&format!("Result: {}", result_text), true);
    }
}
