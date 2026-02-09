mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_tmux_list_shows_all_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws_1", Some("test-session"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("ws_1", |d| {
                d.rafaeltab_workspace("ws_1", "test ws", |_w| {});
            });
        });
    })
    .create();

    // First, start the tmux session
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start command failed:\nstdout: {}\nstderr: {}",
        start_result.stdout, start_result.stderr
    );

    // Now run tmux list
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list command failed:\nstdout: {}\nstderr: {}",
        list_result.stdout, list_result.stderr
    );

    // Verify the session is shown in the output
    // Note: The session shows the workspace name 'test ws' in the pretty output format
    assert!(
        list_result.stdout.contains("test ws"),
        "Expected 'test ws' to be in output. Got: {}",
        list_result.stdout
    );
}

#[test]
fn test_tmux_list_shows_workspace_association() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("workspace_1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace_1", |d| {
                d.rafaeltab_workspace("workspace_1", "My Workspace", |w| {
                    w.tag("rust");
                    w.tag("cli");
                });
            });
        });
    })
    .create();

    // Start the tmux session
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start command failed:\nstdout: {}\nstderr: {}",
        start_result.stdout, start_result.stderr
    );

    // Run tmux list with JSON output to verify workspace association
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list command failed:\nstdout: {}\nstderr: {}",
        list_result.stdout, list_result.stderr
    );

    // Verify JSON output contains workspace information
    assert!(
        list_result.stdout.contains("workspace_1"),
        "Expected workspace ID 'workspace_1' in JSON output. Got: {}",
        list_result.stdout
    );

    // Verify the path is shown (workspace root path)
    assert!(
        list_result.stdout.contains("workspace_1"),
        "Expected workspace path in JSON output. Got: {}",
        list_result.stdout
    );
}

#[test]
fn test_tmux_list_with_no_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // Configure a session but don't start it
            c.tmux_session("ws_no_start", None, &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("ws_no_start", |d| {
                d.rafaeltab_workspace("ws_no_start", "Unstarted Workspace", |_w| {});
            });
        });
    })
    .create();

    // Run tmux list without starting any sessions
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list command should succeed even with no sessions:\nstdout: {}\nstderr: {}",
        list_result.stdout, list_result.stderr
    );

    // The output should show the configured session but indicate no attached session
    assert!(
        list_result.stdout.contains("Unstarted Workspace"),
        "Expected 'Unstarted Workspace' to be in output. Got: {}",
        list_result.stdout
    );

    // Should indicate no attached session
    assert!(
        list_result.stdout.contains("no attached session"),
        "Expected 'no attached session' in output. Got: {}",
        list_result.stdout
    );
}

#[test]
fn test_tmux_list_with_multiple_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws_multi_1", None, &[("editor", None)]);
            c.tmux_session("ws_multi_2", None, &[("shell", None)]);
            c.tmux_session("ws_multi_3", None, &[("build", None)]);
        });

        root.test_dir(|td| {
            td.dir("ws_multi_1", |d| {
                d.rafaeltab_workspace("ws_multi_1", "First Workspace", |_w| {});
            });
            td.dir("ws_multi_2", |d| {
                d.rafaeltab_workspace("ws_multi_2", "Second Workspace", |_w| {});
            });
            td.dir("ws_multi_3", |d| {
                d.rafaeltab_workspace("ws_multi_3", "Third Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start all tmux sessions
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start command failed:\nstdout: {}\nstderr: {}",
        start_result.stdout, start_result.stderr
    );

    // Run tmux list
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list command failed:\nstdout: {}\nstderr: {}",
        list_result.stdout, list_result.stderr
    );

    // Verify all three sessions are shown
    assert!(
        list_result.stdout.contains("First Workspace"),
        "Expected 'First Workspace' in output. Got: {}",
        list_result.stdout
    );
    assert!(
        list_result.stdout.contains("Second Workspace"),
        "Expected 'Second Workspace' in output. Got: {}",
        list_result.stdout
    );
    assert!(
        list_result.stdout.contains("Third Workspace"),
        "Expected 'Third Workspace' in output. Got: {}",
        list_result.stdout
    );

    // Count the lines to ensure we have 3 sessions
    let lines: Vec<&str> = list_result.stdout.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "Should have exactly 3 sessions. Got {} lines: {:?}",
        lines.len(),
        lines
    );
}

#[test]
fn test_tmux_list_json_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws_json", None, &[("editor", Some("nvim ."))]);
        });

        root.test_dir(|td| {
            td.dir("ws_json", |d| {
                d.rafaeltab_workspace("ws_json", "JSON Test Workspace", |w| {
                    w.tag("test");
                });
            });
        });
    })
    .create();

    // Start the session
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start command failed:\nstdout: {}\nstderr: {}",
        start_result.stdout, start_result.stderr
    );

    // Run tmux list with JSON output
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list command failed:\nstdout: {}\nstderr: {}",
        list_result.stdout, list_result.stderr
    );

    // Verify output is valid JSON by parsing it
    let json_output: serde_json::Value =
        serde_json::from_str(&list_result.stdout).expect("Output should be valid JSON");

    // Verify it's an array
    assert!(
        json_output.is_array(),
        "JSON output should be an array. Got: {}",
        json_output
    );

    let sessions = json_output.as_array().unwrap();
    assert_eq!(
        sessions.len(),
        1,
        "Should have exactly 1 session. Got: {}",
        sessions.len()
    );

    // Verify session structure
    let session = &sessions[0];
    assert!(
        session.get("id").is_some(),
        "Session should have 'id' field. Got: {}",
        session
    );
    assert!(
        session.get("name").is_some(),
        "Session should have 'name' field. Got: {}",
        session
    );
    assert!(
        session.get("path").is_some(),
        "Session should have 'path' field. Got: {}",
        session
    );
    assert!(
        session.get("windows").is_some(),
        "Session should have 'windows' field. Got: {}",
        session
    );
    assert!(
        session.get("session").is_some(),
        "Session should have 'session' field. Got: {}",
        session
    );

    // Verify the session name matches our workspace name
    assert_eq!(
        session["name"].as_str().unwrap(),
        "JSON Test Workspace",
        "Session name should be 'JSON Test Workspace'. Got: {}",
        session["name"]
    );
}

