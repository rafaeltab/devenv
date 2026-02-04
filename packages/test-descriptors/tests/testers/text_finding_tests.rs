//! 1.3 Text Finding Tests
//!
//! Tests for text search functionality in the TUI buffer.

use test_descriptors::testers::{Command, TuiAsserter};
use test_descriptors::TestEnvironment;

/// Find "Hello" in "Hello World", position is Some.
#[test]
fn find_text_returns_position() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["Hello World"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Hello");
    assert!(text_match.position().is_some());
}

/// Find "Goodbye" in "Hello", position is None.
#[test]
fn find_text_not_found_returns_none() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["Hello"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Goodbye");
    assert!(text_match.position().is_none());
}

/// Find "test" when it appears twice, should panic.
#[test]
#[should_panic(expected = "multiple")]
fn find_text_panics_on_multiple_matches() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["test one test two"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Should panic because "test" appears twice
    let _ = asserter.find_text("test");
}

/// Find all "test" when it appears twice.
#[test]
fn find_all_text_returns_all_positions() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["test one test two"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let matches = asserter.find_all_text("test");
    assert_eq!(matches.len(), 2);
}

/// Find all "Goodbye" in "Hello", returns empty vec.
#[test]
fn find_all_text_empty_when_not_found() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["Hello"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let matches = asserter.find_all_text("Goodbye");
    assert!(matches.is_empty());
}

/// Finds exact substrings.
#[test]
fn find_text_exact_match_only() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["testing"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // "test" is a substring of "testing"
    let text_match = asserter.find_text("test");
    assert!(text_match.position().is_some());

    // "tested" is not a substring
    let text_match = asserter.find_text("tested");
    assert!(text_match.position().is_none());
}

/// "Hello" != "hello" (case sensitive).
#[test]
fn find_text_case_sensitive() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["Hello"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Exact case match
    let text_match = asserter.find_text("Hello");
    assert!(text_match.position().is_some());

    // Different case should not match
    let text_match = asserter.find_text("hello");
    assert!(text_match.position().is_none());
}
