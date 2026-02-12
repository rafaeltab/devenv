use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, WidgetRef};
use sublime_fuzzy::{FuzzySearch, Scoring};

use crate::tui::picker_item::PickerItem;
use crate::tui::theme::Theme;

/// A picker for selecting items from a list with fuzzy search.
///
/// This picker displays a list of items that can be filtered using
/// fuzzy search. Users can navigate with arrow keys and select with Enter.
///
/// The layout matches the tmux switch command style:
/// - Input area with query display
/// - Scrollable matches list with selection highlight
/// - Help footer with colored key hints
pub struct SelectPicker<T: PickerItem> {
    items: Vec<T>,
    selected: usize,
    query: String,
    filtered_indices: Vec<(usize, isize)>, // (index, score) pairs sorted by score
    theme: Theme,
}

impl<T: PickerItem> SelectPicker<T> {
    /// Create a new select picker with the given items.
    pub fn new(items: Vec<T>) -> Self {
        let filtered_indices: Vec<(usize, isize)> = (0..items.len()).map(|i| (i, 0)).collect();
        Self {
            items,
            selected: 0,
            query: String::new(),
            filtered_indices,
            theme: Theme::default(),
        }
    }

    /// Set the initial query.
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self.update_filter();
        self
    }

    /// Update the filtered indices based on the current query.
    fn update_filter(&mut self) {
        if self.query.is_empty() {
            // No query - show all items in original order
            self.filtered_indices = (0..self.items.len()).map(|i| (i, 0)).collect();
        } else {
            // Filter and score items using fuzzy matching
            let query_lower = self.query.to_lowercase();
            let mut matches: Vec<(usize, isize)> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let search_text = item.search_text();
                    let search_lower = search_text.to_lowercase();
                    let scoring = Scoring::default();

                    // Use case-insensitive matching by lowercasing both query and text
                    let result = FuzzySearch::new(&query_lower, &search_lower)
                        .score_with(&scoring)
                        .best_match();

                    // Apply length penalty - shorter matches with same prefix should rank higher
                    result.map(|m| {
                        let base_score = m.score();
                        // Small bonus for shorter text (better match density)
                        let length_bonus = (100.0 / search_text.len() as f64) as isize;
                        let adjusted_score = base_score + length_bonus;
                        (idx, adjusted_score)
                    })
                })
                .collect();

            // Sort by score (descending) - higher scores are better matches
            matches.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_indices = matches;
        }

        // Reset selection to first item if out of bounds
        if self.selected >= self.filtered_indices.len() && !self.filtered_indices.is_empty() {
            self.selected = 0;
        }
    }

    /// Move selection down (wraps around).
    fn move_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = (self.selected + 1) % self.filtered_indices.len();
        }
    }

    /// Move selection up (wraps around).
    fn move_up(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected == 0 {
                self.selected = self.filtered_indices.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    /// Handle user input and return the selected item or None if cancelled.
    pub fn run(
        &mut self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Option<&T> {
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
                    // Navigation - Down
                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    } => {
                        self.move_down();
                    }
                    // Navigation - Up
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
                    // Selection - Enter
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        if let Some(&(idx, _)) = self.filtered_indices.get(self.selected) {
                            return Some(&self.items[idx]);
                        }
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
                        self.query.pop();
                        self.update_filter();
                    }
                    // Character input
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => {
                        self.query.push(c);
                        self.update_filter();
                        // Reset selection to first item when filtering
                        self.selected = 0;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Render the UI with three-section layout matching tmux switch style.
    fn render_ui(&self, buf: &mut Buffer, area: Rect) {
        let theme = &self.theme;

        // Layout: input (3 lines), list (min 3), help (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input area
                Constraint::Min(3),    // Matches list
                Constraint::Length(1), // Help footer
            ])
            .split(area);

        // Input area with "Query:" label
        let input_text = Line::from(vec![
            Span::styled("Query: ", theme.primary_style()),
            Span::raw(&self.query),
        ]);
        let input_widget = Paragraph::new(input_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Fuzzy Picker")
                .border_style(theme.border_style()),
        );
        input_widget.render(chunks[0], buf);

        // Matches list
        let list_area = chunks[1];
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Matches")
            .border_style(theme.border_style());
        let inner_area = block.inner(list_area);
        block.render(list_area, buf);

        // Render items manually with selection highlighting
        if self.filtered_indices.is_empty() && !self.query.is_empty() {
            // Show "No matches" message
            let no_matches = Paragraph::new("No matches").style(theme.muted_style());
            no_matches.render(inner_area, buf);
        } else {
            let max_visible = inner_area.height as usize;
            for (display_idx, &(item_idx, _score)) in
                self.filtered_indices.iter().enumerate().take(max_visible)
            {
                let is_selected = display_idx == self.selected;
                let item = &self.items[item_idx];

                let item_area = Rect {
                    x: inner_area.x,
                    y: inner_area.y + display_idx as u16,
                    width: inner_area.width,
                    height: 1,
                };

                // Render the item widget with selection state
                item.render(is_selected).render_ref(item_area, buf);
            }
        }

        // Help footer
        let help = Line::from(vec![
            Span::styled("Enter", theme.success_style()),
            Span::raw(" confirm  "),
            Span::styled("Esc", theme.danger_style()),
            Span::raw(" cancel  "),
            Span::styled("↑/↓", theme.primary_style()),
            Span::raw(" navigate  "),
            Span::styled("Type", theme.info_style()),
            Span::raw(" to filter"),
        ]);
        let help_widget = Paragraph::new(help);
        help_widget.render(chunks[2], buf);
    }
}

impl<T: PickerItem> Widget for SelectPicker<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Delegate to WidgetRef implementation
        self.render_ref(area, buf);
    }
}

impl<T: PickerItem> WidgetRef for SelectPicker<T> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.render_ui(buf, area);
    }
}

/// A simple item implementation for testing.
///
/// This is a basic `PickerItem` implementation that displays text directly.
#[derive(Debug, Clone)]
pub struct SimpleItem {
    text: String,
}

impl SimpleItem {
    /// Create a new simple item.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl PickerItem for SimpleItem {
    fn constraint(&self) -> ratatui::layout::Constraint {
        ratatui::layout::Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        &self.text
    }

    fn render(&self, selected: bool) -> Box<dyn WidgetRef> {
        Box::new(SimpleItemWidget {
            text: self.text.clone(),
            selected,
        })
    }
}

struct SimpleItemWidget {
    text: String,
    selected: bool,
}

impl WidgetRef for SimpleItemWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let theme = Theme::default();
        let mut para = Paragraph::new(self.text.clone());
        if self.selected {
            para = para.style(theme.selected_style());
        }

        para.render(area, buf);
    }
}
