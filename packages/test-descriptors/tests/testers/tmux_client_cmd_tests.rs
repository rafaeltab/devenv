//! 3.2 TmuxClientCmdTester Tests
//!
//! Tests for command execution inside a tmux client.

use test_descriptors::testers::{Command, CommandTester};
use test_descriptors::TestEnvironment;

/// $TMUX env var is set.
#[test]
fn tmux_client_cmd_runs_inside_client() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("test-session", |s| {
                    s.window("main");
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
            });
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "echo TMUX=$TMUX"]);
    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(result.success);
    // TMUX should be set when running inside tmux
    assert!(
        !result.stdout.contains("TMUX=\n") && !result.stdout.contains("TMUX=$"),
        "TMUX should be set when running inside tmux client"
    );
}

/// Panics if no client was created.
#[test]
#[should_panic(expected = "client")]
fn tmux_client_cmd_panics_without_client() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                // Session without client
                d.tmux_session("test-session", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let cmd = Command::new("echo").args(&["test"]);
    // Should panic - no client
    let _ = env.testers().tmux_client_cmd().run(&cmd);
}

/// stdout and stderr are separate.
#[test]
fn tmux_client_cmd_separates_stdout_stderr() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("test-session", |s| {
                    s.window("main");
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
            });
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "echo stdout_msg; echo stderr_msg >&2"]);
    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(result.success);
    assert!(result.stdout.contains("stdout_msg"));
    assert!(result.stderr.contains("stderr_msg"));
    assert!(!result.stdout.contains("stderr_msg"));
    assert!(!result.stderr.contains("stdout_msg"));
}

/// Exit code is correct.
#[test]
fn tmux_client_cmd_captures_exit_code() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("test-session", |s| {
                    s.window("main");
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
            });
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "exit 123"]);
    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(!result.success);
    assert_eq!(result.exit_code, 123);
}
