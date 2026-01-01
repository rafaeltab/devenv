#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // Printable
    Char(char),

    // Special keys
    Enter,
    Esc,
    Tab,
    Backspace,

    // Navigation
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,

    // Modifiers (for combinations)
    Ctrl,
    Alt,
    Shift,
    Super,
}
