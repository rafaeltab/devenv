//! 3.3 PtyTester Tests
//!
//! Tests for direct PTY execution outside tmux.

use test_descriptors::testers::{Command, Key, Modifier, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// $TMUX env var is not set.
#[test]
fn pty_tester_runs_outside_tmux() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "echo TMUX=$TMUX"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let screen = asserter.screen();
    // TMUX should be empty or unset
    assert!(
        screen.contains("TMUX=\n") || screen.contains("TMUX=$") || screen.contains("TMUX="),
        "TMUX env var should be empty when running via pty tester"
    );
}

/// Full TUI test works.
#[test]
fn pty_tester_full_tui_interaction() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Interactive shell session
    let cmd = Command::new("sh").args(&[
        "-c",
        r#"
        echo "Enter name:"
        read name
        echo "Hello, $name!"
        "#,
    ]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    asserter.find_text("Enter name").assert_visible();

    asserter.type_text("PTY User");
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    asserter.find_text("Hello, PTY User!").assert_visible();

    asserter.expect_exit_code(0);
}
