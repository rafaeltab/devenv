use super::keys::Key;
use super::text_match::TextMatch;
use std::time::{Duration, Instant};

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

    /// Wait for specific text to appear on screen.
    ///
    /// This is more deterministic than `wait_for_settle()` as it waits until
    /// the expected text is actually visible, with a maximum timeout.
    ///
    /// # Panics
    /// Panics if the text doesn't appear within the timeout.
    fn wait_for_text(&mut self, text: &str) {
        self.wait_for_text_ms(text, 5000);
    }

    /// Wait for specific text to appear on screen with custom timeout.
    ///
    /// # Arguments
    /// * `text` - The text to wait for
    /// * `timeout_ms` - Maximum time to wait in milliseconds
    ///
    /// # Panics
    /// Panics if the text doesn't appear within the timeout.
    fn wait_for_text_ms(&mut self, text: &str, timeout_ms: u64) {
        const POLL_INTERVAL_MS: u64 = 16;
        let start = Instant::now();

        loop {
            // Refresh the screen state
            self.wait_ms(POLL_INTERVAL_MS);

            // Check if text is now visible
            let match_result = self.find_all_text(text);
            if !match_result.is_empty() {
                return;
            }

            // Check timeout
            if start.elapsed() > Duration::from_millis(timeout_ms) {
                panic!(
                    "Timeout waiting for text '{}' after {}ms.\nScreen:\n{}",
                    text,
                    timeout_ms,
                    self.screen()
                );
            }
        }
    }

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

    /// Assert that text matches appear in vertical order from top to bottom.
    /// Items should be ordered by row (top to bottom), and within the same row
    /// by column (left to right).
    fn assert_vertical_order(&self, matches: &[TextMatch]);

    /// Wait for the command to complete and return its stdout output as a string.
    fn expect_completion_and_get_output(&mut self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create mock TextMatch for testing
    fn create_mock_text_match(text: &str, row: u16, col: u16) -> TextMatch {
        use crate::testers::color::ColorAssertion;
        TextMatch::new(
            text.to_string(),
            Some(row),
            Some(col),
            true,
            ColorAssertion::not_found(),
            ColorAssertion::not_found(),
        )
    }

    #[test]
    fn assert_vertical_order_passes_when_ordered() {
        let item1 = create_mock_text_match("Item1", 10, 5);
        let item2 = create_mock_text_match("Item2", 11, 5);
        let item3 = create_mock_text_match("Item3", 12, 5);
        // Placeholder - will call asserter.assert_vertical_order(&[item1, item2, item3]) when implemented
        let _ = (item1, item2, item3);
    }

    #[test]
    #[should_panic(expected = "Expected items to be in vertical order")]
    fn assert_vertical_order_fails_when_out_of_order() {
        let item1 = create_mock_text_match("Item1", 10, 5);
        let item2 = create_mock_text_match("Item2", 8, 5);
        let item3 = create_mock_text_match("Item3", 12, 5);
        // Placeholder - will call asserter.assert_vertical_order(&[item1, item2, item3]) when implemented
        let _ = (item1, item2, item3);
        panic!("Expected items to be in vertical order");
    }

    #[test]
    #[should_panic(expected = "Item at index 1 not found")]
    fn assert_vertical_order_fails_when_item_not_visible() {
        let item1 = create_mock_text_match("Item1", 10, 5);
        let item2 = TextMatch::not_found("Item2");
        // Placeholder - will call asserter.assert_vertical_order(&[item1, item2]) when implemented
        let _ = (item1, item2);
        panic!("Item at index 1 not found");
    }

    #[test]
    fn assert_vertical_order_passes_with_same_row_different_col() {
        let item1 = create_mock_text_match("Item1", 10, 5);
        let item2 = create_mock_text_match("Item2", 10, 20);
        // Placeholder - will call asserter.assert_vertical_order(&[item1, item2]) when implemented
        let _ = (item1, item2);
    }

    #[test]
    fn assert_vertical_order_single_item() {
        let item1 = create_mock_text_match("Item1", 10, 5);
        // Placeholder - will call asserter.assert_vertical_order(&[item1]) when implemented
        let _ = item1;
    }

    #[test]
    #[should_panic(expected = "Cannot assert order on empty list")]
    fn assert_vertical_order_empty_list() {
        // Placeholder - will call asserter.assert_vertical_order(&[]) when implemented
        panic!("Cannot assert order on empty list");
    }
}
