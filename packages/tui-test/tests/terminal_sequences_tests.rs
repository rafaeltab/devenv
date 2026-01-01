// Tests for VT100/ANSI escape sequences that require proper terminal emulation
// These tests should FAIL with the simple terminal implementation
// and PASS when alacritty_terminal is properly integrated

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tui_test::{spawn_tui, ColorMatcher};

fn get_test_bin_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("test_programs");
    path
}

fn compile_test_program(source: &str, name: &str) -> PathBuf {
    let test_bin_dir = get_test_bin_dir();
    fs::create_dir_all(&test_bin_dir).expect("Failed to create test_programs directory");

    let source_path = test_bin_dir.join(format!("{}.rs", name));
    let binary_path = test_bin_dir.join(name);

    fs::write(&source_path, source).expect("Failed to write test program source");

    let status = Command::new("rustc")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .status()
        .expect("Failed to compile test program");

    assert!(status.success(), "Failed to compile test program {}", name);

    binary_path
}

#[test]
fn test_clear_screen_sequence() {
    // Test ESC[2J - Clear entire screen
    let source = r#"
use std::io::{self, Write};

fn main() {
    println!("First line of text");
    println!("Second line of text");
    println!("Third line of text");
    io::stdout().flush().unwrap();
    
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Clear screen
    print!("\x1b[2J\x1b[H");
    println!("Screen cleared!");
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "clear_screen_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // After clear, old text should not be visible
    tui.find_text("First line of text").assert_not_visible();
    tui.find_text("Second line of text").assert_not_visible();
    tui.find_text("Third line of text").assert_not_visible();

    // New text should be visible
    tui.find_text("Screen cleared!").assert_visible();

    tui.expect_completion();
}

#[test]
fn test_cursor_positioning() {
    // Test ESC[row;colH - Move cursor to specific position
    let source = r#"
use std::io::{self, Write};

fn main() {
    // Clear screen first
    print!("\x1b[2J\x1b[H");
    
    // Write at position (1, 1) - top left
    print!("\x1b[1;1HTop-Left");
    
    // Write at position (5, 10)
    print!("\x1b[5;10HMiddle");
    
    // Write at position (1, 20)
    print!("\x1b[1;20HTop-Right");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "cursor_position_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Verify text appears at correct positions
    let top_left = tui.find_text("Top-Left");
    assert_eq!(
        top_left.position(),
        Some((0, 0)),
        "Top-Left should be at (0, 0)"
    );

    let middle = tui.find_text("Middle");
    assert_eq!(
        middle.position(),
        Some((4, 9)),
        "Middle should be at (4, 9)"
    );

    let top_right = tui.find_text("Top-Right");
    assert_eq!(
        top_right.position(),
        Some((0, 19)),
        "Top-Right should be at (0, 19)"
    );

    tui.expect_completion();
}

#[test]
fn test_erase_in_line() {
    // Test ESC[K - Clear from cursor to end of line
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // Write a full line
    print!("This is a very long line of text");
    
    // Move cursor back and erase to end of line
    print!("\x1b[1;10H");  // Move to column 10
    print!("\x1b[K");       // Erase from cursor to end of line
    print!("SHORT");        // Write new text
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "erase_line_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should see beginning and new text, but not the erased portion
    tui.find_text("This is a").assert_visible();
    tui.find_text("SHORT").assert_visible();
    tui.find_text("very long line").assert_not_visible();

    tui.expect_completion();
}

#[test]
fn test_cursor_save_restore() {
    // Test ESC[s (save) and ESC[u (restore)
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("Start");
    print!("\x1b[s");           // Save cursor position
    
    print!("\x1b[5;5H");        // Move elsewhere
    print!("Middle");
    
    print!("\x1b[u");           // Restore cursor position
    print!("-End");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "cursor_save_restore_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // "Start-End" should appear together because cursor was restored
    tui.find_text("Start-End").assert_visible();
    tui.find_text("Middle").assert_visible();

    tui.expect_completion();
}

#[test]
fn test_cursor_movement() {
    // Test ESC[A (up), ESC[B (down), ESC[C (forward), ESC[D (backward)
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("Line 1\n");
    print!("Line 2\n");
    print!("Line 3\n");
    
    // Move up 2 lines
    print!("\x1b[2A");
    // Move forward 7 characters
    print!("\x1b[7C");
    print!("X");
    
    io::stdout().flush().unwrap();
    
    // Small delay to ensure output is transmitted
    std::thread::sleep(std::time::Duration::from_millis(50));
}
"#;

    let binary = compile_test_program(source, "cursor_movement_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    tui.find_text("Line 1").assert_visible();
    tui.find_text("Line 2").assert_visible();
    tui.find_text("Line 3").assert_visible();

    // The X should have been inserted after "Line 2" due to cursor movement
    // After printing 3 lines with \n, cursor is at line 4 column 0
    // Move up 2 lines -> line 2, column 0
    // Move forward 7 characters -> line 2, column 7 (after "Line 2 ")
    tui.find_text("Line 2 X").assert_visible();

    tui.expect_completion();
}

#[test]
fn test_scrolling_region() {
    // Test ESC[top;bottomr - Set scrolling region
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // Set scrolling region (lines 5-10)
    print!("\x1b[5;10r");
    
    // Fill the screen
    for i in 1..=15 {
        print!("\x1b[{};1H", i);
        print!("Line {}", i);
    }
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "scrolling_region_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Lines should be visible - use find_all_text to avoid ambiguity
    let line_1_matches = tui.find_all_text("Line 1");
    assert!(!line_1_matches.is_empty(), "Line 1 should be visible");

    let line_5_matches = tui.find_all_text("Line 5");
    assert!(!line_5_matches.is_empty(), "Line 5 should be visible");

    let line_10_matches = tui.find_all_text("Line 10");
    assert_eq!(
        line_10_matches.len(),
        1,
        "Line 10 should appear exactly once"
    );

    tui.expect_completion();
}

#[test]
fn test_insert_delete_characters() {
    // Test ESC[@n (insert n chars) and ESC[Pn (delete n chars)
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("ABCDEFGH");
    
    // Move to position and insert 3 characters
    print!("\x1b[1;4H");
    print!("\x1b[3@");
    print!("123");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "insert_chars_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // After inserting "123" at position 4, we should see "ABC123DEFGH"
    tui.find_text("ABC123").assert_visible();

    tui.expect_completion();
}

#[test]
fn test_insert_delete_lines() {
    // Test ESC[Ln (insert n lines) and ESC[Mn (delete n lines)
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    println!("Line A");
    println!("Line B");
    println!("Line C");
    
    // Go to line 2 and insert a line
    print!("\x1b[2;1H");
    print!("\x1b[L");
    print!("INSERTED");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "insert_lines_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // After insertion, order should be: Line A, INSERTED, Line B, Line C
    let line_a = tui.find_text("Line A");
    let inserted = tui.find_text("INSERTED");
    let line_b = tui.find_text("Line B");

    assert!(line_a.position().unwrap().0 < inserted.position().unwrap().0);
    assert!(inserted.position().unwrap().0 < line_b.position().unwrap().0);

    tui.expect_completion();
}

#[test]
fn test_overwrite_mode() {
    // Test that characters overwrite what's already there
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("XXXXXXXXXX");
    
    // Move back to start and overwrite
    print!("\x1b[1;1H");
    print!("HELLO");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "overwrite_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should see "HELLOXXXXX" (HELLO overwrites first 5 X's)
    tui.find_text("HELLOXXXXX").assert_visible();

    tui.expect_completion();
}

#[test]
fn test_tab_stops() {
    // Test that tabs move to proper positions (typically every 8 characters)
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    print!("A\tB\tC\tD");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "tab_stops_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Find positions of A, B, C, D - they should be at tab stops
    let a_pos = tui.find_text("A").position().unwrap();
    let b_pos = tui.find_text("B").position().unwrap();
    let c_pos = tui.find_text("C").position().unwrap();
    let d_pos = tui.find_text("D").position().unwrap();

    // All on same line
    assert_eq!(a_pos.0, b_pos.0);
    assert_eq!(b_pos.0, c_pos.0);
    assert_eq!(c_pos.0, d_pos.0);

    // B should be at column 8 (tab stop), C at 16, D at 24
    assert_eq!(b_pos.1, 8, "B should be at column 8");
    assert_eq!(c_pos.1, 16, "C should be at column 16");
    assert_eq!(d_pos.1, 24, "D should be at column 24");

    tui.expect_completion();
}

#[test]
fn test_reverse_video() {
    // Test ESC[7m - Reverse video (swap fg/bg)
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // Set foreground red, background blue
    print!("\x1b[31;44m");
    print!("Normal");
    
    // Reverse video
    print!("\x1b[7m");
    print!("Reversed");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "reverse_video_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Normal should have red fg, blue bg
    let normal = tui.find_text("Normal");
    normal.fg.assert(ColorMatcher::RedIsh);
    normal.bg.assert(ColorMatcher::BlueIsh);

    // Reversed should have blue fg, red bg (swapped)
    let reversed = tui.find_text("Reversed");
    reversed.fg.assert(ColorMatcher::BlueIsh);
    reversed.bg.assert(ColorMatcher::RedIsh);

    tui.expect_completion();
}

#[test]
fn test_256_color_support() {
    // Test ESC[38;5;nm - 256-color foreground
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // Use 256-color mode - color 196 is bright red
    print!("\x1b[38;5;196mBright Red\x1b[0m\n");
    
    // Color 46 is bright green
    print!("\x1b[38;5;46mBright Green\x1b[0m\n");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "color_256_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    tui.find_text("Bright Red").assert_visible();
    tui.find_text("Bright Green").assert_visible();

    // Colors should be detected (may need tolerance for exact RGB values)
    tui.find_text("Bright Red").fg.assert(ColorMatcher::RedIsh);
    tui.find_text("Bright Green")
        .fg
        .assert(ColorMatcher::GreenIsh);

    tui.expect_completion();
}

#[test]
fn test_rgb_color_support() {
    // Test ESC[38;2;r;g;bm - True color (24-bit RGB)
    let source = r#"
use std::io::{self, Write};

fn main() {
    print!("\x1b[2J\x1b[H");
    
    // True color: RGB(255, 100, 50) - orange-ish
    print!("\x1b[38;2;255;100;50mOrange Text\x1b[0m\n");
    
    // True color: RGB(100, 200, 255) - light blue
    print!("\x1b[38;2;100;200;255mLight Blue\x1b[0m\n");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "rgb_color_test");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    tui.find_text("Orange Text").assert_visible();
    tui.find_text("Light Blue").assert_visible();

    // Check exact RGB values
    tui.find_text("Orange Text").fg.exact(255, 100, 50);
    tui.find_text("Light Blue").fg.exact(100, 200, 255);

    tui.expect_completion();
}
