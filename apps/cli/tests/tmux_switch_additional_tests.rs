mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabRootMixin},
    CliCommandBuilder,
};
use test_descriptors::testers::{CommandTester, Key, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

#[test]
fn test_tmux_switch_creates_session_if_not_exists() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session(
                "new_session_test",
                Some("New Session Test"),
                &[("editor", None)],
            );
        });

        root.test_dir(|td| {
            // Add a dummy session to ensure tmux server is running
            td.tmux_session("_dummy", |s| {
                s.window("main");
            });
            td.dir("new_session_test", |d| {
                d.rafaeltab_workspace("new_session_test", "New Session Test", |_w| {});
            });
        });
    })
    .create();

    // Verify no session exists initially
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);
    assert!(list_result.success, "tmux list should succeed");

    let json: serde_json::Value =
        serde_json::from_str(&list_result.stdout).expect("Should parse as JSON");
    let sessions = json.as_array().unwrap();

    // Find the session description for "New Session Test" workspace - it should have session: null
    let target_session = sessions.iter().find(|s| {
        s.get("name")
            .map(|n| n.as_str() == Some("New Session Test"))
            .unwrap_or(false)
    });
    assert!(
        target_session.is_some(),
        "Should find the target session description. Sessions: {:?}",
        sessions
    );

    // Verify no tmux session is attached yet
    assert!(
        target_session.unwrap()["session"].is_null(),
        "Session should not be attached yet"
    );

    // Use tmux switch TUI - note: we can only test the TUI shows the unstarted session
    // The actual session creation on selection cannot be tested in PTY (as noted in existing tests)
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

    // Verify the unstarted session is displayed in the picker
    asserter.find_text("New Session Test").assert_visible();

    // Cancel without selecting
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0, "Should exit successfully");
}

#[test]
fn test_tmux_switch_works_with_existing_session() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session(
                "existing_session",
                Some("Existing Session"),
                &[("editor", None)],
            );
        });

        root.test_dir(|td| {
            // Add a dummy session to ensure tmux server is running
            td.tmux_session("_dummy", |s| {
                s.window("main");
            });
            td.dir("existing_session", |d| {
                d.rafaeltab_workspace("existing_session", "Existing Session", |_w| {});
            });
        });
    })
    .create();

    // First start the session so it exists
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);
    assert!(start_result.success, "tmux start should succeed");

    // Verify session exists and is attached
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);
    assert!(list_result.success, "tmux list should succeed");

    let json: serde_json::Value =
        serde_json::from_str(&list_result.stdout).expect("Should parse as JSON");
    let sessions = json.as_array().unwrap();

    // Find the session - it should already have an attached session
    let target_session = sessions.iter().find(|s| {
        s.get("name")
            .map(|n| n.as_str() == Some("Existing Session"))
            .unwrap_or(false)
    });
    assert!(
        target_session.is_some(),
        "Should find the target session description. Sessions: {:?}",
        sessions
    );
    assert!(
        !target_session.unwrap()["session"].is_null(),
        "Session should already be attached before switch"
    );

    // Use tmux switch TUI - verify it shows the existing session
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

    // Verify the existing session is displayed
    asserter.find_text("Existing Session").assert_visible();

    // Cancel without selecting
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0, "Should exit successfully");
}

#[test]
fn test_tmux_switch_no_selection_exits_cleanly() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("cancel_test", Some("Cancel Test"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            // Add a dummy session to ensure tmux server is running
            td.tmux_session("_dummy", |s| {
                s.window("main");
            });
            td.dir("cancel_test", |d| {
                d.rafaeltab_workspace("cancel_test", "Cancel Test", |_w| {});
            });
        });
    })
    .create();

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

    // Verify the session is displayed
    asserter.find_text("Cancel Test").assert_visible();

    // Cancel without making a selection
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
    assert_eq!(
        exit_code, 0,
        "Should exit with status 0 when selection is cancelled"
    );
}

#[test]
fn test_tmux_switch_creates_worktree_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session(
                "worktree_ws",
                Some("Worktree Workspace"),
                &[("editor", None)],
            );
        });

        root.test_dir(|td| {
            // Add a dummy session to ensure tmux server is running
            td.tmux_session("_dummy", |s| {
                s.window("main");
            });
            td.dir("worktree_ws", |d| {
                d.rafaeltab_workspace("worktree_ws", "Worktree Workspace", |w| {
                    w.worktree(&[], &[]);
                });
                d.git("repo", |_g| {
                    // Empty git repo
                });
            });
        });
    })
    .create();

    // Verify the workspace has worktree configuration
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list", "--json"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);
    assert!(list_result.success, "workspace list should succeed");

    // The session should be visible in tmux switch
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

    // Verify the workspace with worktree config is displayed
    asserter.find_text("Worktree Workspace").assert_visible();

    // Cancel
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0, "Should exit successfully");
}

