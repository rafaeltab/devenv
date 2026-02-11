use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

/// A picker for yes/no confirmation.
///
/// This picker displays a prompt with Yes/No options and allows
/// the user to select one using arrow keys.
pub struct ConfirmPicker {
    prompt: String,
    default: bool,
    selected: bool,
}

impl ConfirmPicker {
    /// Create a new confirm picker with the given prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: true,
            selected: true,
        }
    }

    /// Set the default selection.
    pub fn with_default(mut self, default: bool) -> Self {
        self.default = default;
        self.selected = default;
        self
    }
}

impl WidgetRef for ConfirmPicker {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Placeholder implementation
        // Full implementation in Phase 4
        let yes_style = if self.selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let no_style = if !self.selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let text = vec![
            Line::from(self.prompt.clone()),
            Line::from(vec![
                Span::styled("Yes", yes_style),
                Span::raw(" / "),
                Span::styled("No", no_style),
            ]),
        ];
        Paragraph::new(text).render(area, buf);
    }
}
