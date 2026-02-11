mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabRootMixin},
    CliCommandBuilder,
};
use std::fs;
use test_descriptors::testers::{Key, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// Helper function to read and parse the rafaeltab config
fn read_rafaeltab_config(env: &TestEnvironment) -> serde_json::Value {
    let config_path = env
        .context()
        .config_path()
        .expect("Config path should be set");
    let content = fs::read_to_string(&config_path).expect("Failed to read config file");
    serde_json::from_str(&content).expect("Failed to parse config JSON")
}

// =============================================================================
// Add Workspace Command Tests (AW-001 to AW-012)
// =============================================================================

/// AW-001: Full Flow - Happy Path
///
/// Setup:
/// - One existing workspace in storage: "existing-project" with tags: ["rust", "typescript", "go"]
/// - These workspace tags should be used for tag suggestions
///
/// Flow:
/// 1. Command Palette Display
/// 2. Select Add Workspace
/// 3. Name Input (No Suggestions)
/// 4. Tags Input (With Suggestions)
/// 5. Confirmation Display
/// 6. Confirm Creation
/// 7. Command Completes
#[test]
fn test_add_workspace_happy_path() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        // Create existing workspace with tags for suggestions
        root.test_dir(|td| {
            td.dir("existing-project", |d| {
                d.rafaeltab_workspace("existing-project", "existing-project", |w| {
                    w.tag("rust");
                    w.tag("typescript");
                    w.tag("go");
                });
            });
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Step 1: Command Palette Display - verify "Add Workspace" visible
    asserter.find_text("Add Workspace").assert_visible();
    asserter
        .find_text("Create a workspace in the current directory")
        .assert_visible();

    // Step 2: Select Add Workspace
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Step 3: Name Input (No Suggestions)
    asserter.find_text("Workspace name:").assert_visible();
    // Middle panel should be empty (no suggestions for name input)
    asserter.type_text("my-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Step 4: Tags Input (With Suggestions)
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();

    // Type "rus" to trigger suggestions
    asserter.type_text("rus");
    asserter.wait_for_settle();
    // Suggestion from existing workspace should appear
    asserter.find_text("rust").assert_visible();

    // Press Tab to complete "rust"
    asserter.press_key(Key::Tab);
    asserter.wait_for_settle();

    // Type ", ty" for next tag
    asserter.type_text(", ty");
    asserter.wait_for_settle();
    // Suggestion for "typescript" should appear
    asserter.find_text("typescript").assert_visible();

    // Press Tab to complete "typescript"
    asserter.press_key(Key::Tab);
    asserter.wait_for_settle();

    // Confirm tags input
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Step 5: Confirmation Display
    asserter
        .find_text("Create this workspace?")
        .assert_visible();
    asserter.find_text("my-project").assert_visible();
    asserter.find_text("rust").assert_visible();
    asserter.find_text("typescript").assert_visible();
    // Default should be Yes
    let yes = asserter.find_text("Yes");
    yes.fg.assert_not_grayscale();

    // Step 6: Confirm Creation
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Step 7: Command Completes
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0, "Command should complete successfully");

    // Verify workspace created in config
    let config = read_rafaeltab_config(&env);
    let workspaces = config["workspaces"]
        .as_array()
        .expect("workspaces should be an array");
    let workspace = workspaces.iter().find(|w| w["name"] == "my-project");
    assert!(workspace.is_some(), "Workspace should be created");
    let workspace = workspace.unwrap();
    assert_eq!(
        workspace["id"], "my-project",
        "ID should be slugified from name"
    );

    let tags = workspace["tags"]
        .as_array()
        .expect("tags should be an array");
    let tag_strings: Vec<String> = tags
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();
    assert!(
        tag_strings.contains(&"rust".to_string()),
        "Should have rust tag"
    );
    assert!(
        tag_strings.contains(&"typescript".to_string()),
        "Should have typescript tag"
    );
}

/// AW-002: Cancel at Name Input
/// Given at name input step
/// When user presses Escape
/// Then command should exit
/// And no workspace should be added (verify config unchanged)
#[test]
fn test_add_workspace_cancel_at_name_input() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("existing-project", |d| {
                d.rafaeltab_workspace("existing-project", "existing-project", |w| {
                    w.tag("rust");
                });
            });
        });
    })
    .create();

    // Get initial workspace count
    let initial_config = read_rafaeltab_config(&env);
    let initial_count = initial_config["workspaces"].as_array().unwrap().len();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // At name input step, type something then cancel
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("my-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Esc);

    let _ = asserter.expect_completion();

    // Verify no workspace was added
    let final_config = read_rafaeltab_config(&env);
    let final_count = final_config["workspaces"].as_array().unwrap().len();
    assert_eq!(final_count, initial_count, "No workspace should be added");
}

