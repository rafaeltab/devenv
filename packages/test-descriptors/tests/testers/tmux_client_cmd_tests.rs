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

/// Command runs in the specified working directory.
#[test]
fn tmux_client_cmd_runs_in_specified_cwd() {
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

    // Create a test directory
    let test_dir = env.root_path().join("test_working_dir");
    std::fs::create_dir(&test_dir).expect("Failed to create test directory");

    // Canonicalize the path to resolve symlinks (e.g., /tmp -> /private/tmp on macOS)
    let canonical_dir = std::fs::canonicalize(&test_dir).expect("Failed to canonicalize path");
    let expected_path = canonical_dir.to_string_lossy().to_string();

    // Run pwd -P to get physical path (equivalent to std::env::current_dir())
    let cmd = Command::new("pwd").args(&["-P"]).cwd(&test_dir);
    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(result.success, "Command should succeed: {}", result.stderr);
    let actual_path = result.stdout.trim();
    assert_eq!(
        actual_path, expected_path,
        "Working directory should match specified cwd.\nExpected: {}\nActual: {}",
        expected_path, actual_path
    );
}

/// Command runs in /tmp correctly (symlink handling on macOS).
#[test]
fn tmux_client_cmd_handles_tmp_symlink() {
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

    // Canonicalize /tmp to handle macOS symlink (/tmp -> /private/tmp)
    let tmp_path = std::path::PathBuf::from("/tmp");
    let canonical_tmp = std::fs::canonicalize(&tmp_path).expect("Failed to canonicalize /tmp");
    let expected_path = canonical_tmp.to_string_lossy().to_string();

    // Run pwd -P to get physical path
    let cmd = Command::new("pwd").args(&["-P"]).cwd(&tmp_path);
    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(result.success, "Command should succeed: {}", result.stderr);
    let actual_path = result.stdout.trim();
    assert_eq!(
        actual_path, expected_path,
        "Working directory should match canonicalized /tmp.\nExpected: {}\nActual: {}",
        expected_path, actual_path
    );
}

/// Nested directory paths are handled correctly.
#[test]
fn tmux_client_cmd_handles_nested_paths() {
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

    // Create nested directories
    let nested_dir = env.root_path().join("level1").join("level2").join("level3");
    std::fs::create_dir_all(&nested_dir).expect("Failed to create nested directories");

    let canonical_dir = std::fs::canonicalize(&nested_dir).expect("Failed to canonicalize path");
    let expected_path = canonical_dir.to_string_lossy().to_string();

    let cmd = Command::new("pwd").args(&["-P"]).cwd(&nested_dir);
    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(result.success, "Command should succeed: {}", result.stderr);
    let actual_path = result.stdout.trim();
    assert_eq!(
        actual_path, expected_path,
        "Working directory should match nested path.\nExpected: {}\nActual: {}",
        expected_path, actual_path
    );
}
