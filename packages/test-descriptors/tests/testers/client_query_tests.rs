//! 4.2 Client Query Tests
//!
//! Tests for querying tmux client state.

use test_descriptors::testers::{Command, CommandTester};
use test_descriptors::TestEnvironment;

/// Returns true when client exists.
#[test]
fn has_tmux_client_returns_true() {
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

/// Returns false when no client.
#[test]
fn has_tmux_client_returns_false() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("test-session", |s| {
                    s.window("main");
                    // No with_client()
                });
            });
        });
    })
    .create();

    assert!(!env.has_tmux_client());
}

/// Returns client handle when exists.
#[test]
fn tmux_client_returns_handle() {
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

    let client = env.tmux_client();
    assert!(client.is_some());
}

/// Returns None when no client.
#[test]
fn tmux_client_returns_none() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("test-session", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let client = env.tmux_client();
    assert!(client.is_none());
}

/// Returns session name.
#[test]
fn current_session_returns_attached_session() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("my-session", |s| {
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
    assert_eq!(client.current_session(), "my-session");
}

/// Returns new session after switch.
#[test]
fn current_session_updates_after_switch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("session-1", |s| {
                    s.window("main");
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
                d.tmux_session("session-2", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let client = env.tmux_client().expect("Client should exist");
    assert_eq!(client.current_session(), "session-1");

    // Switch to session-2 using tmux command
    env.tmux()
        .run_tmux(&["switch-client", "-t", "session-2"])
        .expect("Switch should succeed");

    // Client should now report session-2
    assert_eq!(client.current_session(), "session-2");
}
