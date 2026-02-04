//! 3.4 TmuxClientPtyTester Tests
//!
//! Tests for TUI execution inside tmux via capture-pane.

use test_descriptors::testers::{ColorMatcher, Command, Key, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// Output captured via capture-pane.
#[test]
fn tmux_client_pty_captures_pane_output() {
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

    let cmd = Command::new("echo").args(&["Captured via pane"]);
    let mut asserter = env.testers().tmux_client_pty().run(&cmd);
    asserter.wait_for_settle();

    asserter.find_text("Captured via pane").assert_visible();
}

/// Keys sent via send-keys.
#[test]
fn tmux_client_pty_sends_keys() {
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

    let cmd = Command::new("cat");
    let mut asserter = env.testers().tmux_client_pty().run(&cmd);

    asserter.type_text("typed via send-keys");
    asserter.wait_for_settle();

    asserter.find_text("typed via send-keys").assert_visible();

    asserter.send_keys(&[test_descriptors::testers::Key::Char('d')
        .with_modifier(test_descriptors::testers::Modifier::Ctrl)]);
}

/// Panics if no client.
#[test]
#[should_panic(expected = "client")]
fn tmux_client_pty_panics_without_client() {
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
    let _ = env.testers().tmux_client_pty().run(&cmd);
}

/// ANSI colors captured with -e flag.
#[test]
fn tmux_client_pty_captures_colors() {
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

    let cmd = Command::new("printf").args(&["\x1b[31mRed Output\x1b[0m"]);
    let mut asserter = env.testers().tmux_client_pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Red Output");
    text_match.fg.assert_matches(ColorMatcher::RedIsh);
}

/// Cursor positioning works.
#[test]
fn tmux_client_pty_supports_cursor_position() {
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

    // Clear and position at row 5, col 10
    let cmd = Command::new("printf").args(&["\x1b[2J\x1b[H\x1b[5;10HMarker"]);
    let mut asserter = env.testers().tmux_client_pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Marker");
    let pos = text_match.position().expect("Marker should be found");
    assert_eq!(pos, (4, 9)); // 0-indexed
}
