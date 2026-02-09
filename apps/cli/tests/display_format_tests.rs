mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabGitMixin, RafaeltabRootMixin},
    CliCommandBuilder,
};
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_json_output_valid_structure() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace1", |d| {
                d.rafaeltab_workspace("ws1", "Workspace One", |w| {
                    w.tag("rust");
                });
            });
        });
    })
    .create();

    // Test workspace list with --json
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "Command should succeed.\nSTDERR: {}",
        result.stderr
    );

    // Verify output is valid JSON
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&result.stdout);
    assert!(
        json_result.is_ok(),
        "Output should be valid JSON. Got: {}",
        result.stdout
    );
}

#[test]
fn test_json_pretty_output_formatted() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace1", |d| {
                d.rafaeltab_workspace("ws1", "Workspace One", |_w| {});
            });
        });
    })
    .create();

    // Test with --json-pretty
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list", "--json-pretty"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "Command should succeed.\nSTDERR: {}",
        result.stderr
    );

    // Verify it's valid JSON
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&result.stdout);
    assert!(
        json_result.is_ok(),
        "Output should be valid JSON. Got: {}",
        result.stdout
    );

    // Verify it has newlines (pretty formatting)
    assert!(
        result.stdout.contains('\n'),
        "Pretty JSON should contain newlines. Got: {}",
        result.stdout
    );
}

#[test]
fn test_json_output_contains_expected_fields() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace1", |d| {
                d.rafaeltab_workspace("ws1", "Workspace One", |w| {
                    w.tag("rust");
                    w.tag("cli");
                });
            });
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success, "Command should succeed");

    let json: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Should parse as JSON");

    // Should be an array
    assert!(json.is_array(), "JSON should be an array");

    let workspaces = json.as_array().unwrap();
    assert!(!workspaces.is_empty(), "Should have at least one workspace");

    let first = &workspaces[0];

    // Verify required fields (workspace JSON uses 'root' for path, not 'path')
    assert!(
        first.get("id").is_some(),
        "Workspace should have 'id' field"
    );
    assert!(
        first.get("name").is_some(),
        "Workspace should have 'name' field"
    );
    assert!(
        first.get("root").is_some(),
        "Workspace should have 'root' field (path)"
    );
    assert!(
        first.get("tags").is_some(),
        "Workspace should have 'tags' field"
    );
}

#[test]
fn test_pretty_output_format() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace1", |d| {
                d.rafaeltab_workspace("ws1", "Test Workspace", |_w| {});
            });
        });
    })
    .create();

    // Test default pretty output (no --json flags)
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "Command should succeed.\nSTDERR: {}",
        result.stderr
    );

    // Pretty output should contain workspace name
    assert!(
        result.stdout.contains("Test Workspace"),
        "Pretty output should contain workspace name. Got: {}",
        result.stdout
    );
}

#[test]
fn test_json_output_empty_arrays() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace1", |d| {
                d.rafaeltab_workspace("ws1", "No Tags Workspace", |w| {
                    // No tags added
                });
            });
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success, "Command should succeed");

    let json: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Should parse as JSON");

    let workspaces = json.as_array().unwrap();
    let first = &workspaces[0];

    // Tags field should be an empty array, not null
    assert!(
        first["tags"].is_array(),
        "Tags should be an array (possibly empty), not null"
    );
}

#[test]
fn test_both_json_flags_together() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace1", |d| {
                d.rafaeltab_workspace("ws1", "Test", |_w| {});
            });
        });
    })
    .create();

    // Test with both --json and --json-pretty
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list", "--json", "--json-pretty"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "Command should succeed.\nSTDERR: {}",
        result.stderr
    );

    // Should be valid JSON
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&result.stdout);
    assert!(
        json_result.is_ok(),
        "Output should be valid JSON. Got: {}",
        result.stdout
    );

    // With both flags, pretty takes precedence (should have newlines)
    assert!(
        result.stdout.contains('\n'),
        "With both flags, pretty formatting should be used"
    );
}