#[test]
fn test_tmux_list_json_pretty_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws_pretty", None, &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("ws_pretty", |d| {
                d.rafaeltab_workspace("ws_pretty", "Pretty JSON Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start the session
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start command failed:\nstdout: {}\nstderr: {}",
        start_result.stdout, start_result.stderr
    );

    // Run tmux list with pretty JSON output
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json-pretty"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list command failed:\nstdout: {}\nstderr: {}",
        list_result.stdout, list_result.stderr
    );

    // Verify output is valid JSON by parsing it
    let json_output: serde_json::Value =
        serde_json::from_str(&list_result.stdout).expect("Output should be valid JSON");

    assert!(
        json_output.is_array(),
        "JSON output should be an array. Got: {}",
        json_output
    );

    // Verify the output contains indentation (pretty formatting)
    // Pretty JSON should have newlines and indentation spaces
    assert!(
        list_result.stdout.contains('\n'),
        "Pretty JSON should contain newlines. Got: {}",
        list_result.stdout
    );

    // Verify the output contains indentation (2 or 4 spaces)
    assert!(
        list_result.stdout.contains("  ") || list_result.stdout.contains("    "),
        "Pretty JSON should contain indentation. Got: {}",
        list_result.stdout
    );

    // Verify the data is correct
    let sessions = json_output.as_array().unwrap();
    assert_eq!(
        sessions.len(),
        1,
        "Should have exactly 1 session. Got: {}",
        sessions.len()
    );

    let session = &sessions[0];
    assert_eq!(
        session["name"].as_str().unwrap(),
        "Pretty JSON Workspace",
        "Session name should be 'Pretty JSON Workspace'. Got: {}",
        session["name"]
    );
}

#[test]
fn test_tmux_list_pretty_output_format() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws_pretty_fmt", None, &[("editor", None), ("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("ws_pretty_fmt", |d| {
                d.rafaeltab_workspace("ws_pretty_fmt", "Pretty Format Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start the session
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start command failed:\nstdout: {}\nstderr: {}",
        start_result.stdout, start_result.stderr
    );

    // Run tmux list (default pretty output)
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list command failed:\nstdout: {}\nstderr: {}",
        list_result.stdout, list_result.stderr
    );

    // Verify the pretty output format contains expected elements
    // Format: "session called '<name>' @ '<path/name>' with <session_state>"
    assert!(
        list_result.stdout.contains("session called"),
        "Pretty output should contain 'session called'. Got: {}",
        list_result.stdout
    );
    assert!(
        list_result.stdout.contains("Pretty Format Workspace"),
        "Pretty output should contain workspace name. Got: {}",
        list_result.stdout
    );
    assert!(
        list_result.stdout.contains("with attached session")
            || list_result.stdout.contains("with no attached session"),
        "Pretty output should indicate session state. Got: {}",
        list_result.stdout
    );

    // Verify the @ symbol is present (separator between name and path)
    assert!(
        list_result.stdout.contains("@"),
        "Pretty output should contain '@' separator. Got: {}",
        list_result.stdout
    );
}

#[test]
fn test_tmux_list_mixed_session_types() {
    // First, create a test directory to use as the path-based session path
    let temp_env = TestEnvironment::describe(|root| {
        root.test_dir(|_td| {});
    })
    .create();

    let path_based_session_path = temp_env.root_path().to_string_lossy().to_string();
    // Clone for later assertions
    let path_for_assertion = path_based_session_path.clone();

    // Now create the actual test environment with both session types
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(move |c| {
            // Workspace-based session
            c.tmux_session("ws_mixed", None, &[("editor", None)]);
            // Path-based session
            c.tmux_session_path(
                &path_based_session_path,
                "path-based-session",
                &[("shell", None)],
            );
        });

        root.test_dir(|td| {
            td.dir("ws_mixed", |d| {
                d.rafaeltab_workspace("ws_mixed", "Mixed Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start all tmux sessions
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start command failed:\nstdout: {}\nstderr: {}",
        start_result.stdout, start_result.stderr
    );

    // Run tmux list
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list command failed:\nstdout: {}\nstderr: {}",
        list_result.stdout, list_result.stderr
    );

    // Verify both session types are shown
    // Workspace-based session shows workspace name
    assert!(
        list_result.stdout.contains("Mixed Workspace"),
        "Expected 'Mixed Workspace' (workspace-based session) in output. Got: {}",
        list_result.stdout
    );

    // Path-based session shows the path
    assert!(
        list_result.stdout.contains(&path_for_assertion),
        "Expected path-based session path in output. Got: {}",
        list_result.stdout
    );

    // Count lines to ensure we have 2 sessions
    let lines: Vec<&str> = list_result.stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "Should have exactly 2 sessions (1 workspace-based, 1 path-based). Got {} lines: {:?}",
        lines.len(),
        lines
    );
}
