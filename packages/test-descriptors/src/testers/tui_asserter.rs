use super::keys::Key;
use super::text_match::TextMatch;

/// Trait defining the interface for TUI test assertions and interactions.
///
/// All TUI testers return types that implement this trait, allowing tests
/// to be written generically against any TUI execution backend.
pub trait TuiAsserter {
    // Lifecycle

    /// Wait for the TUI to settle (no new output for a period).
    fn wait_for_settle(&mut self);

    /// Wait for the TUI to settle with custom timeout parameters.
    ///
    /// # Arguments
    /// * `timeout_ms` - How long to wait for no new output before considering settled
    /// * `max_wait_ms` - Maximum total time to wait
    fn wait_for_settle_ms(&mut self, timeout_ms: u64, max_wait_ms: u64);

    /// Wait for a specific number of milliseconds.
    fn wait_ms(&mut self, ms: u64);

    /// Wait for the command to complete and return its exit code.
    fn expect_completion(&mut self) -> i32;

    /// Wait for the command to complete and assert the expected exit code.
    fn expect_exit_code(&mut self, expected: i32);

    // Input

    /// Type text into the TUI.
    fn type_text(&mut self, text: &str);

    /// Press a single key.
    fn press_key(&mut self, key: Key);

    /// Send multiple keys in sequence.
    fn send_keys(&mut self, keys: &[Key]);

    // Queries

    /// Find text in the TUI output and return a TextMatch for assertions.
    fn find_text(&self, text: &str) -> TextMatch;

    /// Find all occurrences of text in the TUI output.
    fn find_all_text(&self, text: &str) -> Vec<TextMatch>;

    /// Get the current screen content as a string.
    fn screen(&self) -> String;

    // Debug

    /// Dump the current screen to stderr for debugging.
    fn dump_screen(&self);
}