#[test]
fn test_json_output_workspace_structure() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace1", |d| {
                d.rafaeltab_workspace("ws1", "Test Workspace", |w| {
                    w.tag("test");
                });
            });
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success, "Command should succeed");

    let json: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Should parse as JSON");

    let workspaces = json.as_array().unwrap();
    let workspace = &workspaces[0];

    // Verify structure: id, name, root, tags
    assert!(workspace.get("id").is_some(), "Should have id");
    assert!(workspace.get("name").is_some(), "Should have name");
    assert!(workspace.get("root").is_some(), "Should have root (path)");
    assert!(workspace.get("tags").is_some(), "Should have tags");

    // Verify types
    assert!(workspace["id"].is_string(), "id should be a string");
    assert!(workspace["name"].is_string(), "name should be a string");
    assert!(workspace["root"].is_string(), "root should be a string");
    assert!(workspace["tags"].is_array(), "tags should be an array");
}

#[test]
fn test_display_command_inheritance() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace1", |d| {
                d.rafaeltab_workspace("ws1", "Test", |_w| {});
            });
        });
    })
    .create();

    // Test different subcommands with --json
    let subcommands = vec![
        vec!["workspace", "list", "--json"],
        vec!["workspace", "find", "ws1", "--json"],
    ];

    for args in subcommands {
        let cmd = CliCommandBuilder::new().with_env(&env).args(&args).build();
        let result = env.testers().cmd().run(&cmd);

        assert!(
            result.success,
            "Command {:?} should succeed.\nSTDERR: {}",
            args, result.stderr
        );

        // Verify valid JSON output
        let json_result: Result<serde_json::Value, _> = serde_json::from_str(&result.stdout);
        assert!(
            json_result.is_ok(),
            "Command {:?} should produce valid JSON. Got: {}",
            args,
            result.stdout
        );
    }
}

#[test]
fn test_json_output_null_handling() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws_no_session", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace_no_session", |d| {
                d.rafaeltab_workspace("ws_no_session", "No Session Workspace", |w| {
                    w.tag("test");
                });
            });
        });
    })
    .create();

    // Test with tmux list (which shows session descriptions with optional session field)
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "Command should succeed.\nSTDERR: {}",
        result.stderr
    );

    // Parse JSON
    let json: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Should parse as JSON");

    // Should be an array of session descriptions
    assert!(json.is_array(), "JSON should be an array");

    let sessions = json.as_array().unwrap();
    assert!(
        !sessions.is_empty(),
        "Should have at least one session description"
    );

    let first_session = &sessions[0];

    // Verify required fields are present
    assert!(
        first_session.get("id").is_some(),
        "Session should have 'id' field"
    );
    assert!(
        first_session.get("name").is_some(),
        "Session should have 'name' field"
    );
    assert!(
        first_session.get("path").is_some(),
        "Session should have 'path' field"
    );
    assert!(
        first_session.get("windows").is_some(),
        "Session should have 'windows' field"
    );

    // Verify session field exists (can be null when no tmux session is attached)
    assert!(
        first_session.get("session").is_some(),
        "Session should have 'session' field (even if null)"
    );

    // The session field should either be null or an object
    let session_field = &first_session["session"];
    assert!(
        session_field.is_null() || session_field.is_object(),
        "Session field should be either null or an object. Got: {:?}",
        session_field
    );
}

#[test]
fn test_json_output_session_structure() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws_with_session", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace_with_session", |d| {
                d.rafaeltab_workspace("ws_with_session", "With Session Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start the tmux session first (starts all configured sessions)
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

    // Test with tmux list JSON output
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "Command should succeed.\nSTDERR: {}",
        result.stderr
    );

    // Parse JSON
    let json: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Should parse as JSON");

    let sessions = json.as_array().expect("Should be an array");
    let session = &sessions[0];

    // Verify session structure: id, name, path, windows, session
    assert!(session.get("id").is_some(), "Should have id");
    assert!(session.get("name").is_some(), "Should have name");
    assert!(session.get("path").is_some(), "Should have path");
    assert!(session.get("windows").is_some(), "Should have windows");
    assert!(
        session.get("session").is_some(),
        "Should have session field"
    );

    // Verify types
    assert!(session["id"].is_string(), "id should be a string");
    assert!(session["name"].is_string(), "name should be a string");
    assert!(session["path"].is_string(), "path should be a string");
    assert!(session["windows"].is_array(), "windows should be an array");

    // When session is present (not null), verify its structure
    if session["session"].is_object() {
        let attached_session = &session["session"];
        // The attached session should have name and id fields
        assert!(
            attached_session.get("name").is_some(),
            "Attached session should have 'name' field"
        );
        assert!(
            attached_session.get("id").is_some(),
            "Attached session should have 'id' field"
        );
    }
}
