//! 2.1 Basic Execution Tests
//!
//! Tests for Command and CommandResult based testers.

use test_descriptors::testers::{Command, CommandTester};
use test_descriptors::TestEnvironment;

/// Run `echo hello`, verify stdout.
#[test]
fn run_simple_command() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["hello"]);
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert_eq!(result.stdout.trim(), "hello");
}

/// Run with multiple arguments.
#[test]
fn run_command_with_args() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["hello", "world", "test"]);
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert_eq!(result.stdout.trim(), "hello world test");
}

/// Verify stderr is captured separately.
#[test]
fn run_command_captures_stderr() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "echo stdout; echo stderr >&2"]);
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert!(result.stdout.contains("stdout"));
    assert!(result.stderr.contains("stderr"));
    assert!(!result.stdout.contains("stderr"));
}

/// Non-zero exit code captured.
#[test]
fn run_command_captures_exit_code() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "exit 42"]);
    let result = env.testers().cmd().run(&cmd);

    assert_eq!(result.exit_code, 42);
}

/// Environment variable is accessible.
#[test]
fn run_command_with_env_var() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh")
        .args(&["-c", "echo $MY_VAR"])
        .env("MY_VAR", "test_value");
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert_eq!(result.stdout.trim(), "test_value");
}

/// Working directory is set correctly.
#[test]
fn run_command_with_cwd() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("my-workspace", |_d| {});
        });
    })
    .create();

    let workspace_path = env.find_dir("my-workspace").unwrap().path();
    let cmd = Command::new("pwd").cwd(&workspace_path);
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert!(result.stdout.trim().ends_with("my-workspace"));
}

/// `success` is true when exit_code is 0.
#[test]
fn run_command_success_flag() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("true");
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert_eq!(result.exit_code, 0);
}

/// `success` is false when exit_code is non-zero.
#[test]
fn run_command_failure_flag() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("false");
    let result = env.testers().cmd().run(&cmd);

    assert!(!result.success);
    assert_ne!(result.exit_code, 0);
}
