use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::WidgetRef;

/// A picker for selecting items from a list with fuzzy search.
///
/// This picker displays a list of items that can be filtered using
/// fuzzy search. Users can navigate with arrow keys and select with Enter.
pub struct SelectPicker<T: PickerItem> {
    items: Vec<T>,
    selected: usize,
    query: String,
    filtered_indices: Vec<usize>,
}

impl<T: PickerItem> SelectPicker<T> {
    /// Create a new select picker with the given items and prompt.
    pub fn new(items: Vec<T>) -> Self {
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
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
        self
    }
}

impl<T: PickerItem> WidgetRef for SelectPicker<T> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        // Placeholder implementation
        // Full implementation in Phase 4
        for (i, &idx) in self.filtered_indices.iter().enumerate() {
            if i >= area.height as usize {
                break;
            }
            let item = &self.items[idx];
            item.render_ref(
                Rect {
                    x: area.x,
                    y: area.y + i as u16,
                    width: area.width,
                    height: 1,
                },
                buf,
            );
        }
    }
}

use crate::tui::picker_item::PickerItem;
