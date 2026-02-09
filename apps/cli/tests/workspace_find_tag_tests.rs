mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_workspace_find_tag_single_match() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws_one", "Workspace One", |w| {
                    w.tag("rust");
                });
            });
            td.dir("ws2", |d| {
                d.rafaeltab_workspace("ws_two", "Workspace Two", |w| {
                    w.tag("python");
                });
            });
        });
    })
    .create();

    // Find workspaces with "rust" tag
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "rust"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify only the workspace with "rust" tag is found
    assert!(
        result.stdout.contains("Workspace One"),
        "Expected 'Workspace One' with 'rust' tag. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("ws_one"),
        "Expected workspace ID 'ws_one'. Got: {}",
        result.stdout
    );

    // Verify workspace without the tag is not shown
    assert!(
        !result.stdout.contains("Workspace Two"),
        "Should not contain 'Workspace Two' (has 'python' tag). Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_find_tag_multiple_matches() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws_alpha", "Workspace Alpha", |w| {
                    w.tag("shared");
                });
            });
            td.dir("ws2", |d| {
                d.rafaeltab_workspace("ws_beta", "Workspace Beta", |w| {
                    w.tag("shared");
                });
            });
            td.dir("ws3", |d| {
                d.rafaeltab_workspace("ws_gamma", "Workspace Gamma", |w| {
                    w.tag("unique");
                });
            });
        });
    })
    .create();

    // Find workspaces with "shared" tag - should return multiple results
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "shared"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify both workspaces with "shared" tag are found
    assert!(
        result.stdout.contains("Workspace Alpha"),
        "Expected 'Workspace Alpha' with 'shared' tag. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("ws_alpha"),
        "Expected workspace ID 'ws_alpha'. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("Workspace Beta"),
        "Expected 'Workspace Beta' with 'shared' tag. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("ws_beta"),
        "Expected workspace ID 'ws_beta'. Got: {}",
        result.stdout
    );

    // Verify workspace without the "shared" tag is not shown
    assert!(
        !result.stdout.contains("Workspace Gamma"),
        "Should not contain 'Workspace Gamma' (has 'unique' tag). Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_find_tag_no_matches() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws_one", "Workspace One", |w| {
                    w.tag("existing");
                });
            });
        });
    })
    .create();

    // Find workspaces with a non-existent tag
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "nonexistent"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace find-tag command should succeed even with no matches.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify empty output when no workspaces have the tag
    assert!(
        result.stdout.trim().is_empty(),
        "Expected empty output when no workspaces have the tag. Got: {:?}",
        result.stdout
    );
}

#[test]
fn test_workspace_find_tag_case_sensitive() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws_one", "Workspace One", |w| {
                    w.tag("Rust");
                });
            });
        });
    })
    .create();

    // Search with exact case "Rust"
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "Rust"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // With exact case, should find the workspace
    assert!(
        result.stdout.contains("Workspace One"),
        "Expected 'Workspace One' with exact case 'Rust' tag. Got: {}",
        result.stdout
    );

    // Search with different case "rust"
    let cmd2 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "rust"])
        .build();
    let result2 = env.testers().cmd().run(&cmd2);

    assert!(
        result2.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result2.stdout, result2.stderr
    );

    // With lowercase, should NOT find the workspace (case-sensitive)
    assert!(
        result2.stdout.trim().is_empty(),
        "Expected empty output with lowercase 'rust' (case-sensitive). Got: {:?}",
        result2.stdout
    );
}

#[test]
fn test_workspace_find_tag_multiple_tags_filter() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws_rust", "Rust Workspace", |w| {
                    w.tag("rust");
                });
            });
            td.dir("ws2", |d| {
                d.rafaeltab_workspace("ws_python", "Python Workspace", |w| {
                    w.tag("python");
                });
            });
            td.dir("ws3", |d| {
                d.rafaeltab_workspace("ws_js", "JavaScript Workspace", |w| {
                    w.tag("javascript");
                });
            });
        });
    })
    .create();

    // Test filtering by "rust" tag
    let cmd1 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "rust"])
        .build();
    let result1 = env.testers().cmd().run(&cmd1);

    assert!(
        result1.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result1.stdout, result1.stderr
    );
    assert!(
        result1.stdout.contains("Rust Workspace"),
        "Expected 'Rust Workspace'. Got: {}",
        result1.stdout
    );
    assert!(
        !result1.stdout.contains("Python Workspace"),
        "Should not contain 'Python Workspace'. Got: {}",
        result1.stdout
    );

    // Test filtering by "python" tag
    let cmd2 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "python"])
        .build();
    let result2 = env.testers().cmd().run(&cmd2);

    assert!(
        result2.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result2.stdout, result2.stderr
    );
    assert!(
        result2.stdout.contains("Python Workspace"),
        "Expected 'Python Workspace'. Got: {}",
        result2.stdout
    );
    assert!(
        !result2.stdout.contains("JavaScript Workspace"),
        "Should not contain 'JavaScript Workspace'. Got: {}",
        result2.stdout
    );
}

