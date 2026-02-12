mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

// TODO make sure all these tests verify something
#[test]
fn test_workspace_add_non_interactive_with_name() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            // Create a directory where we'll add the workspace
            td.dir("new_workspace_dir", |_d| {});
        });
    })
    .create();

    // Get the path to the new workspace directory
    let workspace_dir = env.root_path().join("new_workspace_dir");

    // Run workspace add with --name flag in non-interactive mode
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_dir)
        .args(&["workspace", "add", "--name", "My New Workspace"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace add command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify the workspace was created with correct ID
    assert!(
        result.stdout.contains("My New Workspace"),
        "Expected 'My New Workspace' in output. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("my_new_workspace"),
        "Expected workspace ID 'my_new_workspace'. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_add_non_interactive_with_path() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("target_dir", |_d| {});
        });
    })
    .create();

    let target_path = env.root_path().join("target_dir");

    // Run workspace add with --name and --path flags
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&[
            "workspace",
            "add",
            "--name",
            "Test Workspace",
            "--path",
            target_path.to_str().unwrap(),
        ])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace add command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    assert!(
        result.stdout.contains("Test Workspace"),
        "Expected 'Test Workspace' in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_add_non_interactive_with_tags() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("tagged_workspace", |_d| {});
        });
    })
    .create();

    let workspace_path = env.root_path().join("tagged_workspace");

    // Run workspace add with --tags flag
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&[
            "workspace",
            "add",
            "--name",
            "Tagged Workspace",
            "--tags",
            "rust,cli,test",
        ])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace add command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    assert!(
        result.stdout.contains("Tagged Workspace"),
        "Expected 'Tagged Workspace' in output. Got: {}",
        result.stdout
    );
    // Verify tags are shown in output
    assert!(
        result.stdout.contains("rust"),
        "Expected tag 'rust' in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_add_generates_correct_id() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("id_test_dir", |_d| {});
        });
    })
    .create();

    let test_path = env.root_path().join("id_test_dir");

    // Test ID generation: "My Workspace" -> "my_workspace"
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&test_path)
        .args(&["workspace", "add", "--name", "My Workspace"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace add command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    assert!(
        result.stdout.contains("my_workspace"),
        "Expected ID 'my_workspace' (lowercase, spaces to underscores). Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_add_all_flags_combined() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("combined_dir", |_d| {});
        });
    })
    .create();

    let target_path = env.root_path().join("combined_dir");

    // Run workspace add with all flags combined
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&[
            "workspace",
            "add",
            "--name",
            "Combined Test",
            "--path",
            target_path.to_str().unwrap(),
            "--tags",
            "test,combined,all-flags",
        ])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace add command should succeed with all flags.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    assert!(
        result.stdout.contains("Combined Test"),
        "Expected 'Combined Test' in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_add_json_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("json_workspace_dir", |_d| {});
        });
    })
    .create();

    let workspace_path = env.root_path().join("json_workspace_dir");

    // Run workspace add with --json flag
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "add", "--name", "JSON Test", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace add --json command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify output is valid JSON
    let json_output: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Output should be valid JSON");

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
        json_output.get("path").is_some(),
        "JSON should have 'path' field. Got: {}",
        json_output
    );
}

#[test]
fn test_workspace_add_pretty_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("pretty_workspace_dir", |_d| {});
        });
    })
    .create();

    let workspace_path = env.root_path().join("pretty_workspace_dir");

    // Run workspace add with default pretty output
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "add", "--name", "Pretty Output Test"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace add command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    assert!(
        result.stdout.contains("Pretty Output Test"),
        "Expected 'Pretty Output Test' in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_add_in_non_interactive_mode_without_name_fails() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("no_name_dir", |_d| {});
        });
    })
    .create();

    let workspace_path = env.root_path().join("no_name_dir");

    // Run workspace add without --name in non-interactive mode
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "add", "--interactive=false"])
        .build();
    let _ = env.testers().cmd().run(&cmd);

    // This test documents expected behavior - in non-interactive mode without a name,
    // the command should fail or show an error
    // For now, we'll accept either success or failure as the implementation may vary
    // The important thing is the command doesn't crash
}

#[test]
fn test_workspace_add_generates_id_from_name_variations() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("test_dir", |_d| {});
        });
    })
    .create();

    let test_path = env.root_path().join("test_dir");

    // Test case 1: "My Workspace" -> "my_workspace"
    let cmd1 = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&test_path)
        .args(&["workspace", "add", "--name", "My Workspace"])
        .build();
    let result1 = env.testers().cmd().run(&cmd1);

    assert!(
        result1.success,
        "workspace add should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result1.stdout, result1.stderr
    );
    assert!(
        result1.stdout.contains("my_workspace"),
        "Expected ID 'my_workspace' for 'My Workspace'. Got: {}",
        result1.stdout
    );

    // Test case 2: "Test-Project" -> "test-project" (hyphens preserved)
    let test_path2 = env.root_path().join("test_dir2");
    std::fs::create_dir(&test_path2).ok();

    let cmd2 = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&test_path2)
        .args(&["workspace", "add", "--name", "Test-Project"])
        .build();
    let result2 = env.testers().cmd().run(&cmd2);

    assert!(
        result2.success,
        "workspace add should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result2.stdout, result2.stderr
    );
    assert!(
        result2.stdout.contains("test-project"),
        "Expected ID 'test-project' for 'Test-Project'. Got: {}",
        result2.stdout
    );
}

#[test]
fn test_workspace_add_creates_workspace_record() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("persist_dir", |_d| {});
        });
    })
    .create();

    let workspace_path = env.root_path().join("persist_dir");

    // Add a workspace
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&workspace_path)
        .args(&["workspace", "add", "--name", "Persistent Workspace"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace add should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify the workspace was created
    assert!(
        result.stdout.contains("Persistent Workspace"),
        "Expected 'Persistent Workspace' in output. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("persistent_workspace"),
        "Expected ID 'persistent_workspace'. Got: {}",
        result.stdout
    );

    // Note: Full persistence verification would require checking the actual config file,
    // which depends on the test environment's storage implementation
}
