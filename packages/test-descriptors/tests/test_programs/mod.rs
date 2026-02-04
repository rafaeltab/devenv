//! Test helper programs for TUI testing.
//!
//! These programs are compiled at test time and used to verify
//! terminal behavior, input handling, and screen rendering.

use std::path::PathBuf;
use std::process::Command;

/// Compiles a test program and returns the path to the executable.
pub fn compile_test_program(name: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = manifest_dir
        .join("tests")
        .join("test_programs")
        .join(format!("{}.rs", name));

    let target_dir = std::env::temp_dir().join("test_programs");
    std::fs::create_dir_all(&target_dir).expect("Failed to create target dir");

    let output_path = target_dir.join(name);

    let output = Command::new("rustc")
        .arg(&src_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to compile test program");

    if !output.status.success() {
        panic!(
            "Failed to compile {}: {}",
            name,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    output_path
}

/// Echo program - prompts for input, echoes back with greeting.
pub mod echo_program {
    pub const SOURCE: &str = r#"
use std::io::{self, Write, BufRead};

fn main() {
    print!("Enter your name: ");
    io::stdout().flush().unwrap();
    
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    
    println!("Hello, {}!", line.trim());
}
"#;
}

/// Menu program - interactive menu with arrow key navigation.
pub mod menu_program {
    pub const SOURCE: &str = r#"
use std::io::{self, Read, Write};

fn main() {
    let items = ["Option 1", "Option 2", "Option 3"];
    let mut selected = 0;
    
    // Enable raw mode manually (simplified)
    print!("\x1b[?25l"); // Hide cursor
    
    loop {
        // Clear screen and draw menu
        print!("\x1b[2J\x1b[H");
        println!("Select an option:\n");
        
        for (i, item) in items.iter().enumerate() {
            if i == selected {
                println!("> {}", item);
            } else {
                println!("  {}", item);
            }
        }
        
        io::stdout().flush().unwrap();
        
        // Read input
        let mut buf = [0u8; 3];
        let n = io::stdin().read(&mut buf).unwrap();
        
        if n == 1 {
            match buf[0] {
                b'q' => break,
                b'\r' | b'\n' => {
                    print!("\x1b[?25h"); // Show cursor
                    println!("\nSelected: {}", items[selected]);
                    break;
                }
                _ => {}
            }
        } else if n == 3 && buf[0] == 0x1b && buf[1] == b'[' {
            match buf[2] {
                b'A' => selected = selected.saturating_sub(1), // Up
                b'B' => selected = (selected + 1).min(items.len() - 1), // Down
                _ => {}
            }
        }
    }
}
"#;
}

/// Key detector - detects and prints key combinations.
pub mod key_detector {
    pub const SOURCE: &str = r#"
use std::io::{self, Read, Write};

fn main() {
    println!("Key Detector - Press keys to see their codes (Ctrl+C to exit)");
    io::stdout().flush().unwrap();
    
    let mut buf = [0u8; 16];
    loop {
        let n = io::stdin().read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        
        // Check for Ctrl+C
        if n == 1 && buf[0] == 3 {
            println!("\nExiting...");
            break;
        }
        
        // Detect key combinations
        let key_name = match &buf[..n] {
            [0x1b, b'a'] => "Alt+A",
            [0x1b, b'[', b'1', b';', b'2', b'A'] => "Shift+Up",
            [0x1b, b'[', b'1', b';', b'6', b'R'] => "Ctrl+Shift+R",
            [0x1b, b'[', b'A'] => "Up",
            [0x1b, b'[', b'B'] => "Down",
            [0x1b, b'[', b'C'] => "Right",
            [0x1b, b'[', b'D'] => "Left",
            [0x1b] => "Escape",
            [b'\r'] | [b'\n'] => "Enter",
            [b'\t'] => "Tab",
            [0x7f] => "Backspace",
            _ => "Unknown",
        };
        
        print!("Key: {} (bytes: {:?})\r\n", key_name, &buf[..n]);
        io::stdout().flush().unwrap();
    }
}
"#;
}

/// Colored program - outputs text in various colors.
pub mod colored_program {
    pub const SOURCE: &str = r#"
fn main() {
    // Standard colors
    println!("\x1b[31mRed text\x1b[0m");
    println!("\x1b[32mGreen text\x1b[0m");
    println!("\x1b[33mYellow text\x1b[0m");
    println!("\x1b[34mBlue text\x1b[0m");
    println!("\x1b[35mMagenta text\x1b[0m");
    println!("\x1b[36mCyan text\x1b[0m");
    println!("\x1b[90mGray text\x1b[0m");
    
    // 256 colors
    println!("\x1b[38;5;208mOrange 256-color\x1b[0m");
    
    // True color RGB
    println!("\x1b[38;2;255;128;64mRGB Orange\x1b[0m");
    
    // Background colors
    println!("\x1b[41mRed background\x1b[0m");
    println!("\x1b[48;2;64;128;255mRGB Blue background\x1b[0m");
}
"#;
}

/// Multiscreen program - multi-screen navigation with clear screen between.
pub mod multiscreen_program {
    pub const SOURCE: &str = r#"
use std::io::{self, Read, Write};

fn main() {
    let screens = [
        "Screen 1: Welcome!\nPress Enter to continue...",
        "Screen 2: Configuration\nPress Enter to continue...",
        "Screen 3: Complete!\nPress Enter to exit...",
    ];
    
    for (i, screen) in screens.iter().enumerate() {
        // Clear screen
        print!("\x1b[2J\x1b[H");
        println!("=== Page {}/{} ===\n", i + 1, screens.len());
        println!("{}", screen);
        io::stdout().flush().unwrap();
        
        // Wait for Enter
        let mut buf = [0u8; 1];
        loop {
            io::stdin().read(&mut buf).unwrap();
            if buf[0] == b'\r' || buf[0] == b'\n' {
                break;
            }
        }
    }
    
    print!("\x1b[2J\x1b[H");
    println!("Done!");
}
"#;
}

/// Clear screen test program.
pub mod clear_screen_test {
    pub const SOURCE: &str = r#"
fn main() {
    println!("Before clear");
    print!("\x1b[2J"); // Clear screen
    println!("After clear");
}
"#;
}

/// Cursor position test program.
pub mod cursor_position_test {
    pub const SOURCE: &str = r#"
fn main() {
    print!("\x1b[2J\x1b[H"); // Clear and home
    print!("\x1b[5;10H"); // Move to row 5, col 10
    print!("X");
    print!("\x1b[1;1H"); // Move back to top
    println!();
}
"#;
}

/// Erase line test program.
pub mod erase_line_test {
    pub const SOURCE: &str = r#"
use std::io::{self, Write};

fn main() {
    print!("AAAAAAAAAA");
    print!("\x1b[5G"); // Move to column 5
    print!("\x1b[K"); // Erase to end of line
    println!();
    io::stdout().flush().unwrap();
}
"#;
}

/// Cursor save/restore test program.
pub mod cursor_save_restore_test {
    pub const SOURCE: &str = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H"); // Clear
    print!("Start");
    print!("\x1b[s"); // Save cursor
    print!("\x1b[10;10H"); // Move away
    print!("Away");
    print!("\x1b[u"); // Restore cursor
    print!("Back");
    println!();
    io::stdout().flush().unwrap();
}
"#;
}

/// Cursor movement test program.
pub mod cursor_movement_test {
    pub const SOURCE: &str = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H"); // Clear
    print!("\x1b[5;5H"); // Start at 5,5
    print!("O"); // Origin
    print!("\x1b[A"); // Up
    print!("U"); // Up marker
    print!("\x1b[2B"); // Down 2
    print!("D"); // Down marker
    print!("\x1b[C"); // Right
    print!("R"); // Right marker
    print!("\x1b[2D"); // Left 2
    print!("L"); // Left marker
    println!();
    io::stdout().flush().unwrap();
}
"#;
}

/// Scrolling region test program.
pub mod scrolling_region_test {
    pub const SOURCE: &str = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H"); // Clear
    println!("Line 1 - Header");
    println!("Line 2");
    println!("Line 3");
    println!("Line 4");
    println!("Line 5 - Footer");
    
    // Set scroll region to lines 2-4
    print!("\x1b[2;4r");
    print!("\x1b[4;1H"); // Go to line 4
    println!("New scroll line");
    
    // Reset scroll region
    print!("\x1b[r");
    io::stdout().flush().unwrap();
}
"#;
}

