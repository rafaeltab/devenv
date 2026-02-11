use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget, WidgetRef};

use crate::tui::picker_ctx::SuggestionProvider;

/// A picker for text input with autocomplete suggestions.
///
/// This picker provides a text input field with tab completion and
/// suggestion navigation using arrow keys.
pub struct TextPickerWithSuggestions {
    prompt: String,
    input: String,
    suggestions: Vec<String>,
    selected_suggestion: usize,
    provider: Box<dyn SuggestionProvider>,
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
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

        enable_raw_mode().ok()?;

        loop {
            // Create a copy of the data needed for rendering
            let prompt = self.prompt.clone();
            let input = self.input.clone();
            let suggestions = self.suggestions.clone();
            let selected_suggestion = self.selected_suggestion;

            // Draw the picker
            terminal
                .draw(|frame| {
                    let area = frame.area();

                    // Split area into input and suggestions
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(1), Constraint::Min(1)])
                        .split(area);

                    // Render input line
                    let input_text = format!("Query: {}", input);
                    let input_paragraph = Paragraph::new(Line::from(input_text));
                    input_paragraph.render(chunks[0], frame.buffer_mut());

                    // Render suggestions
                    let suggestions_area = chunks[1];
                    let max_visible = suggestions_area.height as usize;

                    if suggestions.is_empty() && !input.is_empty() {
                        // Show "No suggestions available" when input exists but no matches
                        let no_suggestions = Paragraph::new("No suggestions available")
                            .style(Style::default().fg(Color::Gray));
                        no_suggestions.render(suggestions_area, frame.buffer_mut());
                    } else {
                        for (i, suggestion) in suggestions.iter().enumerate().take(max_visible) {
                            let is_selected = i == selected_suggestion;

                            let suggestion_area = Rect {
                                x: suggestions_area.x,
                                y: suggestions_area.y + i as u16,
                                width: suggestions_area.width,
                                height: 1,
                            };

                            if is_selected {
                                // Highlight selected suggestion
                                let style = Style::default().fg(Color::Yellow);
                                let highlighted = Paragraph::new(suggestion.clone()).style(style);
                                highlighted.render(suggestion_area, frame.buffer_mut());
                            } else {
                                let normal = Paragraph::new(suggestion.clone());
                                normal.render(suggestion_area, frame.buffer_mut());
                            }
                        }
                    }
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
                        disable_raw_mode().ok()?;
                        // Return current input (even if empty, as per TPS-005)
                        return Some(self.input.clone());
                    }
                    // Cancel - Escape
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => {
                        disable_raw_mode().ok()?;
                        return None;
                    }
                    // Cancel - Ctrl+C
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => {
                        disable_raw_mode().ok()?;
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
}

impl Widget for TextPickerWithSuggestions {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for TextPickerWithSuggestions {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Split area into input and suggestions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        // Render input line
        let input_text = format!("Query: {}", self.input);
        let input_paragraph = Paragraph::new(Line::from(input_text));
        input_paragraph.render(chunks[0], buf);

        // Render suggestions
        let suggestions_area = chunks[1];
        let max_visible = suggestions_area.height as usize;

        if self.suggestions.is_empty() && !self.input.is_empty() {
            // Show "No suggestions available" when input exists but no matches
            let no_suggestions =
                Paragraph::new("No suggestions available").style(Style::default().fg(Color::Gray));
            no_suggestions.render(suggestions_area, buf);
        } else {
            for (i, suggestion) in self.suggestions.iter().enumerate().take(max_visible) {
                let is_selected = i == self.selected_suggestion;

                let suggestion_area = Rect {
                    x: suggestions_area.x,
                    y: suggestions_area.y + i as u16,
                    width: suggestions_area.width,
                    height: 1,
                };

                if is_selected {
                    // Highlight selected suggestion
                    let style = Style::default().fg(Color::Yellow);
                    let highlighted = Paragraph::new(suggestion.clone()).style(style);
                    highlighted.render(suggestion_area, buf);
                } else {
                    let normal = Paragraph::new(suggestion.clone());
                    normal.render(suggestion_area, buf);
                }
            }
        }
    }
}
