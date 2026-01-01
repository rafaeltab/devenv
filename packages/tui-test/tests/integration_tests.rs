use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tui_test::{spawn_tui, ColorMatcher, Key};

// Helper to get the directory where test binaries should be located
fn get_test_bin_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("test_programs");
    path
}

// Helper to compile a test program
fn compile_test_program(source: &str, name: &str) -> PathBuf {
    let test_bin_dir = get_test_bin_dir();
    fs::create_dir_all(&test_bin_dir).expect("Failed to create test_programs directory");

    let source_path = test_bin_dir.join(format!("{}.rs", name));
    let binary_path = test_bin_dir.join(name);

    // Write source file
    fs::write(&source_path, source).expect("Failed to write test program source");

    // Compile with rustc
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
fn interactive_menu_navigation() {
    // Create a simple menu program using line-buffered input
    let source = r#"
use std::io::{self, Write, BufRead};

fn main() {
    let items = vec!["Option A", "Option B", "Option C"];
    let mut selected = 0;

    // Initial display
    print!("\x1b[2J\x1b[H");
    println!("Select an option (type number 0-2, then press Enter):\n");
    
    for (i, item) in items.iter().enumerate() {
        if i == selected {
            println!("\x1b[33m> {}\x1b[0m", item);
        } else {
            println!("  {}", item);
        }
    }
    
    println!("\nCurrent selection: {}", selected);
    io::stdout().flush().unwrap();

    // Read selection
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    
    if let Ok(choice) = line.trim().parse::<usize>() {
        if choice < items.len() {
            selected = choice;
        }
    }
    
    // Show final selection
    print!("\x1b[2J\x1b[H");
    println!("You selected: {}", items[selected]);
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "menu_program");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn menu program");

    tui.wait_for_settle();

    // Verify initial state
    tui.find_text("Select an option").assert_visible();
    tui.find_text("Option A").assert_visible();
    tui.find_text("Option B").assert_visible();
    tui.find_text("Option C").assert_visible();

    // Verify first item is selected (should be yellow)
    tui.find_text("> Option A")
        .fg
        .assert(ColorMatcher::YellowIsh);

    // Verify current selection indicator
    tui.find_text("Current selection: 0").assert_visible();

    // Select option B (index 1)
    tui.type_text("1");
    tui.press_key(Key::Enter);
    tui.wait_for_settle();

    // Verify selection message
    tui.find_text("You selected: Option B").assert_visible();

    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn text_input_echo() {
    // Create a text input program that echoes back
    let source = r#"
use std::io::{self, Write, BufRead};

fn main() {
    println!("Enter your name:");
    io::stdout().flush().unwrap();
    
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    
    let name = line.trim();
    println!("Hello, {}!", name);
    println!("Your name has {} characters.", name.len());
}
"#;

    let binary = compile_test_program(source, "echo_program");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn echo program");

    tui.wait_for_settle();

    // Verify prompt
    tui.find_text("Enter your name:").assert_visible();

    // Type name
    tui.type_text("Alice");
    tui.wait_for_settle();

    // Verify typed text appears
    tui.find_text("Alice").assert_visible();

    // Press enter
    tui.press_key(Key::Enter);
    tui.wait_for_settle();

    // Verify echo
    tui.find_text("Hello, Alice!").assert_visible();
    tui.find_text("Your name has 5 characters.")
        .assert_visible();

    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn colored_output_detection() {
    // Create a program with colored output
    let source = r#"
use std::io::{self, Write};

fn main() {
    // Red text
    println!("\x1b[31mError: Something went wrong\x1b[0m");
    
    // Green text
    println!("\x1b[32mSuccess: Operation completed\x1b[0m");
    
    // Blue text
    println!("\x1b[34mInfo: Processing data\x1b[0m");
    
    // Yellow text
    println!("\x1b[33mWarning: Be careful\x1b[0m");
    
    // Cyan text
    println!("\x1b[36mDebug: Variable value = 42\x1b[0m");
    
    // Magenta text
    println!("\x1b[35mNote: Remember this\x1b[0m");
    
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "colored_program");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn colored program");

    tui.wait_for_settle();

    // Verify all text is visible
    tui.find_text("Error: Something went wrong")
        .assert_visible();
    tui.find_text("Success: Operation completed")
        .assert_visible();
    tui.find_text("Info: Processing data").assert_visible();
    tui.find_text("Warning: Be careful").assert_visible();
    tui.find_text("Debug: Variable value = 42").assert_visible();
    tui.find_text("Note: Remember this").assert_visible();

    // Verify colors
    tui.find_text("Error: Something went wrong")
        .fg
        .assert(ColorMatcher::RedIsh);
    tui.find_text("Success: Operation completed")
        .fg
        .assert(ColorMatcher::GreenIsh);
    tui.find_text("Info: Processing data")
        .fg
        .assert(ColorMatcher::BlueIsh);
    tui.find_text("Warning: Be careful")
        .fg
        .assert(ColorMatcher::YellowIsh);
    tui.find_text("Debug: Variable value = 42")
        .fg
        .assert(ColorMatcher::CyanIsh);
    tui.find_text("Note: Remember this")
        .fg
        .assert(ColorMatcher::MagentaIsh);

    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn multi_screen_interaction() {
    // Create a multi-screen program
    let source = r#"
use std::io::{self, Write, BufRead};

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().unwrap();
}

fn main() {
    // Screen 1: Welcome
    clear_screen();
    println!("Welcome to Multi-Screen App");
    println!("\nPress Enter to continue...");
    io::stdout().flush().unwrap();
    
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    
    // Screen 2: Input
    clear_screen();
    println!("Please enter a number (1-3):");
    io::stdout().flush().unwrap();
    
    line.clear();
    stdin.lock().read_line(&mut line).unwrap();
    let choice: u8 = line.trim().parse().unwrap_or(1);
    
    // Screen 3: Result
    clear_screen();
    println!("You chose option {}", choice);
    
    match choice {
        1 => println!("\x1b[32mGood choice!\x1b[0m"),
        2 => println!("\x1b[33mInteresting choice!\x1b[0m"),
        3 => println!("\x1b[34mBold choice!\x1b[0m"),
        _ => {}
    }
    
    println!("\nThank you for using this app!");
    io::stdout().flush().unwrap();
}
"#;

    let binary = compile_test_program(source, "multiscreen_program");

    let mut tui = spawn_tui(binary.to_str().unwrap(), &[])
        .settle_timeout(200)
        .spawn()
        .expect("Failed to spawn multiscreen program");

    tui.wait_for_settle();

    // Screen 1: Welcome
    tui.find_text("Welcome to Multi-Screen App")
        .assert_visible();
    tui.find_text("Press Enter to continue...").assert_visible();

    // Progress to screen 2
    tui.press_key(Key::Enter);
    tui.wait_for_settle();

    // Screen 2: Input
    tui.find_text("Please enter a number (1-3):")
        .assert_visible();
    // Welcome text should no longer be visible (screen cleared)
    tui.find_text("Welcome to Multi-Screen App")
        .assert_not_visible();

    // Enter choice
    tui.type_text("2");
    tui.press_key(Key::Enter);
    tui.wait_for_settle();

    // Screen 3: Result
    tui.find_text("You chose option 2").assert_visible();
    tui.find_text("Interesting choice!").assert_visible();
    tui.find_text("Thank you for using this app!")
        .assert_visible();

    // Verify color of choice message
    tui.find_text("Interesting choice!")
        .fg
        .assert(ColorMatcher::YellowIsh);

    // Input prompt should no longer be visible
    tui.find_text("Please enter a number (1-3):")
        .assert_not_visible();

    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}
