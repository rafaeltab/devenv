mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_workspace_current_shows_workspace_in_cwd() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("my_workspace", |d| {
                d.rafaeltab_workspace("my_ws", "My Workspace", |w| {
                    w.tag("test");
                    w.tag("rust");
                });
            });
        });
    })
    .create();

    // Get the path to the workspace directory
    let workspace_path = env.root_path().join("my_workspace");

    // Run workspace current from within the workspace directory
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "current"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace current command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify the workspace is found
    assert!(
        result.stdout.contains("My Workspace"),
        "Expected 'My Workspace' in output. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("my_ws"),
        "Expected workspace ID 'my_ws' in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_current_shows_nothing_outside_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("some_workspace", |d| {
                d.rafaeltab_workspace("some_ws", "Some Workspace", |_w| {});
            });
            // Create a directory that is NOT a workspace
            td.dir("not_a_workspace", |_| {
                // This directory has no workspace marker
            });
        });
    })
    .create();

    // Get the path to the non-workspace directory
    let non_workspace_path = env.root_path().join("not_a_workspace");

    // Run workspace current from outside any workspace directory
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&non_workspace_path)
        .args(&["workspace", "current"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace current command should succeed even outside workspace.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify no workspace is found (empty output)
    assert!(
        result.stdout.trim().is_empty(),
        "Expected empty output when outside any workspace. Got: {:?}",
        result.stdout
    );
}

#[test]
fn test_workspace_current_nested_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            // Parent workspace
            td.dir("parent_workspace", |d| {
                d.rafaeltab_workspace("parent_ws", "Parent Workspace", |_w| {});
                // Nested child workspace inside parent
                d.dir("child_workspace", |child| {
                    child.rafaeltab_workspace("child_ws", "Child Workspace", |_w| {});
                });
            });
        });
    })
    .create();

    // Get the path to the nested child workspace directory
    let nested_path = env.root_path().join("parent_workspace/child_workspace");

    // Run workspace current from within the nested workspace
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&nested_path)
        .args(&["workspace", "current"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace current command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify the most specific (deepest nested) workspace is shown
    assert!(
        result.stdout.contains("Child Workspace"),
        "Expected 'Child Workspace' (the most specific nested workspace) in output. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("child_ws"),
        "Expected workspace ID 'child_ws' in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_current_json_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("json_workspace", |d| {
                d.rafaeltab_workspace("json_ws", "JSON Test Workspace", |w| {
                    w.tag("json");
                    w.tag("test");
                });
            });
        });
    })
    .create();

    // Get the path to the workspace directory
    let workspace_path = env.root_path().join("json_workspace");

    // Run workspace current with JSON output
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "current", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace current command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify output is valid JSON by parsing it
    let json_output: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Output should be valid JSON");

    // Verify workspace structure
    assert!(
        json_output.get("id").is_some(),
        "JSON should have 'id' field. Got: {}",
        json_output
    );
    assert!(
        json_output.get("name").is_some(),
        "JSON should have 'name' field. Got: {}",
        json_output
    );
    assert!(
        json_output.get("root").is_some(),
        "JSON should have 'root' field. Got: {}",
        json_output
    );
    assert!(
        json_output.get("tags").is_some(),
        "JSON should have 'tags' field. Got: {}",
        json_output
    );

    // Verify the values
    assert_eq!(
        json_output["id"].as_str().unwrap(),
        "json_ws",
        "Workspace ID should be 'json_ws'. Got: {}",
        json_output["id"]
    );
    assert_eq!(
        json_output["name"].as_str().unwrap(),
        "JSON Test Workspace",
        "Workspace name should be 'JSON Test Workspace'. Got: {}",
        json_output["name"]
    );

    // Verify tags array
    let tags = json_output["tags"]
        .as_array()
        .expect("Tags should be an array");
    assert_eq!(tags.len(), 2, "Should have 2 tags");
}

#[test]
fn test_workspace_current_pretty_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("pretty_workspace", |d| {
                d.rafaeltab_workspace("pretty_ws", "Pretty Workspace", |w| {
                    w.tag("test");
                    w.tag("rust");
                    w.tag("cli");
                });
            });
        });
    })
    .create();

    // Get the path to the workspace directory
    let workspace_path = env.root_path().join("pretty_workspace");

    // Run workspace current with default pretty output (no --json flag)
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "current"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace current command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify the pretty output format contains expected elements
    assert!(
        result.stdout.contains("Pretty Workspace"),
        "Pretty output should contain workspace name 'Pretty Workspace'. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("pretty_ws"),
        "Pretty output should contain workspace ID 'pretty_ws'. Got: {}",
        result.stdout
    );

    // Verify tags are displayed in pretty format
    assert!(
        result.stdout.contains("test"),
        "Pretty output should contain tag 'test'. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("rust"),
        "Pretty output should contain tag 'rust'. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("cli"),
        "Pretty output should contain tag 'cli'. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_current_in_subfolder() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("parent_workspace", |d| {
                d.rafaeltab_workspace("parent_ws", "Parent Workspace", |_w| {});
                // Create a subfolder inside the workspace
                d.dir("src", |_src| {});
                d.dir("tests", |_tests| {});
            });
        });
    })
    .create();

    // Get the path to a subfolder of the workspace
    let subfolder_path = env.root_path().join("parent_workspace/src");

    // Run workspace current from within the subfolder
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&subfolder_path)
        .args(&["workspace", "current"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace current command should succeed when in subfolder.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify the workspace is found even when in a subfolder
    assert!(
        result.stdout.contains("Parent Workspace"),
        "Expected 'Parent Workspace' in output when in subfolder. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("parent_ws"),
        "Expected workspace ID 'parent_ws' in output when in subfolder. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_current_with_expanded_path() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        // Note: We can't easily test with ~ in a test environment,
        // so we'll verify the normal path expansion works
        root.test_dir(|td| {
            td.dir("expanded_workspace", |d| {
                d.rafaeltab_workspace("expanded_ws", "Expanded Path Workspace", |w| {
                    w.tag("path-test");
                });
            });
        });
    })
    .create();

    // Get the path to the workspace directory using absolute path
    let workspace_path = env.root_path().join("expanded_workspace");

    // Verify the path is absolute (no ~ in it)
    assert!(
        workspace_path.is_absolute(),
        "Workspace path should be absolute for this test"
    );

    // Run workspace current from within the workspace directory
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "current"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace current command should succeed with expanded path.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify the workspace is found with the expanded path
    assert!(
        result.stdout.contains("Expanded Path Workspace"),
        "Expected 'Expanded Path Workspace' in output. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("expanded_ws"),
        "Expected workspace ID 'expanded_ws' in output. Got: {}",
        result.stdout
    );

    // Verify the JSON output contains the correct absolute path
    let json_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "current", "--json"])
        .build();
    let json_result = env.testers().cmd().run(&json_cmd);

    assert!(
        json_result.success,
        "workspace current --json command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        json_result.stdout, json_result.stderr
    );

    // Parse JSON and verify path is present and absolute
    let json_output: serde_json::Value =
        serde_json::from_str(&json_result.stdout).expect("Output should be valid JSON");

    let root_path = json_output["root"]
        .as_str()
        .expect("root should be a string");
    let root_path_obj = std::path::Path::new(root_path);

    assert!(
        root_path_obj.is_absolute(),
        "Root path in JSON output should be absolute. Got: {}",
        root_path
    );
}
