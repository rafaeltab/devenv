mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use std::fs;
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_config_file_with_all_fields() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
            c.default_window("editor");
            c.default_window_with_command("terminal", "bash");
            c.tmux_session("test_ws", Some("session-name"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("test_ws", |d| {
                d.rafaeltab_workspace("test_ws", "Test Workspace", |w| {
                    w.tag("rust");
                    w.tag("cli");
                    w.worktree(&["npm install"], &["node_modules"]);
                });
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with full config failed: {}",
        result.stderr
    );

    assert!(
        result.stdout.contains("Test Workspace"),
        "Expected workspace in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_config_file_minimal_valid() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("minimal", |d| {
                d.rafaeltab_workspace("minimal", "Minimal", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list with minimal config
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with minimal config failed: {}",
        result.stderr
    );
}

#[test]
fn test_config_file_workspace_without_worktree() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("no_worktree", |d| {
                d.rafaeltab_workspace("no_worktree", "No Worktree", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list without worktree config failed: {}",
        result.stderr
    );
}

#[test]
fn test_config_file_global_worktree_config() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.worktree_global(&["echo 'setup'"], &[".env"]);
        });

        root.test_dir(|td| {
            td.dir("global_wt", |d| {
                d.rafaeltab_workspace("global_wt", "Global Worktree", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with global worktree config failed: {}",
        result.stderr
    );
}

#[test]
fn test_config_file_tmux_sessions_without_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session_path("standalone", "/tmp/standalone", &[("shell", None)]);
        });
    })
    .create();

    // Test tmux list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should handle path-based sessions
    assert!(
        result.success,
        "tmux list with path-based sessions failed: {}",
        result.stderr
    );
}

#[test]
fn test_config_file_multiple_workspaces() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws1", "Workspace One", |w| {
                    w.tag("rust");
                });
            });
            td.dir("ws2", |d| {
                d.rafaeltab_workspace("ws2", "Workspace Two", |w| {
                    w.tag("typescript");
                });
            });
            td.dir("ws3", |d| {
                d.rafaeltab_workspace("ws3", "Workspace Three", |w| {
                    w.tag("python");
                });
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with multiple workspaces failed: {}",
        result.stderr
    );

    // Verify all workspaces are present
    assert!(
        result.stdout.contains("Workspace One")
            && result.stdout.contains("Workspace Two")
            && result.stdout.contains("Workspace Three"),
        "Expected all workspaces in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_config_file_empty_arrays() {
    let env = TestEnvironment::describe(|root| {
        // Create a config with empty arrays
        root.test_dir(|_td| {});
    })
    .create();

    let config_path = env.root_path().join(".rafaeltab.json");
    fs::write(
        &config_path,
        r#"{"workspaces": [], "tmux": {"defaultWindows": []}, "worktree": null}"#,
    )
    .expect("Failed to write config");

    // Test workspace list with empty arrays
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(&config_path)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with empty arrays failed: {}",
        result.stderr
    );
}

#[test]
fn test_config_file_with_worktree_commands() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.worktree_global(&["npm install", "echo 'ready'"], &["package.json", ".env"]);
        });

        root.test_dir(|td| {
            td.dir("wt_cmds", |d| {
                d.rafaeltab_workspace("wt_cmds", "Worktree Commands", |w| {
                    w.worktree(&["yarn install"], &["README.md"]);
                });
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with worktree commands failed: {}",
        result.stderr
    );
}

#[test]
fn test_config_file_custom_default_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("editor", None), ("terminal", Some("bash")), ("logs", None)]);
        });

        root.test_dir(|td| {
            td.dir("custom_windows", |d| {
                d.rafaeltab_workspace("custom_windows", "Custom Windows", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with custom default windows failed: {}",
        result.stderr
    );
}

#[test]
fn test_config_file_with_session_name_override() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session_path("standalone", "/tmp/standalone", &[("shell", None)]);
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with session name override failed: {}",
        result.stderr
    );
}

#[test]
fn test_config_file_schema_validation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("schema", |d| {
                d.rafaeltab_workspace("schema", "Schema Test", |_w| {});
            });
        });
    })
    .create();

    // Test JSON output to verify schema compliance
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success, "JSON output failed: {}", result.stderr);

    // Verify valid JSON
    let json: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Output should be valid JSON");

    // Should be an array
    assert!(json.is_array(), "JSON output should be an array");
}