/// Insert characters test program.
pub mod insert_chars_test {
    pub const SOURCE: &str = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H"); // Clear
    print!("ABCDEF");
    print!("\x1b[1;3H"); // Column 3
    print!("\x1b[2@"); // Insert 2 spaces
    print!("XX");
    println!();
    io::stdout().flush().unwrap();
}
"#;
}

/// Insert lines test program.
pub mod insert_lines_test {
    pub const SOURCE: &str = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H"); // Clear
    println!("Line 1");
    println!("Line 2");
    println!("Line 3");
    print!("\x1b[2;1H"); // Go to line 2
    print!("\x1b[L"); // Insert line
    print!("Inserted");
    println!();
    io::stdout().flush().unwrap();
}
"#;
}

/// Overwrite test program.
pub mod overwrite_test {
    pub const SOURCE: &str = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H"); // Clear
    print!("AAAAA");
    print!("\x1b[1;2H"); // Column 2
    print!("BB");
    println!();
    io::stdout().flush().unwrap();
}
"#;
}

/// Tab stops test program.
pub mod tab_stops_test {
    pub const SOURCE: &str = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H"); // Clear
    print!("A\tB\tC");
    println!();
    io::stdout().flush().unwrap();
}
"#;
}

/// Reverse video test program.
pub mod reverse_video_test {
    pub const SOURCE: &str = r#"
fn main() {
    println!("\x1b[7mReverse video\x1b[0m Normal");
}
"#;
}

/// 256 color test program.
pub mod color_256_test {
    pub const SOURCE: &str = r#"
fn main() {
    // Test specific 256 colors
    println!("\x1b[38;5;196mColor 196 (bright red)\x1b[0m");
    println!("\x1b[38;5;46mColor 46 (bright green)\x1b[0m");
    println!("\x1b[38;5;21mColor 21 (bright blue)\x1b[0m");
    println!("\x1b[38;5;226mColor 226 (bright yellow)\x1b[0m");
    println!("\x1b[38;5;244mColor 244 (gray)\x1b[0m");
}
"#;
}

/// RGB color test program.
pub mod rgb_color_test {
    pub const SOURCE: &str = r#"
fn main() {
    // Test RGB true colors
    println!("\x1b[38;2;255;0;0mPure Red\x1b[0m");
    println!("\x1b[38;2;0;255;0mPure Green\x1b[0m");
    println!("\x1b[38;2;0;0;255mPure Blue\x1b[0m");
    println!("\x1b[38;2;255;255;0mPure Yellow\x1b[0m");
    println!("\x1b[38;2;128;128;128mGray\x1b[0m");
    
    // Background RGB
    println!("\x1b[48;2;64;64;64m\x1b[38;2;255;255;255mWhite on dark gray\x1b[0m");
}
"#;
}
