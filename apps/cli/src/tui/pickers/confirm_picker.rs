use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, WidgetRef};

use crate::tui::theme::Theme;

/// A picker for yes/no confirmation.
///
/// This picker displays a prompt with Yes/No options and allows
/// the user to select one using arrow keys or Left/Right.
///
/// The layout matches the tmux switch command style:
/// - Prompt area with Yes/No selection
/// - Help footer with colored key hints
pub struct ConfirmPicker {
    prompt: String,
    default: bool,
    selected: bool,
    theme: Theme,
}

impl ConfirmPicker {
    /// Create a new confirm picker with the given prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            default: true,
            selected: true,
            theme: Theme::default(),
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

    /// Render the UI with two-section layout matching tmux switch style.
    fn render_ui(&self, buf: &mut Buffer, area: Rect) {
        let theme = &self.theme;

        // Layout: prompt (4 lines to fit 2 content lines + borders), help (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Prompt area: 2 content lines + 2 border lines
                Constraint::Length(1), // Help footer
            ])
            .split(area);

        // Yes/No styling
        let yes_style = if self.selected {
            theme.selected_style()
        } else {
            Style::default()
        };
        let no_style = if !self.selected {
            theme.selected_style()
        } else {
            Style::default()
        };

        // Prompt area with Yes/No selection
        let prompt_text = vec![
            Line::from(self.prompt.clone()),
            Line::from(vec![
                Span::styled("Yes", yes_style),
                Span::raw(" / "),
                Span::styled("No", no_style),
            ]),
        ];
        let prompt_widget = Paragraph::new(prompt_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Confirm")
                .border_style(theme.border_style()),
        );
        prompt_widget.render(chunks[0], buf);

        // Help footer
        let help = Line::from(vec![
            Span::styled("Enter", theme.success_style()),
            Span::raw(" confirm  "),
            Span::styled("Esc", theme.danger_style()),
            Span::raw(" cancel  "),
            Span::styled("←/→", theme.primary_style()),
            Span::raw(" or "),
            Span::styled("Tab", theme.info_style()),
            Span::raw(" toggle"),
        ]);
        let help_widget = Paragraph::new(help);
        help_widget.render(chunks[1], buf);
    }
}

impl Widget for ConfirmPicker {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for ConfirmPicker {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render_ui(buf, area);
    }
}
