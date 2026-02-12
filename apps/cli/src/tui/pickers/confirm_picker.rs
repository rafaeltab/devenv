use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

/// A picker for yes/no confirmation.
///
/// This picker displays a prompt with Yes/No options and allows
/// the user to select one using arrow keys or Left/Right.
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

    /// Toggle between Yes and No.
    fn toggle(&mut self) {
        self.selected = !self.selected;
    }

    /// Move selection left (to Yes).
    fn move_left(&mut self) {
        self.selected = true; // Yes is on the left
    }

    /// Move selection right (to No).
    fn move_right(&mut self) {
        self.selected = false; // No is on the right
    }

    /// Run the confirm picker and return the user's choice.
    ///
    /// Returns:
    /// - Some(true) if Yes is selected
    /// - Some(false) if No is selected
    /// - None if cancelled
    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Option<bool> {
        terminal.clear().ok()?;

        loop {
            // Create a copy of the data needed for rendering
            let prompt = self.prompt.clone();
            let selected = self.selected;

            // Draw the picker
            terminal
                .draw(|frame| {
                    let area = frame.area();

                    let yes_style = if selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    let no_style = if !selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };

                    let text = vec![
                        Line::from(prompt),
                        Line::from(vec![
                            Span::styled("Yes", yes_style),
                            Span::raw(" / "),
                            Span::styled("No", no_style),
                        ]),
                    ];
                    Paragraph::new(text).render(area, frame.buffer_mut());
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
                        return Some(self.selected);
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
                    // Navigation - Left (select Yes)
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        self.move_left();
                    }
                    // Navigation - Right (select No)
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        self.move_right();
                    }
                    // Toggle with Tab
                    KeyEvent {
                        code: KeyCode::Tab, ..
                    } => {
                        self.toggle();
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Widget for ConfirmPicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for ConfirmPicker {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
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
