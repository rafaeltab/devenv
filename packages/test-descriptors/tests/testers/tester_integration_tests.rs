//! 1.9 Integration Tests
//!
//! End-to-end integration tests for TUI interactions.

use test_descriptors::testers::{ColorMatcher, Command, Key, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// Multi-step menu interaction.
#[test]
fn interactive_menu_navigation() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Use a simple menu simulation with fzf-style interface
    let cmd = Command::new("sh").args(&[
        "-c",
        r#"
        echo "Option 1"
        echo "Option 2"
        echo "Option 3"
        echo "Select with arrows, Enter to confirm:"
        read selection
        echo "Selected: $selection"
        "#,
    ]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Verify menu options are visible
    asserter.find_text("Option 1").assert_visible();
    asserter.find_text("Option 2").assert_visible();
    asserter.find_text("Option 3").assert_visible();

    // Type selection and press Enter
    asserter.type_text("2");
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    asserter.find_text("Selected: 2").assert_visible();
}

/// Type name, verify greeting.
#[test]
fn text_input_echo() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh").args(&[
        "-c",
        r#"
        echo "Enter your name:"
        read name
        echo "Hello, $name!"
        "#,
    ]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    asserter.find_text("Enter your name").assert_visible();

    asserter.type_text("World");
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    asserter.find_text("Hello, World!").assert_visible();
}

/// Multiple colors in output.
#[test]
fn colored_output_detection() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf")
        .args(&["\x1b[31mError\x1b[0m: \x1b[33mWarning\x1b[0m - \x1b[32mSuccess\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Verify all text is visible
    asserter.find_text("Error").assert_visible();
    asserter.find_text("Warning").assert_visible();
    asserter.find_text("Success").assert_visible();

    // Verify colors
    asserter
        .find_text("Error")
        .fg
        .assert_matches(ColorMatcher::RedIsh);
    asserter
        .find_text("Warning")
        .fg
        .assert_matches(ColorMatcher::YellowIsh);
    asserter
        .find_text("Success")
        .fg
        .assert_matches(ColorMatcher::GreenIsh);
}

/// Navigate through multiple screens.
#[test]
fn multi_screen_interaction() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh").args(&[
        "-c",
        r#"
        clear
        echo "Screen 1: Welcome"
        echo "Press Enter to continue..."
        read
        clear
        echo "Screen 2: Setup"
        echo "Press Enter to continue..."
        read
        clear
        echo "Screen 3: Done!"
        "#,
    ]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Screen 1
    asserter.find_text("Screen 1: Welcome").assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Screen 2
    asserter.find_text("Screen 1: Welcome").assert_not_visible();
    asserter.find_text("Screen 2: Setup").assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Screen 3
    asserter.find_text("Screen 2: Setup").assert_not_visible();
    asserter.find_text("Screen 3: Done!").assert_visible();
}