/// AW-003: Cancel at Tags Input
/// Given at tags input step
/// When user presses Escape
/// Then command should exit
/// And no workspace should be added
#[test]
fn test_add_workspace_cancel_at_tags_input() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("existing-project", |d| {
                d.rafaeltab_workspace("existing-project", "existing-project", |w| {
                    w.tag("rust");
                });
            });
        });
    })
    .create();

    // Get initial workspace count
    let initial_config = read_rafaeltab_config(&env);
    let initial_count = initial_config["workspaces"].as_array().unwrap().len();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Complete name input
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("my-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // At tags input step, type something then cancel
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.type_text("rust");
    asserter.wait_for_settle();
    asserter.press_key(Key::Esc);

    let _ = asserter.expect_completion();

    // Verify no workspace was added
    let final_config = read_rafaeltab_config(&env);
    let final_count = final_config["workspaces"].as_array().unwrap().len();
    assert_eq!(final_count, initial_count, "No workspace should be added");
}

/// AW-004: Cancel at Confirmation
/// Given at confirmation step
/// When user selects "No"
/// Then command should exit
/// And no workspace should be added
#[test]
fn test_add_workspace_cancel_at_confirmation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("existing-project", |d| {
                d.rafaeltab_workspace("existing-project", "existing-project", |w| {
                    w.tag("rust");
                });
            });
        });
    })
    .create();

    // Get initial workspace count
    let initial_config = read_rafaeltab_config(&env);
    let initial_count = initial_config["workspaces"].as_array().unwrap().len();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Complete name input
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("my-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Complete tags input (empty)
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // At confirmation step, select "No"
    asserter
        .find_text("Create this workspace?")
        .assert_visible();
    // Navigate to "No"
    asserter.press_key(Key::Right);
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);

    let _ = asserter.expect_completion();

    // Verify no workspace was added
    let final_config = read_rafaeltab_config(&env);
    let final_count = final_config["workspaces"].as_array().unwrap().len();
    assert_eq!(final_count, initial_count, "No workspace should be added");
}

/// AW-005: Empty Name Validation
/// Given at name input
/// When user tries to enter empty string and press Enter
/// Then input should be rejected (or handled gracefully)
/// And user should remain at name input
#[test]
fn test_add_workspace_empty_name_validation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // At name input, try to submit empty name
    asserter.find_text("Workspace name:").assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Should still be at name input prompt
    asserter.find_text("Workspace name:").assert_visible();

    // Now type a valid name and continue
    asserter.type_text("valid-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Should now be at tags input
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
}

/// AW-006: Special Characters in Name
/// Given at name input
/// When user types "My Project!@#"
/// And completes the flow
/// Then ID in config should be "my-project" (slugified)
#[test]
fn test_add_workspace_special_characters_slugified() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter name with special characters
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("My Project!@#");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Skip tags
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Confirm creation
    asserter
        .find_text("Create this workspace?")
        .assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    let _ = asserter.expect_completion();

    // Verify workspace created with slugified ID
    let config = read_rafaeltab_config(&env);
    let workspaces = config["workspaces"]
        .as_array()
        .expect("workspaces should be an array");
    let workspace = workspaces.iter().find(|w| w["name"] == "My Project!@#");
    assert!(workspace.is_some(), "Workspace should be created");
    assert_eq!(
        workspace.unwrap()["id"],
        "my-project",
        "ID should be slugified"
    );
}

/// AW-007: Duplicate Tag Handling
/// Given user types "rust, rust, typescript" in tags input
/// When at confirmation step
/// Then tags in config should be deduplicated to ["rust", "typescript"]
#[test]
fn test_add_workspace_duplicate_tag_handling() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter name
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("my-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter tags with duplicates
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.type_text("rust, rust, typescript");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Confirm creation
    asserter
        .find_text("Create this workspace?")
        .assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    let _ = asserter.expect_completion();

    // Verify tags are deduplicated
    let config = read_rafaeltab_config(&env);
    let workspaces = config["workspaces"]
        .as_array()
        .expect("workspaces should be an array");
    let workspace = workspaces
        .iter()
        .find(|w| w["name"] == "my-project")
        .unwrap();
    let tags = workspace["tags"]
        .as_array()
        .expect("tags should be an array");
    assert_eq!(tags.len(), 2, "Should have 2 unique tags");
    let tag_strings: Vec<String> = tags
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();
    assert!(tag_strings.contains(&"rust".to_string()));
    assert!(tag_strings.contains(&"typescript".to_string()));
}

/// AW-008: Empty Tags
/// Given at tags input
/// When user presses Enter with empty input
/// Then workspace should be created with empty tags list in config
#[test]
fn test_add_workspace_empty_tags() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter name
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("my-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Skip tags (empty input)
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Confirm creation
    asserter
        .find_text("Create this workspace?")
        .assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    let _ = asserter.expect_completion();

    // Verify workspace created with empty tags
    let config = read_rafaeltab_config(&env);
    let workspaces = config["workspaces"]
        .as_array()
        .expect("workspaces should be an array");
    let workspace = workspaces
        .iter()
        .find(|w| w["name"] == "my-project")
        .unwrap();
    let tags = workspace["tags"]
        .as_array()
        .expect("tags should be an array");
    assert!(tags.is_empty(), "Tags should be empty");
}

