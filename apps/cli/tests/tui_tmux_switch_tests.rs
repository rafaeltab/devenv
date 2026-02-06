mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabRootMixin},
    CliCommandBuilder,
};
use test_descriptors::testers::{CommandTester, Key, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

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

    // Start the sessions first
    let cmd_start = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result_start = env.testers().cmd().run(&cmd_start);
    assert!(result_start.success, "Failed to start tmux sessions");

    // Now test the TUI
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "switch"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // Verify UI elements are visible
    asserter.find_text("Fuzzy Picker").assert_visible();
    asserter.find_text("Query:").assert_visible();
    asserter.find_text("Matches").assert_visible();

    // Verify help text is shown
    asserter.find_text("Enter").assert_visible();
    asserter.find_text("confirm").assert_visible();
    asserter.find_text("Esc/q/Ctrl-C").assert_visible();
    asserter.find_text("cancel").assert_visible();

    // Verify all sessions are displayed
    asserter.find_text("Project A").assert_visible();
    asserter.find_text("Project B").assert_visible();
    asserter.find_text("Project C").assert_visible();

    // Cancel without selecting
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
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

    // Start the sessions first
    let cmd_start = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result_start = env.testers().cmd().run(&cmd_start);
    assert!(result_start.success, "Failed to start tmux sessions");

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "switch"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // All sessions should be visible initially
    asserter.find_text("Frontend Dev").assert_visible();
    asserter.find_text("Backend API").assert_visible();
    asserter.find_text("Database Work").assert_visible();

    // Type to filter
    asserter.type_text("back");
    asserter.wait_for_settle();

    // Only Backend should be visible after filtering
    asserter.find_text("Backend API").assert_visible();
    asserter.find_text("Frontend Dev").assert_not_visible();
    asserter.find_text("Database Work").assert_not_visible();

    // Clear filter with backspace
    asserter.press_key(Key::Backspace);
    asserter.press_key(Key::Backspace);
    asserter.press_key(Key::Backspace);
    asserter.press_key(Key::Backspace);
    asserter.wait_for_settle();

    // All should be visible again
    asserter.find_text("Frontend Dev").assert_visible();
    asserter.find_text("Backend API").assert_visible();
    asserter.find_text("Database Work").assert_visible();

    // Cancel
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
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

    // Start the sessions first
    let cmd_start = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result_start = env.testers().cmd().run(&cmd_start);
    assert!(result_start.success, "Failed to start tmux sessions");

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "switch"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // First item should be visible
    asserter.find_text("First Session").assert_visible();

    // Move down
    asserter.press_key(Key::Down);
    asserter.wait_for_settle();

    // Second item should now be visible
    asserter.find_text("Second Session").assert_visible();

    // Move down again
    asserter.press_key(Key::Down);
    asserter.wait_for_settle();

    // Third item should now be visible
    asserter.find_text("Third Session").assert_visible();

    // Move up
    asserter.press_key(Key::Up);
    asserter.wait_for_settle();

    // Second item should be visible again
    asserter.find_text("Second Session").assert_visible();

    // Cancel
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
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

    // Start the sessions first
    let cmd_start = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result_start = env.testers().cmd().run(&cmd_start);
    assert!(result_start.success, "Failed to start tmux sessions");

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "switch"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();
    asserter.find_text("Test Session").assert_visible();

    // Cancel with 'q'
    asserter.press_key(Key::Char('q'));
    let exit_code = asserter.expect_completion();
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

    // Start the sessions first
    let cmd_start = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result_start = env.testers().cmd().run(&cmd_start);
    assert!(result_start.success, "Failed to start tmux sessions");

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "switch"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();
    asserter.find_text("Test Session").assert_visible();

    // Cancel with Ctrl+C
    asserter.send_keys(&[Key::Ctrl('c')]);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0);
}
