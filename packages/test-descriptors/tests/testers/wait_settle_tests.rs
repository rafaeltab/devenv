//! 1.2 Wait/Settle Tests
//!
//! Tests for wait and settle operations that control timing
//! during TUI interactions.

use std::time::Instant;
use test_descriptors::testers::{Command, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// Verify `wait_ms(100)` delays at least 100ms.
#[test]
fn wait_ms_delays_execution() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["hello"]);
    let mut asserter = env.testers().pty().run(&cmd);

    let start = Instant::now();
    asserter.wait_ms(100);
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() >= 100, "Should delay at least 100ms");
}

/// Echo command should settle quickly.
#[test]
fn wait_for_settle_detects_stable_screen() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["stable output"]);
    let mut asserter = env.testers().pty().run(&cmd);

    // Should complete without timing out
    asserter.wait_for_settle();
    asserter.find_text("stable output").assert_visible();
}

/// Custom settle timeout works.
#[test]
fn wait_for_settle_with_custom_timeout() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["test"]);
    let mut asserter = env.testers().pty().run(&cmd);

    // Use custom timeout (50ms settle, 1000ms max)
    asserter.wait_for_settle_ms(50, 1000);
    asserter.find_text("test").assert_visible();
}

/// Continuous output respects max_wait.
#[test]
fn wait_for_settle_max_wait_timeout() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Command that produces continuous output
    let cmd = Command::new("sh").args(&["-c", "while true; do echo tick; sleep 0.01; done"]);
    let mut asserter = env.testers().pty().run(&cmd);

    let start = Instant::now();
    // Should timeout at max_wait (200ms) since screen never settles
    asserter.wait_for_settle_ms(50, 200);
    let elapsed = start.elapsed();

    // Should have respected max_wait
    assert!(
        elapsed.as_millis() >= 200 && elapsed.as_millis() < 500,
        "Should timeout around max_wait time"
    );
}