/// AW-009: Tag Suggestion Based on Existing Workspaces
///
/// Setup:
/// - Existing workspace: "web-project" with tags: ["python", "django"]
///
/// Flow:
/// - Run palette, select "add workspace"
/// - At tags input, user types "dj"
/// - Suggestions should include "django" from existing workspace's tags
#[test]
fn test_add_workspace_tag_suggestion_from_existing() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("web-project", |d| {
                d.rafaeltab_workspace("web-project", "web-project", |w| {
                    w.tag("python");
                    w.tag("django");
                });
            });
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter name
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("new-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // At tags input, type "dj" to trigger suggestions
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.type_text("dj");
    asserter.wait_for_settle();

    // Verify "django" from existing workspace appears in suggestions
    asserter.find_text("django").assert_visible();
}

/// AW-010: Tag Suggestion Partial Match
///
/// Setup:
/// - Existing workspaces:
///   - "backend" with tags: ["rust", "ruby"]
///   - "frontend" with tags: ["react", "typescript"]
///
/// Flow:
/// - At tags input, user types "ru"
/// - Suggestions should include "rust", "ruby" (merged from both workspaces)
/// - User types "rus"
/// - Suggestions should only include "rust"
#[test]
fn test_add_workspace_tag_suggestion_partial_match() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("backend", |d| {
                d.rafaeltab_workspace("backend", "backend", |w| {
                    w.tag("rust");
                    w.tag("ruby");
                });
            });
            td.dir("frontend", |d| {
                d.rafaeltab_workspace("frontend", "frontend", |w| {
                    w.tag("react");
                    w.tag("typescript");
                });
            });
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter name
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("new-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // At tags input, type "ru"
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.type_text("ru");
    asserter.wait_for_settle();

    // Both "rust" and "ruby" should be visible
    asserter.find_text("rust").assert_visible();
    asserter.find_text("ruby").assert_visible();

    // Clear and type "rus"
    asserter.press_key(Key::Ctrl('u'));
    asserter.type_text("rus");
    asserter.wait_for_settle();

    // Only "rust" should be visible now
    asserter.find_text("rust").assert_visible();
    asserter.find_text("ruby").assert_not_visible();
}

/// AW-011: Case Insensitive Tag Matching
///
/// Setup:
/// - Existing workspace: "ProjectA" with tags: ["Rust", "TypeScript"]
///
/// Flow:
/// - At tags input, user types "rust"
/// - Suggestions should include "Rust" (case-insensitive match from workspace)
#[test]
fn test_add_workspace_tag_suggestion_case_insensitive() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("ProjectA", |d| {
                d.rafaeltab_workspace("ProjectA", "ProjectA", |w| {
                    w.tag("Rust");
                    w.tag("TypeScript");
                });
            });
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter name
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("new-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // At tags input, type "rust" (lowercase)
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.type_text("rust");
    asserter.wait_for_settle();

    // "Rust" with capital R should be visible (case-insensitive match)
    asserter.find_text("Rust").assert_visible();
}

/// AW-012: Multi-word Tag Input
/// Given at tags input
/// When user types "web framework, rust, cli"
/// Then after Enter, tags in config should be parsed as: ["web framework", "rust", "cli"]
#[test]
fn test_add_workspace_multi_word_tag_input() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TERM", "xterm-256color")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "add workspace" from palette
    asserter.type_text("add workspace");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter name
    asserter.find_text("Workspace name:").assert_visible();
    asserter.type_text("my-project");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter multi-word tags
    asserter
        .find_text("Tags (comma-separated):")
        .assert_visible();
    asserter.type_text("web framework, rust, cli");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Confirm creation
    asserter
        .find_text("Create this workspace?")
        .assert_visible();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    let _ = asserter.expect_completion();

    // Verify tags parsed correctly
    let config = read_rafaeltab_config(&env);
    let workspaces = config["workspaces"]
        .as_array()
        .expect("workspaces should be an array");
    let workspace = workspaces
        .iter()
        .find(|w| w["name"] == "my-project")
        .unwrap();
    let tags = workspace["tags"]
        .as_array()
        .expect("tags should be an array");
    assert_eq!(tags.len(), 3, "Should have 3 tags");
    let tag_strings: Vec<String> = tags
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();
    assert!(
        tag_strings.contains(&"web framework".to_string()),
        "Should have 'web framework'"
    );
    assert!(
        tag_strings.contains(&"rust".to_string()),
        "Should have 'rust'"
    );
    assert!(
        tag_strings.contains(&"cli".to_string()),
        "Should have 'cli'"
    );
}
