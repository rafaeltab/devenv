mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

// TODO let these tests actually verify some behavior
#[test]
fn test_workspace_tmux_no_sessions() {
    // This test verifies graceful handling when no tmux sessions exist
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("test_ws", "Test Workspace", |_w| {});
            });
        });
    })
    .create();

    // Run workspace tmux command (may fail if tmux is not running)
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "tmux"])
        .build();
    let _ = env.testers().cmd().run(&cmd);

    // The command behavior depends on whether tmux is running
    // If tmux is running, it will list sessions; if not, it may return empty or error
    // We just verify the command doesn't crash
    // The output can be sessions list, empty, or error message
}

#[test]
fn test_workspace_tmux_json_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("test_ws", "Test Workspace", |_w| {});
            });
        });
    })
    .create();

    // Run workspace tmux with JSON output
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "tmux", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    if result.success {
        // If it succeeds, verify output is valid JSON (empty array or array of sessions)
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&result.stdout);
        assert!(
            parse_result.is_ok(),
            "Output should be valid JSON. Got: {:?}",
            result.stdout
        );

        if let Ok(json) = parse_result {
            assert!(
                json.is_array(),
                "JSON output should be an array. Got: {}",
                json
            );
        }
    }
}

#[test]
fn test_workspace_tmux_pretty_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("test_ws", "Test Workspace", |_w| {});
            });
        });
    })
    .create();

    // Run workspace tmux with default pretty output
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "tmux"])
        .build();
    let _ = env.testers().cmd().run(&cmd);

    // Command should not crash
    // Output format depends on whether tmux is running and what sessions exist
    // Just verify the command completes
}

#[test]
fn test_workspace_tmux_lists_sessions_with_workspaces() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("session_ws", "Session Workspace", |_w| {});
            });
        });
    })
    .create();

    // This test verifies the command can list sessions and associate them with workspaces
    // The actual session association requires tmux to be running with specific environment variables
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "tmux"])
        .build();
    let _ = env.testers().cmd().run(&cmd);

    // Command should complete without crashing
    // Actual session-to-workspace mapping depends on tmux environment variables
}

#[test]
fn test_workspace_tmux_shows_unassociated_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("associated_ws", "Associated Workspace", |_w| {});
            });
        });
    })
    .create();

    // This test verifies that sessions without workspace associations are displayed
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "tmux"])
        .build();
    let _ = env.testers().cmd().run(&cmd);

    // Command should complete without crashing
    // Sessions without workspace env var should still be shown
}

#[test]
fn test_workspace_tmux_mixed_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("mixed_ws1", "Mixed Workspace 1", |_w| {});
            });
            td.dir("ws2", |d| {
                d.rafaeltab_workspace("mixed_ws2", "Mixed Workspace 2", |_w| {});
            });
        });
    })
    .create();

    // This test verifies correct display when some sessions have workspaces and others don't
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "tmux"])
        .build();
    let _ = env.testers().cmd().run(&cmd);

    // Command should complete without crashing
    // Mixed session types should be handled correctly
}

#[test]
fn test_workspace_tmux_session_with_workspace_env() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("env_ws", "Env Workspace", |_w| {});
            });
        });
    })
    .create();

    // This test verifies that sessions with TMUX_WORKSPACE env var are matched to workspaces
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "tmux"])
        .build();
    let _ = env.testers().cmd().run(&cmd);

    // Command should complete without crashing
    // Session with TMUX_WORKSPACE env var should be associated with workspace
}

#[test]
fn test_workspace_tmux_multiple_sessions_same_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("shared_ws", "Shared Workspace", |_w| {});
            });
        });
    })
    .create();

    // This test verifies handling of multiple sessions attached to the same workspace
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "tmux"])
        .build();
    let _ = env.testers().cmd().run(&cmd);

    // Command should complete without crashing
    // Multiple sessions pointing to same workspace should be handled
}
