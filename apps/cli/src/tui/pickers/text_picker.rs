use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
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
}

impl WidgetRef for TextPicker {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Placeholder implementation
        // Full implementation in Phase 4
        let text = format!("{}: {}", self.prompt, self.input);
        Paragraph::new(text).render(area, buf);
    }
}
