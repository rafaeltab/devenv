use super::color::ColorAssertion;

/// Represents a text match found in TUI output, with color information.
#[derive(Debug)]
pub struct TextMatch {
    text: String,
    row: Option<u16>,
    col: Option<u16>,
    found: bool,
    /// Foreground color assertion.
    pub fg: ColorAssertion,
    /// Background color assertion.
    pub bg: ColorAssertion,
}

impl TextMatch {
    pub(crate) fn new(
        text: String,
        row: Option<u16>,
        col: Option<u16>,
        found: bool,
        fg: ColorAssertion,
        bg: ColorAssertion,
    ) -> Self {
        Self {
            text,
            row,
            col,
            found,
            fg,
            bg,
        }
    }

    pub(crate) fn not_found(text: &str) -> Self {
        Self {
            text: text.to_string(),
            row: None,
            col: None,
            found: false,
            fg: ColorAssertion::not_found(),
            bg: ColorAssertion::not_found(),
        }
    }

    /// Returns the position (row, col) of the matched text, if found.
    pub fn position(&self) -> Option<(u16, u16)> {
        if self.found {
            Some((self.row?, self.col?))
        } else {
            None
        }
    }

    /// Returns true if the text was found on the screen.
    pub fn is_visible(&self) -> bool {
        self.found
    }

    /// Assert that the text is visible in the TUI output.
    pub fn assert_visible(&self) {
        assert!(
            self.found,
            "Expected text '{}' to be visible, but it was not found",
            self.text
        );
    }

    /// Assert that the text is not visible in the TUI output.
    pub fn assert_not_visible(&self) {
        assert!(
            !self.found,
            "Expected text '{}' to not be visible, but it was found at ({:?}, {:?})",
            self.text, self.row, self.col
        );
    }
}
