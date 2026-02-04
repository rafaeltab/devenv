//! 3.5 TmuxFullClientTester Tests
//!
//! Tests for full tmux client PTY interaction (including tmux UI).

use test_descriptors::testers::{Command, Key, Modifier, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// Tmux status bar visible.
#[test]
fn tmux_full_client_shows_tmux_ui() {
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

    let cmd = Command::new("echo").args(&["Hello"]);
    let mut asserter = env.testers().tmux_full_client().run(&cmd);
    asserter.wait_for_settle();

    // Should see tmux UI elements (session name in status bar)
    asserter.find_text("my-session").assert_visible();
    let hello_occurances = asserter.find_all_text("Hello");
    assert_eq!(hello_occurances.len(), 2, "Output should have both the command and the output of the command");
}

/// Panics if no client.
#[test]
#[should_panic(expected = "client")]
fn tmux_full_client_panics_without_client() {
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
    let _ = env.testers().tmux_full_client().run(&cmd);
}

/// Full PTY keyboard support.
#[test]
fn tmux_full_client_full_pty_interaction() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("interactive-session", |s| {
                    s.window("shell");
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
            });
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().tmux_full_client().run(&cmd);
    asserter.wait_for_settle();

    // Type text
    asserter.type_text("full client test");
    asserter.wait_for_settle();
    asserter.find_text("full client test").assert_visible();

    // Send arrow keys (should work with full PTY)
    asserter.press_key(Key::Up);
    asserter.press_key(Key::Down);

    // Exit
    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);

    // Tmux UI should still be visible
    asserter.find_text("interactive-session").assert_visible();
}
