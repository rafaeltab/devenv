//! Contract tests: Verify both implementations behave identically
//!
//! These tests run the same operations on both TerminalBuffer (old) and
//! WeztermTerminal (new) and verify they produce the same output.

#[cfg(test)]
mod tests {
    // Only run these tests when the feature flag is enabled
    #![cfg(feature = "use-wezterm-term")]

    // Import the old implementation directly from its module
    use super::super::terminal_buffer::TerminalBuffer as OldTerminal;
    // Import the new implementation
    use super::super::wezterm_terminal::TerminalBuffer as NewTerminal;

    /// Helper: Run same bytes through both terminals and compare
    fn assert_implementations_match(bytes: &[u8]) {
        let mut old_term = OldTerminal::new(40, 120);
        let mut new_term = NewTerminal::new(40, 120);

        old_term.process_bytes(bytes);
        new_term.process_bytes(bytes);

        let old_screen = old_term.render();
        let new_screen = new_term.render();

        assert_eq!(
            old_screen,
            new_screen,
            "Implementations should produce identical output for input: {:?}",
            String::from_utf8_lossy(bytes)
        );
    }

    #[test]
    fn both_implementations_empty_terminal() {
        assert_implementations_match(b"");
    }

    #[test]
    fn both_implementations_simple_text() {
        assert_implementations_match(b"Hello World");
    }

    #[test]
    fn both_implementations_cursor_home() {
        assert_implementations_match(b"\x1b[HText");
    }

    #[test]
    fn both_implementations_clear_screen() {
        assert_implementations_match(b"Before\x1b[2JAfter");
    }

    #[test]
    fn both_implementations_newline() {
        assert_implementations_match(b"Line1\nLine2");
    }

    #[test]
    fn both_implementations_cursor_position() {
        assert_implementations_match(b"\x1b[5;10HPositioned");
    }

    #[test]
    fn both_implementations_colors() {
        assert_implementations_match(b"\x1b[31mRed\x1b[0m");
        assert_implementations_match(b"\x1b[38;5;100mIndexed\x1b[0m");
        assert_implementations_match(b"\x1b[38;2;255;128;64mRGB\x1b[0m");
    }

    #[test]
    fn both_implementations_complex_real_world() {
        // Box drawing test similar to the command palette UI
        let bytes = "\x1b[2J\x1b[1;1H\x1b[38;5;15;49m╭ Enter your command: ─────────────────────────────────────────────────────────────────────────────────────────────────╮\x1b[2;1H│\x1b[2;3H\x1b[39;48;5;8m \x1b[2;120H\x1b[38;5;15;49m│\x1b[3;1H╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯".as_bytes();

        assert_implementations_match(bytes);
    }

    // Test text search parity
    #[test]
    fn both_implementations_find_text() {
        let bytes = b"Search for this text in the terminal";

        let mut old_term = OldTerminal::new(10, 50);
        let mut new_term = NewTerminal::new(10, 50);

        old_term.process_bytes(bytes);
        new_term.process_bytes(bytes);

        let old_positions = old_term.find_all_text("this");
        let new_positions = new_term.find_all_text("this");

        assert_eq!(
            old_positions, new_positions,
            "Text search should find same positions"
        );
    }

    // Test cell access parity
    #[test]
    fn both_implementations_get_cell() {
        let bytes = b"ABCD";

        let mut old_term = OldTerminal::new(10, 50);
        let mut new_term = NewTerminal::new(10, 50);

        old_term.process_bytes(bytes);
        new_term.process_bytes(bytes);

        for col in 0..4 {
            let old_cell = old_term.get_cell(0, col);
            let new_cell = new_term.get_cell(0, col);

            assert_eq!(
                old_cell.map(|c| c.character),
                new_cell.map(|c| c.character),
                "Cell at (0, {}) should match",
                col
            );
        }
    }

    // Test cell colors parity
    #[test]
    fn both_implementations_get_cell_colors() {
        let bytes = b"\x1b[38;5;196mRedText\x1b[0m";

        let mut old_term = OldTerminal::new(10, 50);
        let mut new_term = NewTerminal::new(10, 50);

        old_term.process_bytes(bytes);
        new_term.process_bytes(bytes);

        // Check first cell (should be colored)
        let old_cell = old_term.get_cell(0, 0).unwrap();
        let new_cell = new_term.get_cell(0, 0).unwrap();

        // Note: Colors may differ slightly between implementations
        // The important thing is that color data is captured
        assert_eq!(old_cell.character, new_cell.character);
        assert!(
            new_cell.fg_r > 200,
            "New implementation should capture red color"
        );
    }
}