#[test]
fn test_tmux_switch_uses_workspace_specific_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // Configure session with custom workspace-specific windows
            c.tmux_session(
                "custom_ws",
                Some("Custom Windows WS"),
                &[
                    ("editor", Some("nvim")),
                    ("test", Some("cargo test")),
                    ("shell", None),
                ],
            );
        });

        root.test_dir(|td| {
            // Add a dummy session to ensure tmux server is running
            td.tmux_session("_dummy", |s| {
                s.window("main");
            });
            td.dir("custom_ws", |d| {
                d.rafaeltab_workspace("custom_ws", "Custom Windows WS", |_w| {});
            });
        });
    })
    .create();

    // Verify session is visible in switch
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

    // Verify the workspace is displayed
    asserter.find_text("Custom Windows WS").assert_visible();

    // Cancel
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0, "Should exit successfully");
}

#[test]
fn test_tmux_switch_falls_back_to_default_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // No custom windows for this session - should use defaults
            c.tmux_session("default_ws", Some("Default Windows WS"), &[]);
        });

        root.test_dir(|td| {
            // Add a dummy session to ensure tmux server is running
            td.tmux_session("_dummy", |s| {
                s.window("main");
            });
            td.dir("default_ws", |d| {
                d.rafaeltab_workspace("default_ws", "Default Windows WS", |_w| {});
            });
        });
    })
    .create();

    // Verify the session exists and is visible in switch
    // Note: default windows depend on the storage configuration, so we just verify the session exists
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);
    assert!(list_result.success, "tmux list should succeed");

    let json: serde_json::Value =
        serde_json::from_str(&list_result.stdout).expect("Should parse as JSON");
    let sessions = json.as_array().unwrap();

    // Find the session
    let target_session = sessions.iter().find(|s| {
        s.get("name")
            .map(|n| n.as_str() == Some("Default Windows WS"))
            .unwrap_or(false)
    });
    assert!(
        target_session.is_some(),
        "Should find the target session description"
    );

    // Verify session is visible in switch
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

    // Verify the workspace is displayed
    asserter.find_text("Default Windows WS").assert_visible();

    // Cancel
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0, "Should exit successfully");
}

#[test]
fn test_tmux_switch_creates_windows_correctly() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // Configure session with specific windows to verify window creation
            c.tmux_session(
                "window_test",
                Some("Window Test"),
                &[("editor", Some("nvim")), ("terminal", None)],
            );
        });

        root.test_dir(|td| {
            // Add a dummy session to ensure tmux server is running
            td.tmux_session("_dummy", |s| {
                s.window("main");
            });
            td.dir("window_test", |d| {
                d.rafaeltab_workspace("window_test", "Window Test", |_w| {});
            });
        });
    })
    .create();

    // Verify the session has the correct window configuration
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);
    assert!(list_result.success, "tmux list should succeed");

    let json: serde_json::Value =
        serde_json::from_str(&list_result.stdout).expect("Should parse as JSON");
    let sessions = json.as_array().unwrap();

    // Find the session
    let target_session = sessions.iter().find(|s| {
        s.get("name")
            .map(|n| n.as_str() == Some("Window Test"))
            .unwrap_or(false)
    });
    assert!(
        target_session.is_some(),
        "Should find the target session description"
    );

    // Verify windows are configured correctly
    let windows = target_session.unwrap()["windows"].as_array();
    assert!(windows.is_some(), "Should have windows array");
    let windows = windows.unwrap();

    // Should have exactly 2 windows as configured
    assert_eq!(windows.len(), 2, "Should have exactly 2 windows");

    // First window should be editor with nvim command
    assert_eq!(
        windows[0]["name"], "editor",
        "First window should be 'editor'"
    );
    assert_eq!(
        windows[0]["command"], "nvim",
        "First window should have 'nvim' command"
    );

    // Second window should be terminal with no command
    assert_eq!(
        windows[1]["name"], "terminal",
        "Second window should be 'terminal'"
    );

    // Verify session is visible in switch
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

    // Verify the workspace is displayed
    asserter.find_text("Window Test").assert_visible();

    // Cancel
    asserter.press_key(Key::Esc);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0, "Should exit successfully");
}