#[test]
fn test_workspace_find_tag_workspace_with_multiple_tags() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws_multi", "Multi-Tag Workspace", |w| {
                    w.tag("rust");
                    w.tag("cli");
                    w.tag("dev");
                });
            });
            td.dir("ws2", |d| {
                d.rafaeltab_workspace("ws_single", "Single-Tag Workspace", |w| {
                    w.tag("python");
                });
            });
        });
    })
    .create();

    // Search for "rust" - should find the multi-tag workspace
    let cmd1 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "rust"])
        .build();
    let result1 = env.testers().cmd().run(&cmd1);

    assert!(
        result1.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result1.stdout, result1.stderr
    );
    assert!(
        result1.stdout.contains("Multi-Tag Workspace"),
        "Expected 'Multi-Tag Workspace' (has 'rust' tag). Got: {}",
        result1.stdout
    );

    // Search for "cli" - should also find the multi-tag workspace
    let cmd2 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "cli"])
        .build();
    let result2 = env.testers().cmd().run(&cmd2);

    assert!(
        result2.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result2.stdout, result2.stderr
    );
    assert!(
        result2.stdout.contains("Multi-Tag Workspace"),
        "Expected 'Multi-Tag Workspace' (has 'cli' tag). Got: {}",
        result2.stdout
    );

    // Search for "dev" - should also find the multi-tag workspace
    let cmd3 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "dev"])
        .build();
    let result3 = env.testers().cmd().run(&cmd3);

    assert!(
        result3.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result3.stdout, result3.stderr
    );
    assert!(
        result3.stdout.contains("Multi-Tag Workspace"),
        "Expected 'Multi-Tag Workspace' (has 'dev' tag). Got: {}",
        result3.stdout
    );
}

#[test]
fn test_workspace_find_tag_json_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws_json", "JSON Test Workspace", |w| {
                    w.tag("test");
                    w.tag("json");
                });
            });
        });
    })
    .create();

    // Run with JSON output
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "test", "--json"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace find-tag --json command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify output is valid JSON array
    let json_output: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("Output should be valid JSON");

    assert!(
        json_output.is_array(),
        "JSON output should be an array. Got: {}",
        json_output
    );

    let workspaces = json_output.as_array().unwrap();
    assert_eq!(
        workspaces.len(),
        1,
        "Should have exactly 1 workspace. Got: {}",
        workspaces.len()
    );

    let workspace = &workspaces[0];
    assert!(
        workspace.get("id").is_some(),
        "JSON should have 'id' field. Got: {}",
        workspace
    );
    assert!(
        workspace.get("name").is_some(),
        "JSON should have 'name' field. Got: {}",
        workspace
    );
    assert!(
        workspace.get("root").is_some(),
        "JSON should have 'root' field. Got: {}",
        workspace
    );
    assert!(
        workspace.get("tags").is_some(),
        "JSON should have 'tags' field. Got: {}",
        workspace
    );

    assert_eq!(
        workspace["id"].as_str().unwrap(),
        "ws_json",
        "Workspace ID should be 'ws_json'. Got: {}",
        workspace["id"]
    );
}

#[test]
fn test_workspace_find_tag_pretty_output() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("ws_pretty", "Pretty Workspace", |w| {
                    w.tag("pretty");
                });
            });
        });
    })
    .create();

    // Run with default pretty output
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "pretty"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify pretty output format contains expected elements
    assert!(
        result.stdout.contains("Pretty Workspace"),
        "Pretty output should contain workspace name. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("ws_pretty"),
        "Pretty output should contain workspace ID. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("pretty"),
        "Pretty output should contain the tag. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_find_tag_empty_tag_list() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("shell");
        });

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                // Workspace with no tags (empty tag list by default)
                d.rafaeltab_workspace("ws_no_tags", "No Tags Workspace", |_w| {});
            });
            td.dir("ws2", |d| {
                d.rafaeltab_workspace("ws_with_tags", "With Tags Workspace", |w| {
                    w.tag("existing");
                });
            });
        });
    })
    .create();

    // Search for a tag that doesn't exist - should not find the workspace with no tags
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "nonexistent"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace find-tag command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Should not find the workspace with no tags
    assert!(
        !result.stdout.contains("No Tags Workspace"),
        "Should not find workspace without tags. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.trim().is_empty(),
        "Expected empty output when searching for non-existent tag. Got: {:?}",
        result.stdout
    );
}
