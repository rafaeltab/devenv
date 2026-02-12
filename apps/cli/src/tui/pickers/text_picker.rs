use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

/// A picker for text input.
///
/// This picker provides a simple text input field with backspace support
/// and Unicode handling.
pub struct TextPicker {
    prompt: String,
    input: String,
}

impl TextPicker {
    /// Create a new text picker with the given prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            input: String::new(),
        }
    }

    /// Run the text picker and return the entered text or None if cancelled.
    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Option<String> {
        terminal.clear().ok()?;

        loop {
            // Create a copy of the data needed for rendering
            let prompt = self.prompt.clone();
            let input = self.input.clone();

            // Draw the picker
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    let text = format!("{}: {}", prompt, input);
                    let paragraph = Paragraph::new(Line::from(text)).style(Style::default());
                    paragraph.render(area, frame.buffer_mut());
                })
                .ok()?;

            // Handle input
            if let Ok(Event::Key(key)) = crossterm::event::read() {
                match key {
                    // Confirm - Enter
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        if self.input.is_empty() {
                            return None; // Empty input treated as cancel
                        }
                        return Some(self.input.clone());
                    }
                    // Cancel - Escape
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => {
                        return None;
                    }
                    // Cancel - Ctrl+C
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        return None;
                    }
                    // Backspace
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => {
                        // Remove the last character (handles Unicode properly)
                        let _ = self.input.pop();
                    }
                    // Character input (supports Unicode)
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => {
                        self.input.push(c);
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Widget for TextPicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for TextPicker {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let text = format!("{}: {}", self.prompt, self.input);
        let paragraph = Paragraph::new(Line::from(text)).style(Style::default());
        paragraph.render(area, buf);
    }
}
