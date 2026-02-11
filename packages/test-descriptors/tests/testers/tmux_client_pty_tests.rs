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

    // Use wait_for_text for deterministic waiting
    asserter.wait_for_text("Captured via pane");
    // Captures both the command itself, and the output
    assert_eq!(2, asserter.find_all_text("Captured via pane").len());
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

    // Use wait_for_text for deterministic waiting
    asserter.wait_for_text("typed via send-keys");
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

    // Use a unique marker that won't appear in the command itself
    let cmd = Command::new("printf").args(&["\x1b[31mCOLORED_MARKER\x1b[0m"]);
    let mut asserter = env.testers().tmux_client_pty().run(&cmd);

    // Use wait_for_text for deterministic waiting
    asserter.wait_for_text("COLORED_MARKER");

    // find_all_text since the text may appear in both the command and output
    let matches = asserter.find_all_text("COLORED_MARKER");
    assert!(!matches.is_empty(), "COLORED_MARKER should be found");

    // At least one match should have red foreground color
    let has_red = matches.iter().any(|m| {
        m.fg.rgb().map_or(false, |(r, g, b)| {
            // Check if it's reddish (red > green and red > blue by a margin)
            r > g.saturating_add(30) && r > b.saturating_add(30)
        })
    });
    assert!(
        has_red,
        "At least one COLORED_MARKER should have red foreground"
    );
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

    // Use wait_for_text for deterministic waiting
    asserter.wait_for_text("Marker");

    let text_match = asserter.find_text("Marker");
    let pos = text_match.position().expect("Marker should be found");
    assert_eq!(pos, (4, 9)); // 0-indexed
}
