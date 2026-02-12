use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, WidgetRef};

use crate::tui::picker_ctx::SuggestionProvider;
use crate::tui::theme::Theme;

/// A picker for text input with autocomplete suggestions.
///
/// This picker provides a text input field with tab completion and
/// suggestion navigation using arrow keys.
///
/// The layout matches the tmux switch command style:
/// - Input area with query display
/// - Suggestions list with selection highlight
/// - Help footer with colored key hints
pub struct TextPickerWithSuggestions {
    prompt: String,
    input: String,
    suggestions: Vec<String>,
    selected_suggestion: usize,
    provider: Box<dyn SuggestionProvider>,
    theme: Theme,
}

impl TextPickerWithSuggestions {
    /// Create a new text picker with suggestions.
    pub fn new(prompt: impl Into<String>, provider: Box<dyn SuggestionProvider>) -> Self {
        let mut picker = Self {
            prompt: prompt.into(),
            input: String::new(),
            suggestions: Vec::new(),
            selected_suggestion: 0,
            provider,
            theme: Theme::default(),
        };
        picker.update_suggestions();
        picker
    }

    /// Update suggestions based on current input.
    fn update_suggestions(&mut self) {
        self.suggestions = self.provider.suggestions(&self.input).unwrap_or_default();
        self.selected_suggestion = 0; // Reset to first suggestion
    }

    /// Move selection down in suggestions.
    fn move_down(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_suggestion = (self.selected_suggestion + 1) % self.suggestions.len();
        }
    }

    /// Move selection up in suggestions.
    fn move_up(&mut self) {
        if !self.suggestions.is_empty() {
            if self.selected_suggestion == 0 {
                self.selected_suggestion = self.suggestions.len() - 1;
            } else {
                self.selected_suggestion -= 1;
            }
        }
    }

    /// Complete the current input with the selected suggestion.
    fn complete(&mut self) {
        if let Some(suggestion) = self.suggestions.get(self.selected_suggestion) {
            self.input = suggestion.clone();
        }
    }

    /// Run the picker and return the entered text or None if cancelled.
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
                        // Return current input (even if empty, as per TPS-005)
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
                        let _ = self.input.pop();
                        self.update_suggestions();
                    }
                    // Clear line - Ctrl+U (common TUI pattern)
                    KeyEvent {
                        code: KeyCode::Char('u'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        self.input.clear();
                        self.update_suggestions();
                    }
                    // Tab completion
                    KeyEvent {
                        code: KeyCode::Tab, ..
                    } => {
                        self.complete();
                        self.update_suggestions();
                    }
                    // Navigation down
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } => {
                        self.move_down();
                    }
                    // Navigation up
                    KeyEvent {
                        code: KeyCode::Up, ..
                    } => {
                        self.move_up();
                    }
                    // Navigation - Ctrl+J (down)
                    KeyEvent {
                        code: KeyCode::Char('j'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        self.move_down();
                    }
                    // Navigation - Ctrl+K (up)
                    KeyEvent {
                        code: KeyCode::Char('k'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        self.move_up();
                    }
                    // Character input
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => {
                        self.input.push(c);
                        self.update_suggestions();
                    }
                    _ => {}
                }
            }
        }
    }

    /// Render the UI with three-section layout matching tmux switch style.
    fn render_ui(&self, buf: &mut Buffer, area: Rect) {
        let theme = &self.theme;

        // Layout: input (3 lines), suggestions (min 3), help (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input area
                Constraint::Min(3),    // Suggestions list
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

        // Suggestions list
        let list_area = chunks[1];
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Suggestions")
            .border_style(theme.border_style());
        let inner_area = block.inner(list_area);
        block.render(list_area, buf);

        // Render suggestions manually with selection highlighting
        if self.suggestions.is_empty() && !self.input.is_empty() {
            // Show "No suggestions available" when input exists but no matches
            let no_suggestions =
                Paragraph::new("No suggestions available").style(theme.muted_style());
            no_suggestions.render(inner_area, buf);
        } else {
            let max_visible = inner_area.height as usize;
            for (i, suggestion) in self.suggestions.iter().enumerate().take(max_visible) {
                let is_selected = i == self.selected_suggestion;

                let suggestion_area = Rect {
                    x: inner_area.x,
                    y: inner_area.y + i as u16,
                    width: inner_area.width,
                    height: 1,
                };

                if is_selected {
                    // Highlight selected suggestion
                    let highlighted =
                        Paragraph::new(suggestion.clone()).style(theme.selected_style());
                    highlighted.render(suggestion_area, buf);
                } else {
                    let normal = Paragraph::new(suggestion.clone());
                    normal.render(suggestion_area, buf);
                }
            }
        }

        // Help footer
        let help = Line::from(vec![
            Span::styled("Enter", theme.success_style()),
            Span::raw(" confirm  "),
            Span::styled("Esc", theme.danger_style()),
            Span::raw(" cancel  "),
            Span::styled("Tab", theme.info_style()),
            Span::raw(" complete  "),
            Span::styled("↑/↓", theme.primary_style()),
            Span::raw(" navigate"),
        ]);
        let help_widget = Paragraph::new(help);
        help_widget.render(chunks[2], buf);
    }
}

impl Widget for TextPickerWithSuggestions {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for TextPickerWithSuggestions {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render_ui(buf, area);
    }
}
