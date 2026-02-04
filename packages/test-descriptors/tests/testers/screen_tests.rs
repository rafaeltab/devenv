//! 1.6 Screen Tests
//!
//! Tests for screen buffer access and debugging.

use test_descriptors::testers::{Command, Key, Modifier, TuiAsserter};
use test_descriptors::TestEnvironment;

/// `screen()` contains output text.
#[test]
fn screen_returns_full_buffer() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["Hello Screen Test"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let screen = asserter.screen();
    assert!(screen.contains("Hello Screen Test"));
}

/// Screen changes after input.
#[test]
fn screen_reflects_current_state() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    // Initially empty or just prompt
    let screen_before = asserter.screen();

    asserter.type_text("new content");
    asserter.wait_for_settle();

    let screen_after = asserter.screen();

    // Screen should now contain the new content
    assert!(screen_after.contains("new content"));
    assert!(!screen_before.contains("new content"));

    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);
}

/// `dump_screen()` outputs to stderr.
#[test]
fn dump_screen_prints_to_stderr() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["Debug Output"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // This should print to stderr for debugging purposes
    // In a real test, we'd capture stderr, but for now just verify it doesn't panic
    asserter.dump_screen();
}
