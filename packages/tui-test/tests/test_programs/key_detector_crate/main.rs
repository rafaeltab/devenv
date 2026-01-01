// Test program that detects key combinations using crossterm
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use std::io::{self, Write};
use std::time::Duration;

fn main() {
    // Enable raw mode to get individual keypresses
    terminal::enable_raw_mode().expect("Failed to enable raw mode");

    // Signal ready
    print!("READY\r\n");
    io::stdout().flush().unwrap();

    // Wait for a key event with timeout
    if event::poll(Duration::from_secs(2)).unwrap() {
        if let Ok(Event::Key(key_event)) = event::read() {
            let result = detect_key_combination(key_event);
            print!("{}\r\n", result);
            io::stdout().flush().unwrap();
        }
    }

    // Disable raw mode before exit
    terminal::disable_raw_mode().expect("Failed to disable raw mode");
}

fn detect_key_combination(key: KeyEvent) -> &'static str {
    match (key.modifiers, key.code) {
        // Alt+A
        (m, KeyCode::Char('a'))
            if m.contains(KeyModifiers::ALT)
                && !m.contains(KeyModifiers::CONTROL)
                && !m.contains(KeyModifiers::SHIFT) =>
        {
            "ALT_A_DETECTED"
        }
        // Shift+Up
        (m, KeyCode::Up)
            if m.contains(KeyModifiers::SHIFT)
                && !m.contains(KeyModifiers::CONTROL)
                && !m.contains(KeyModifiers::ALT) =>
        {
            "SHIFT_UP_DETECTED"
        }
        // Ctrl+Shift+R
        (m, KeyCode::Char('r') | KeyCode::Char('R'))
            if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
        {
            "CTRL_SHIFT_R_DETECTED"
        }
        _ => "UNKNOWN_KEY",
    }
}
