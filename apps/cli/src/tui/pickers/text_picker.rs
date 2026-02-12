use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, WidgetRef};

use crate::tui::theme::Theme;

/// A picker for text input.
///
/// This picker provides a simple text input field with backspace support
/// and Unicode handling.
///
/// The layout matches the tmux switch command style:
/// - Input area with prompt display
/// - Help footer with colored key hints
pub struct TextPicker {
    prompt: String,
    input: String,
    theme: Theme,
}

impl TextPicker {
    /// Create a new text picker with the given prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            input: String::new(),
            theme: Theme::default(),
        }
    }

    /// Run the text picker and return the entered text or None if cancelled.
    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Option<String> {
        terminal.clear().ok()?;

        loop {
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    self.render_ui(frame.buffer_mut(), area);
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

    /// Render the UI with two-section layout matching tmux switch style.
    fn render_ui(&self, buf: &mut Buffer, area: Rect) {
        let theme = &self.theme;

        // Layout: input (3 lines), help (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input area
                Constraint::Length(1), // Help footer
            ])
            .split(area);

        // Input area with prompt label
        let input_text = Line::from(vec![
            Span::styled(format!("{}: ", self.prompt), theme.primary_style()),
            Span::raw(&self.input),
        ]);
        let input_widget = Paragraph::new(input_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Input")
                .border_style(theme.border_style()),
        );
        input_widget.render(chunks[0], buf);

        // Help footer
        let help = Line::from(vec![
            Span::styled("Enter", theme.success_style()),
            Span::raw(" confirm  "),
            Span::styled("Esc", theme.danger_style()),
            Span::raw(" cancel"),
        ]);
        let help_widget = Paragraph::new(help);
        help_widget.render(chunks[1], buf);
    }
}

impl Widget for TextPicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for TextPicker {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render_ui(buf, area);
    }
}
