mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabRootMixin},
    run_cli_tui,
};
use test_descriptors::TestEnvironment;
use tui_test::Key;

#[test]
fn test_tmux_switch_displays_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("project-a", Some("Project A"), &[("shell", None)]);
            c.tmux_session("project-b", Some("Project B"), &[("shell", None)]);
            c.tmux_session("project-c", Some("Project C"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project-a", |d| {
                d.rafaeltab_workspace("project-a", "Project A", |_w| {});
            });
            td.dir("project-b", |d| {
                d.rafaeltab_workspace("project-b", "Project B", |_w| {});
            });
            td.dir("project-c", |d| {
                d.rafaeltab_workspace("project-c", "Project C", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Start the sessions first
    let (_, _, success) = common::run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );
    assert!(success, "Failed to start tmux sessions");

    // Now test the TUI
    let mut tui = run_cli_tui(
        &["tmux", "switch"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // Verify UI elements are visible
    tui.find_text("Fuzzy Picker").assert_visible();
    tui.find_text("Query:").assert_visible();
    tui.find_text("Matches").assert_visible();

    // Verify all sessions are displayed
    tui.find_text("Project A").assert_visible();
    tui.find_text("Project B").assert_visible();
    tui.find_text("Project C").assert_visible();

    // Verify help text is shown
    tui.find_text("Enter").assert_visible();
    tui.find_text("confirm").assert_visible();
    tui.find_text("Esc/q/Ctrl-C").assert_visible();
    tui.find_text("cancel").assert_visible();

    // Cancel without selecting
    tui.press_key(Key::Esc);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_tmux_switch_fuzzy_filtering() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("frontend", Some("Frontend Dev"), &[("shell", None)]);
            c.tmux_session("backend", Some("Backend API"), &[("shell", None)]);
            c.tmux_session("database", Some("Database Work"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("frontend", |d| {
                d.rafaeltab_workspace("frontend", "Frontend Dev", |_w| {});
            });
            td.dir("backend", |d| {
                d.rafaeltab_workspace("backend", "Backend API", |_w| {});
            });
            td.dir("database", |d| {
                d.rafaeltab_workspace("database", "Database Work", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Start the sessions first
    let (_, _, success) = common::run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );
    assert!(success, "Failed to start tmux sessions");

    let mut tui = run_cli_tui(
        &["tmux", "switch"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // All sessions should be visible initially
    tui.find_text("Frontend Dev").assert_visible();
    tui.find_text("Backend API").assert_visible();
    tui.find_text("Database Work").assert_visible();

    // Type to filter
    tui.type_text("back");
    tui.wait_for_settle();

    // Only Backend should be visible after filtering
    tui.find_text("Backend API").assert_visible();
    tui.find_text("Frontend Dev").assert_not_visible();
    tui.find_text("Database Work").assert_not_visible();

    // Clear filter with backspace
    tui.press_key(Key::Backspace);
    tui.press_key(Key::Backspace);
    tui.press_key(Key::Backspace);
    tui.press_key(Key::Backspace);
    tui.wait_for_settle();

    // All should be visible again
    tui.find_text("Frontend Dev").assert_visible();
    tui.find_text("Backend API").assert_visible();
    tui.find_text("Database Work").assert_visible();

    // Cancel
    tui.press_key(Key::Esc);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_tmux_switch_navigation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("first", Some("First Session"), &[("shell", None)]);
            c.tmux_session("second", Some("Second Session"), &[("shell", None)]);
            c.tmux_session("third", Some("Third Session"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("first", |d| {
                d.rafaeltab_workspace("first", "First Session", |_w| {});
            });
            td.dir("second", |d| {
                d.rafaeltab_workspace("second", "Second Session", |_w| {});
            });
            td.dir("third", |d| {
                d.rafaeltab_workspace("third", "Third Session", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Start the sessions first
    let (_, _, success) = common::run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );
    assert!(success, "Failed to start tmux sessions");

    let mut tui = run_cli_tui(
        &["tmux", "switch"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // First item should be highlighted (yellow)
    let first_match = tui.find_text("First Session");
    first_match.assert_visible();
    first_match.fg.assert(tui_test::ColorMatcher::YellowIsh);

    // Move down
    tui.press_key(Key::Down);
    tui.wait_for_settle();

    // Second item should now be highlighted
    let second_match = tui.find_text("Second Session");
    second_match.assert_visible();
    second_match.fg.assert(tui_test::ColorMatcher::YellowIsh);

    // Move down again
    tui.press_key(Key::Down);
    tui.wait_for_settle();

    // Third item should now be highlighted
    let third_match = tui.find_text("Third Session");
    third_match.assert_visible();
    third_match.fg.assert(tui_test::ColorMatcher::YellowIsh);

    // Move up
    tui.press_key(Key::Up);
    tui.wait_for_settle();

    // Second item should be highlighted again
    let second_match = tui.find_text("Second Session");
    second_match.assert_visible();
    second_match.fg.assert(tui_test::ColorMatcher::YellowIsh);

    // Cancel
    tui.press_key(Key::Esc);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_tmux_switch_cancel_with_q() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("test", Some("Test Session"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("test", |d| {
                d.rafaeltab_workspace("test", "Test Session", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Start the sessions first
    let (_, _, success) = common::run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );
    assert!(success, "Failed to start tmux sessions");

    let mut tui = run_cli_tui(
        &["tmux", "switch"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();
    tui.find_text("Test Session").assert_visible();

    // Cancel with 'q'
    tui.press_key(Key::Char('q'));
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_tmux_switch_cancel_with_ctrl_c() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("test", Some("Test Session"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("test", |d| {
                d.rafaeltab_workspace("test", "Test Session", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Start the sessions first
    let (_, _, success) = common::run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );
    assert!(success, "Failed to start tmux sessions");

    let mut tui = run_cli_tui(
        &["tmux", "switch"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();
    tui.find_text("Test Session").assert_visible();

    // Cancel with Ctrl+C
    tui.send_keys(&[Key::Ctrl, Key::Char('c')]);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}
