/// Key modifier enum for modified key combinations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Super,
}

/// Key enum representing keyboard inputs for TUI testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Tab,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Ctrl(char),
    Alt(char),
    Shift(char),
    F(u8),
    Delete,
    Insert,
    /// A key with a modifier applied.
    Modified {
        key: Box<Key>,
        modifier: Modifier,
    },
}

impl Key {
    /// Apply a modifier to this key.
    pub fn with_modifier(self, modifier: Modifier) -> Self {
        Key::Modified {
            key: Box::new(self),
            modifier,
        }
    }
}
