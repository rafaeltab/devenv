//! 1.4 Assertion Tests
//!
//! Tests for text visibility assertions.

use test_descriptors::testers::{Command, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// `assert_visible()` passes when text exists.
#[test]
fn assert_visible_succeeds() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["visible text"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Should not panic
    asserter.find_text("visible text").assert_visible();
}

/// `assert_visible()` panics with helpful message.
#[test]
#[should_panic(expected = "not found")]
fn assert_visible_fails_with_message() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["something else"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Should panic with helpful message
    asserter.find_text("invisible text").assert_visible();
}

/// `assert_not_visible()` passes when text absent.
#[test]
fn assert_not_visible_succeeds() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["hello"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Should not panic - "goodbye" is not visible
    asserter.find_text("goodbye").assert_not_visible();
}

/// `assert_not_visible()` panics when text present.
#[test]
#[should_panic(expected = "was found")]
fn assert_not_visible_fails() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["present text"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Should panic - text is visible
    asserter.find_text("present text").assert_not_visible();
}

/// Position returns (row, col) tuple.
#[test]
fn text_match_position_returns_coords() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["test"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("test");
    let pos = text_match.position();
    assert!(pos.is_some());

    let (row, col) = pos.unwrap();
    // Row and col should be valid coordinates (1000 since this is outside of the default pty size)
    assert!(row < 1000);
    assert!(col < 1000);
}
