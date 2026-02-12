//! Characterization tests for TerminalBuffer
//!
//! These tests document the current behavior of the custom TerminalBuffer
//! implementation. They serve as the baseline for the migration to wezterm-term.

#[cfg(test)]
mod tests {
    use super::super::TerminalBuffer;

    #[test]
    fn test_new_creates_empty_terminal() {
        let term = TerminalBuffer::new(10, 20);
        let screen = term.render();
        assert!(screen.trim().is_empty(), "New terminal should be empty");
    }

    #[test]
    fn test_simple_text_rendering() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"Hello World");
        let screen = term.render();
        assert!(screen.contains("Hello World"), "Should render simple text");
    }

    #[test]
    fn test_cursor_home_and_text() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"\x1b[H");
        term.process_bytes(b"First Line");
        let screen = term.render();
        let lines: Vec<_> = screen.lines().collect();
        assert!(
            lines.first().is_some_and(|l| l.contains("First Line")),
            "Text at cursor position (0,0) should be on first line. Got: {}",
            screen
        );
    }

    #[test]
    fn test_clear_screen() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"Hello");
        term.process_bytes(b"\x1b[2J");
        let screen = term.render();
        assert!(!screen.contains("Hello"), "Screen should be cleared");
    }

    #[test]
    fn test_color_output() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"\x1b[38;2;255;0;0mRed Text\x1b[0m");
        let screen = term.render();
        assert!(screen.contains("Red Text"), "Should render colored text");

        let cell = term.get_cell(0, 0);
        assert!(cell.is_some(), "Should be able to get cell at (0, 0)");
        let cell = cell.unwrap();
        assert_eq!(cell.fg_r, 255, "Red component should be 255");
        assert_eq!(cell.fg_g, 0, "Green component should be 0");
        assert_eq!(cell.fg_b, 0, "Blue component should be 0");
    }

    #[test]
    fn test_find_text() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"Search for this text");
        let positions = term.find_all_text("this");
        assert_eq!(positions.len(), 1, "Should find one occurrence");
        assert_eq!(
            positions[0],
            (0, 11),
            "Should be at correct position (0, 11)"
        );
    }

    #[test]
    fn test_find_text_multiple_occurrences() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"test test test");
        let positions = term.find_all_text("test");
        assert_eq!(positions.len(), 3, "Should find three occurrences");
        assert_eq!(positions[0], (0, 0));
        assert_eq!(positions[1], (0, 5));
        assert_eq!(positions[2], (0, 10));
    }

    #[test]
    fn test_find_text_not_found() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"Hello World");
        let positions = term.find_all_text("missing");
        assert!(
            positions.is_empty(),
            "Should return empty when text not found"
        );
    }

    #[test]
    fn test_newline_moves_cursor() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"Line1\nLine2");
        let screen = term.render();
        let lines: Vec<_> = screen.lines().collect();
        assert!(
            lines[0].contains("Line1"),
            "First line should contain Line1"
        );
        assert!(
            lines[1].contains("Line2"),
            "Second line should contain Line2"
        );
    }

    #[test]
    fn test_carriage_return() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"Hello\rWorld");
        let screen = term.render();
        assert!(screen.contains("World"), "CR should return cursor to start");
        assert!(!screen.contains("Hello"), "Hello should be overwritten");
    }

    #[test]
    fn test_unicode_characters() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes("日本語".as_bytes());
        let screen = term.render();
        assert!(
            screen.contains("日本語"),
            "Should handle Japanese characters"
        );
    }

    #[test]
    fn test_box_drawing_characters() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes("╭───╮".as_bytes());
        let screen = term.render();
        assert!(screen.contains("╭"), "Should contain corner character");
        assert!(screen.contains("╮"), "Should contain corner character");
        assert!(screen.contains("─"), "Should contain horizontal line");
    }

    #[test]
    fn test_cursor_position_absolute() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"\x1b[5;10H");
        term.process_bytes(b"Positioned");
        let screen = term.render();
        let lines: Vec<_> = screen.lines().collect();
        assert!(
            lines.get(4).is_some_and(|l| l.contains("Positioned")),
            "Text should be at row 5 (index 4), got:\n{}",
            screen
        );
    }

    #[test]
    fn test_backspace() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"AB\x08C");
        let screen = term.render();
        assert!(screen.contains("AC"), "Backspace should move cursor back");
    }

    #[test]
    fn test_tab_handling() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"A\tB");
        let cell = term.get_cell(0, 8);
        assert!(cell.is_some());
        assert_eq!(
            cell.unwrap().character,
            'B',
            "Tab should move to next tab stop (8)"
        );
    }

    #[test]
    fn test_256_color_indexed() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"\x1b[38;5;196mText\x1b[0m");
        let cell = term.get_cell(0, 0).unwrap();
        assert_eq!(cell.fg_r, 255, "Color 196 should be bright red");
        assert_eq!(cell.fg_g, 0);
        assert_eq!(cell.fg_b, 0);
    }

    #[test]
    fn test_reset_attributes() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"\x1b[31mRed\x1b[0mReset");
        let red_cell = term.get_cell(0, 0).unwrap();
        // Verify red cell has more red than green/blue (exact values differ between implementations)
        assert!(
            red_cell.fg_r > red_cell.fg_g,
            "Red cell should have more red than green"
        );
        assert!(
            red_cell.fg_r > red_cell.fg_b,
            "Red cell should have more red than blue"
        );

        let reset_cell = term.get_cell(0, 7).unwrap();
        assert_eq!(reset_cell.fg_r, 255, "Should reset to white");
        assert_eq!(reset_cell.fg_g, 255);
        assert_eq!(reset_cell.fg_b, 255);
    }

    #[test]
    fn test_clear_buffer_method() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"Hello World");
        term.clear();
        let screen = term.render();
        assert!(screen.trim().is_empty(), "Clear should reset buffer");
    }

    #[test]
    fn test_get_cell_out_of_bounds() {
        let term = TerminalBuffer::new(10, 40);
        assert!(
            term.get_cell(100, 0).is_none(),
            "Row out of bounds should return None"
        );
        assert!(
            term.get_cell(0, 100).is_none(),
            "Col out of bounds should return None"
        );
    }

    #[test]
    fn test_cursor_save_and_restore() {
        let mut term = TerminalBuffer::new(10, 40);
        term.process_bytes(b"\x1b[5;10H");
        term.process_bytes(b"\x1b[s");
        term.process_bytes(b"\x1b[1;1H");
        term.process_bytes(b"First");
        term.process_bytes(b"\x1b[u");
        term.process_bytes(b"Middle");
        let screen = term.render();
        let lines: Vec<_> = screen.lines().collect();
        assert!(lines[0].contains("First"), "Should write at (1,1)");
        assert!(
            lines.get(4).is_some_and(|l| l.contains("Middle")),
            "Should write at saved position (5, 10)"
        );
    }

    #[test]
    fn test_partial_eq() {
        let term1 = TerminalBuffer::new(10, 40);
        let term2 = TerminalBuffer::new(10, 40);
        assert_eq!(term1, term2, "Two empty terminals should be equal");
    }

    #[test]
    fn test_current_behavior_snapshot() {
        let mut term = TerminalBuffer::new(40, 120);

        // Box drawing characters: ╭ (U+256D), ─ (U+2500), ╮ (U+256E), │ (U+2502), ╰ (U+2570)
        let bytes = "\x1b[2J\x1b[1;1H\x1b[38;5;15;49m╭ Enter your command: ─────────────────────────────────────────────────────────────────────────────────────────────────╮\x1b[2;1H│\x1b[2;3H\x1b[39;48;5;8m \x1b[2;120H\x1b[38;5;15;49m│\x1b[3;1H╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯".as_bytes();

        term.process_bytes(bytes);

        let screen = term.render();
        println!("Current implementation output:");
        println!("{}", screen);

        assert!(
            screen.contains("Enter your command:"),
            "Should capture first line with border and title. Got:\n{}",
            screen
        );
    }
}
