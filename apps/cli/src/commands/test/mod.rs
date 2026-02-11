//! Test commands for the command palette.
//!
//! These commands are only registered when TEST_MODE environment variable
//! is set. They are used for testing the picker implementations.

pub mod test_confirm;
pub mod test_picker;
pub mod test_text_input;
pub mod test_text_input_suggestions;

pub use test_confirm::TestConfirmCommand;
pub use test_picker::TestPickerCommand;
pub use test_text_input::TestTextInputCommand;
pub use test_text_input_suggestions::TestTextInputSuggestionsCommand;
