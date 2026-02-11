use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget, WidgetRef};
use ratatui::Frame as RatatuiFrame;

/// A three-panel layout frame for pickers.
///
/// The frame provides a consistent layout with:
/// - Top panel: Input/query display
/// - Middle panel: Scrollable item list
/// - Bottom panel: Status/help text
///
/// # Example
///
/// ```ignore
/// use ratatui::Frame;
/// use rafaeltab::tui::Frame as PickerFrame;
///
/// fn draw_picker(frame: &mut Frame) {
///     let picker_frame = PickerFrame::new("Select an item:", "Type to filter");
///     picker_frame.render(frame);
/// }
/// ```
pub struct Frame {
    /// Title/prompt displayed in the top panel
    pub title: String,
    /// Help text displayed in the bottom panel
    pub help_text: String,
    /// The current input/query text
    pub input: String,
    /// Style for the frame
    pub style: Style,
}

impl Frame {
    /// Create a new frame with the given title and help text.
    pub fn new(title: impl Into<String>, help_text: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            help_text: help_text.into(),
            input: String::new(),
            style: Style::default(),
        }
    }

    /// Set the current input text.
    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.input = input.into();
        self
    }

    /// Set the style for the frame.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Calculate the layout areas for the three panels.
    ///
    /// Returns a tuple of (top_area, middle_area, bottom_area).
    pub fn calculate_layout(&self, area: Rect) -> (Rect, Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Top: 1 line for title + 1 line for input
                Constraint::Min(5),    // Middle: minimum 5 lines for items
                Constraint::Length(1), // Bottom: 1 line for help
            ])
            .split(area);

        (chunks[0], chunks[1], chunks[2])
    }

    /// Render the frame structure (panels without content).
    ///
    /// This renders the borders and titles. Content should be rendered
    /// separately into the returned middle area.
    pub fn render(&self, frame: &mut RatatuiFrame, content_area: &mut Rect) {
        let (top_area, middle_area, bottom_area) = self.calculate_layout(frame.area());

        // Render top panel with title and input
        let top_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(self.style);

        let top_inner = top_block.inner(top_area);
        top_block.render(top_area, frame.buffer_mut());

        // Render title
        let title_paragraph = Paragraph::new(self.title.clone()).style(self.style);
        title_paragraph.render(top_inner, frame.buffer_mut());

        // Render input if present
        if !self.input.is_empty() {
            let input_paragraph = Paragraph::new(format!("> {}", self.input)).style(self.style);
            input_paragraph.render(
                Rect {
                    x: top_inner.x,
                    y: top_inner.y + 1,
                    width: top_inner.width,
                    height: 1,
                },
                frame.buffer_mut(),
            );
        }

        // Render middle panel (just borders, content will be rendered separately)
        let middle_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(self.style);
        *content_area = middle_block.inner(middle_area);
        middle_block.render(middle_area, frame.buffer_mut());

        // Render bottom panel with help text
        let help_paragraph =
            Paragraph::new(self.help_text.clone()).style(self.style.fg(Color::Gray));
        help_paragraph.render(bottom_area, frame.buffer_mut());
    }

    /// Render the frame with custom content in the middle panel.
    ///
    /// The content widget will be rendered in the middle panel area.
    pub fn render_with_content<W: WidgetRef>(&self, frame: &mut RatatuiFrame, content: &W) {
        let (top_area, middle_area, bottom_area) = self.calculate_layout(frame.area());

        // Render top panel with title and input
        let top_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(self.style);

        let top_inner = top_block.inner(top_area);
        top_block.render(top_area, frame.buffer_mut());

        // Render title
        let title_paragraph = Paragraph::new(self.title.clone()).style(self.style);
        title_paragraph.render(top_inner, frame.buffer_mut());

        // Render input if present
        if !self.input.is_empty() {
            let input_paragraph = Paragraph::new(format!("> {}", self.input)).style(self.style);
            input_paragraph.render(
                Rect {
                    x: top_inner.x,
                    y: top_inner.y + 1,
                    width: top_inner.width,
                    height: 1,
                },
                frame.buffer_mut(),
            );
        }

        // Render middle panel with content
        let middle_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(self.style);
        let middle_inner = middle_block.inner(middle_area);
        middle_block.render(middle_area, frame.buffer_mut());
        content.render_ref(middle_inner, frame.buffer_mut());

        // Render bottom panel with help text
        let help_paragraph =
            Paragraph::new(self.help_text.clone()).style(self.style.fg(Color::Gray));
        help_paragraph.render(bottom_area, frame.buffer_mut());
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new(
            "Picker",
            "Use arrows to navigate, Enter to select, Esc to cancel",
        )
    }
}
