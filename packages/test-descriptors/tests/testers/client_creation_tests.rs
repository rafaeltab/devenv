//! 4.1 Client Creation Tests
//!
//! Tests for tmux client creation and configuration.

use test_descriptors::TestEnvironment;

/// Client is spawned.
#[test]
fn with_client_creates_tmux_client() {
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

    assert!(env.has_tmux_client());
}

/// Client attached to correct session.
#[test]
fn with_client_attaches_to_session() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("target-session", |s| {
                    s.window("main");
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
            });
        });
    })
    .create();

    let client = env.tmux_client().expect("Client should exist");
    assert_eq!(client.current_session(), "target-session");
}

/// Second `with_client()` errors.
#[test]
#[should_panic(expected = "one client")]
fn with_client_only_one_allowed() {
    let _env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("session1", |s| {
                    s.window("main");
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
                d.tmux_session("session2", |s| {
                    s.window("main");
                    // Second client should error
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
            });
        });
    })
    .create();
}

/// PTY size is set correctly.
#[test]
fn with_client_respects_pty_size() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("test-session", |s| {
                    s.window("main");
                    s.with_client(|c| {
                        c.pty_size(30, 100);
                    });
                });
            });
        });
    })
    .create();

    let client = env.tmux_client().expect("Client should exist");
    let (rows, cols) = client.pty_size();
    assert_eq!(rows, 30);
    assert_eq!(cols, 100);
}
