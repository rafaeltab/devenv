use ratatui::layout::Constraint;
use ratatui::widgets::WidgetRef;

/// Trait for items that can be displayed and selected in a picker.
///
/// This trait combines rendering capabilities (via `WidgetRef`) with
/// search and layout functionality needed by pickers.
///
/// # Example
///
/// ```ignore
/// use ratatui::layout::Constraint;
/// use ratatui::widgets::{Paragraph, WidgetRef};
/// use ratatui::layout::Rect;
/// use ratatui::buffer::Buffer;
///
/// struct SimpleItem {
///     text: String,
/// }
///
/// impl WidgetRef for SimpleItem {
///     fn render_ref(&self, area: Rect, buf: &mut Buffer) {
///         Paragraph::new(self.text.clone()).render(area, buf);
///     }
/// }
///
/// impl PickerItem for SimpleItem {
///     fn constraint(&self) -> Constraint {
///         Constraint::Length(1)
///     }
///
///     fn search_text(&self) -> &str {
///         &self.text
///     }
/// }
/// ```
pub trait PickerItem: Clone {
    /// Returns the layout constraint for this item.
    ///
    /// This determines how much space the item takes in the picker list.
    fn constraint(&self) -> Constraint;

    /// Returns the text used for fuzzy searching.
    ///
    /// This text will be matched against user input when filtering items.
    fn search_text(&self) -> &str;

    fn render(&self, selected: bool) -> Box<dyn WidgetRef>;
}
