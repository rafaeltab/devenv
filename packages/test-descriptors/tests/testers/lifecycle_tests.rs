//! 1.1 Lifecycle Tests
//!
//! Tests for TuiAsserter lifecycle operations including spawning commands,
//! environment configuration, terminal sizing, and exit code handling.

use test_descriptors::testers::{Command, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// Spawn a simple command (echo) and verify it completes with exit code 0.
#[test]
fn spawn_simple_command() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["hello", "world"]);
    let mut asserter = env.testers().pty().run(&cmd);

    asserter.wait_for_settle();
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0);
}

/// Spawn with custom environment variable, verify it's accessible.
#[test]
fn spawn_with_custom_env() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh")
        .args(&["-c", "echo $MY_TEST_VAR"])
        .env("MY_TEST_VAR", "custom_value");

    let mut asserter = env.testers().pty().run(&cmd);

    asserter.wait_for_settle();
    asserter.find_text("custom_value").assert_visible();
}

/// Set custom terminal size, verify via `tput cols`.
#[test]
fn spawn_with_custom_terminal_size() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("tput").args(&["cols"]).pty_size(40, 120);

    let mut asserter = env.testers().pty().run(&cmd);

    asserter.wait_for_settle();
    asserter.find_text("120").assert_visible();
}

/// Set working directory, verify via `pwd`.
#[test]
fn spawn_with_custom_cwd() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("my-workspace", |_d| {});
        });
    })
    .create();

    let dir_ref = env.find_dir("my-workspace").unwrap();
    let workspace_path = dir_ref.path();
    let cmd = Command::new("pwd").cwd(workspace_path);

    let mut asserter = env.testers().pty().run(&cmd);

    asserter.wait_for_settle();
    asserter.find_text("my-workspace").assert_visible();
}

/// Run `exit 42`, verify exit code is 42.
#[test]
fn expect_completion_returns_exit_code() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "exit 42"]);
    let mut asserter = env.testers().pty().run(&cmd);

    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 42);
}

/// Verify `expect_exit_code(0)` passes for successful command.
#[test]
fn expect_exit_code_matches() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("true");
    let mut asserter = env.testers().pty().run(&cmd);

    // Should not panic
    asserter.expect_exit_code(0);
}

/// Verify `expect_exit_code(0)` panics when exit code is 42.
#[test]
#[should_panic(expected = "exit code")]
fn expect_exit_code_panics_on_mismatch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "exit 42"]);
    let mut asserter = env.testers().pty().run(&cmd);

    // Should panic because exit code is 42, not 0
    asserter.expect_exit_code(0);
}
