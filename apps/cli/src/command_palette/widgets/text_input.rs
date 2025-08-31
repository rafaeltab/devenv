use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::command_palette::tui::KeyListener;

pub struct TextInput {
    pub title: String,
}

#[derive(Default)]
pub struct TextInputState {
    pub current_input: String,
    pub cursor_index: usize,
}

impl KeyListener for TextInputState {
    fn handle_key(&mut self, key: KeyEvent) {
        if let KeyCode::Char(char) = key.code {
            let mut c: char = char;
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                c = c.to_ascii_uppercase();
            }
            self.current_input.insert(self.cursor_index, c);
            self.move_right();
        }

        if let KeyCode::Left = key.code {
            self.move_left();
        }

        if let KeyCode::Right = key.code {
            self.move_right();
        }

        if let KeyCode::Backspace = key.code {
            if !self.current_input.is_empty() {
                self.current_input.pop();
                self.move_left()
            }
        }
    }
}

impl TextInputState {
    fn move_left(&mut self) {
        if self.cursor_index == 0 {
            return;
        }

        self.cursor_index -= 1;
    }

    fn move_right(&mut self) {
        if self.cursor_index >= self.current_input.len() - 1 {
            self.cursor_index = self.current_input.len() - 1;
        }

        self.cursor_index += 1;
    }
}

impl StatefulWidget for TextInput {
    type State = TextInputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let spans = split_string_at_cursor(&state.current_input, state.cursor_index);
        let text = Paragraph::new(Line::from(spans)).block(
            Block::new()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Reset))
                .title(format!(" {} ", self.title)),
        );
        text.render(area, buf);
    }
}

fn split_string_at_cursor(input_str: &'_ str, cursor_pos: usize) -> Vec<Span<'_>> {
    let len = input_str.len();

    // Clamp cursor_pos to be at most len.
    // A cursor at `len` means it's positioned after the last character.
    let cursor = cursor_pos.min(len);

    // 1. First string: everything before the cursor
    // If cursor is 0, this slice input_str[0..0] will be empty.
    let before_cursor = input_str[0..cursor].to_string();

    // 2. Second string: the character at the cursor, or a space if at the end
    let cursor_char_str = if cursor < len {
        // Cursor is within the string, take the character
        // Slicing input_str[cursor..cursor+1] gets the char as a &str
        input_str[cursor..cursor + 1].to_string()
    } else {
        // Cursor is at len (i.e., at the end of the string)
        " ".to_string()
    };

    // 3. Third string: everything after the cursor character.
    // Empty if the cursor is at the end or second to last position.
    // "Second to last position" means cursor points to the last character.
    // "End position" means cursor points after the last character.
    let after_cursor = if cursor + 1 < len {
        // There are characters after the one at the cursor position
        input_str[cursor + 1..].to_string()
    } else {
        // - If cursor == len - 1 (points to last char), then cursor + 1 == len.
        //   len < len is false. So, empty.
        // - If cursor == len (points after last char), then cursor + 1 == len + 1.
        //   len + 1 < len is false. So, empty.
        "".to_string()
    };

    vec![
        " ".into(),
        before_cursor.into(),
        cursor_char_str.on_dark_gray(),
        after_cursor.into(),
    ]
}
