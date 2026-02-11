use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph, Widget, WidgetRef};
use sublime_fuzzy::{FuzzySearch, Scoring};

use crate::tui::picker_item::PickerItem;

/// A picker for selecting items from a list with fuzzy search.
///
/// This picker displays a list of items that can be filtered using
/// fuzzy search. Users can navigate with arrow keys and select with Enter.
pub struct SelectPicker<T: PickerItem> {
    items: Vec<T>,
    selected: usize,
    query: String,
    filtered_indices: Vec<(usize, isize)>, // (index, score) pairs sorted by score
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
            let mut matches: Vec<(usize, isize)> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let search_text = item.search_text();
                    let scoring = Scoring::default();
                    let result = FuzzySearch::new(&self.query, search_text)
                        .score_with(&scoring)
                        .best_match();

                    result.map(|m| (idx, m.score()))
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
        use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

        enable_raw_mode().ok()?;

        loop {
            // Create a copy of the data needed for rendering
            let query = self.query.clone();
            let selected = self.selected;
            let filtered_indices = self.filtered_indices.clone();

            // Draw the picker
            terminal
                .draw(|frame| {
                    let area = frame.area();
                    let max_visible = area.height as usize;

                    if filtered_indices.is_empty() {
                        // Show "No matches" message
                        let no_matches =
                            Paragraph::new("No matches").style(Style::default().fg(Color::Gray));
                        no_matches.render(area, frame.buffer_mut());
                        return;
                    }

                    for (display_idx, &(item_idx, _score)) in
                        filtered_indices.iter().enumerate().take(max_visible)
                    {
                        let is_selected = display_idx == selected;

                        // Calculate the area for this item
                        let item_area = Rect {
                            x: area.x,
                            y: area.y + display_idx as u16,
                            width: area.width,
                            height: 1,
                        };

                        if is_selected {
                            // Show that something is selected with yellow
                            let highlighted = Paragraph::new(format!("> Item {}", item_idx))
                                .style(Style::default().fg(Color::Yellow));
                            highlighted.render(item_area, frame.buffer_mut());
                        } else {
                            // Render unselected item normally
                            let normal = Paragraph::new(format!("  Item {}", item_idx));
                            normal.render(item_area, frame.buffer_mut());
                        }
                    }
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
                            disable_raw_mode().ok()?;
                            return Some(&self.items[idx]);
                        }
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
}

impl<T: PickerItem> Widget for SelectPicker<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Delegate to WidgetRef implementation
        self.render_ref(area, buf);
    }
}

impl<T: PickerItem> WidgetRef for SelectPicker<T> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let max_visible = area.height as usize;

        if self.filtered_indices.is_empty() {
            // Show "No matches" message
            let no_matches = Paragraph::new("No matches").style(Style::default().fg(Color::Gray));
            no_matches.render(area, buf);
            return;
        }

        for (display_idx, &(item_idx, _score)) in
            self.filtered_indices.iter().enumerate().take(max_visible)
        {
            let item = &self.items[item_idx];
            let is_selected = display_idx == self.selected;

            // Calculate the area for this item
            let item_area = Rect {
                x: area.x,
                y: area.y + display_idx as u16,
                width: area.width,
                height: 1,
            };

            if is_selected {
                // Highlight selected item with yellow
                let style = Style::default().fg(Color::Yellow);
                let highlighted = Paragraph::new(item.search_text()).style(style);
                highlighted.render(item_area, buf);
            } else {
                // Render unselected item normally
                item.render_ref(item_area, buf);
            }
        }
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

impl WidgetRef for SimpleItem {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.text.clone()).render(area, buf);
    }
}

impl PickerItem for SimpleItem {
    fn constraint(&self) -> ratatui::layout::Constraint {
        ratatui::layout::Constraint::Length(1)
    }

    fn search_text(&self) -> &str {
        &self.text
    }
}
