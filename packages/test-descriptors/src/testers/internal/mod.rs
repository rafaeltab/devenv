// Internal shared implementations for testers.

mod key_conversion;
mod pty_backend;
mod terminal_buffer;

pub(crate) use key_conversion::KeyConversion;
#[allow(unused_imports)]
pub(crate) use pty_backend::PtyBackend;

// Use conditional compilation to select terminal implementation
#[cfg(not(feature = "use-wezterm-term"))]
pub(crate) use terminal_buffer::{Cell, TerminalBuffer};

#[cfg(feature = "use-wezterm-term")]
pub(crate) use wezterm_terminal::{Cell, TerminalBuffer};

pub(crate) mod terminal_buffer_debug;

// New module using wezterm-term - exists alongside old one during transition
#[cfg(feature = "use-wezterm-term")]
pub(crate) mod wezterm_terminal;

#[cfg(test)]
mod terminal_buffer_tests;

// Contract tests to ensure parity between implementations
#[cfg(all(test, feature = "use-wezterm-term"))]
mod terminal_comparison_tests;
