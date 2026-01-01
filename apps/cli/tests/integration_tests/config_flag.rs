use super::helpers;

/// Test that the --config flag actually uses the specified config file
#[test]
pub fn test_config_flag_uses_specified_file() {
    let input = r#"{
  "workspaces": [
    {
      "root": "~/test/path",
      "id": "test_workspace_unique_123",
      "name": "Test Workspace Unique",
      "tags": ["test"]
    }
  ],
  "tmux": {
    "sessions": [],
    "defaultWindows": []
  }
}
        "#;

    let test_ctx = helpers::TestContext::new(input).expect("Failed to create test config");
    let (stdout, _stderr) =
        helpers::run_cli_with_stdin(&["workspace", "list"], "", test_ctx.config_path());

    // Verify the output contains the unique workspace ID from our temp config
    assert!(
        stdout.contains("test_workspace_unique_123"),
        "Output should contain the workspace ID from the specified config file.\nGot: {}",
        stdout
    );
    assert!(
        stdout.contains("Test Workspace Unique"),
        "Output should contain the workspace name from the specified config file.\nGot: {}",
        stdout
    );
}

/// Test that --config flag isolates from the home config file
#[test]
pub fn test_config_flag_isolates_from_home_config() {
    let input = r#"{
  "workspaces": [
    {
      "root": "~/isolated/path",
      "id": "isolated_workspace",
      "name": "Isolated Workspace",
      "tags": ["isolated"]
    }
  ],
  "tmux": {
    "sessions": [],
    "defaultWindows": []
  }
}
        "#;

    let test_ctx = helpers::TestContext::new(input).expect("Failed to create test config");
    let (stdout, _stderr) =
        helpers::run_cli_with_stdin(&["workspace", "list"], "", test_ctx.config_path());

    // Verify we only see the isolated workspace
    assert!(
        stdout.contains("isolated_workspace"),
        "Output should contain the isolated workspace.\nGot: {}",
        stdout
    );

    // The output should be a single line (plus newline) for our one workspace
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "Should only have one workspace in output (not loading from home config).\nGot {} lines: {:?}",
        lines.len(),
        lines
    );
}

/// Test that multiple tests with different configs don't cross-contaminate
#[test]
pub fn test_multiple_configs_no_crosstalk() {
    // Create three different configs with unique IDs
    let config_a = r#"{
  "workspaces": [
    {
      "root": "~/path/a",
      "id": "config_a",
      "name": "Config A",
      "tags": []
    }
  ],
  "tmux": {
    "sessions": [],
    "defaultWindows": []
  }
}
        "#;

    let config_b = r#"{
  "workspaces": [
    {
      "root": "~/path/b",
      "id": "config_b",
      "name": "Config B",
      "tags": []
    }
  ],
  "tmux": {
    "sessions": [],
    "defaultWindows": []
  }
}
        "#;

    let config_c = r#"{
  "workspaces": [
    {
      "root": "~/path/c",
      "id": "config_c",
      "name": "Config C",
      "tags": []
    }
  ],
  "tmux": {
    "sessions": [],
    "defaultWindows": []
  }
}
        "#;

    // Test config A
    let test_ctx_a = helpers::TestContext::new(config_a).expect("Failed to create test config A");
    let (stdout_a, _) =
        helpers::run_cli_with_stdin(&["workspace", "list"], "", test_ctx_a.config_path());
    assert!(
        stdout_a.contains("config_a"),
        "Config A should show config_a"
    );
    assert!(
        !stdout_a.contains("config_b"),
        "Config A should not show config_b"
    );
    assert!(
        !stdout_a.contains("config_c"),
        "Config A should not show config_c"
    );

    // Test config B
    let test_ctx_b = helpers::TestContext::new(config_b).expect("Failed to create test config B");
    let (stdout_b, _) =
        helpers::run_cli_with_stdin(&["workspace", "list"], "", test_ctx_b.config_path());
    assert!(
        !stdout_b.contains("config_a"),
        "Config B should not show config_a"
    );
    assert!(
        stdout_b.contains("config_b"),
        "Config B should show config_b"
    );
    assert!(
        !stdout_b.contains("config_c"),
        "Config B should not show config_c"
    );

    // Test config C
    let test_ctx_c = helpers::TestContext::new(config_c).expect("Failed to create test config C");
    let (stdout_c, _) =
        helpers::run_cli_with_stdin(&["workspace", "list"], "", test_ctx_c.config_path());
    assert!(
        !stdout_c.contains("config_a"),
        "Config C should not show config_a"
    );
    assert!(
        !stdout_c.contains("config_b"),
        "Config C should not show config_b"
    );
    assert!(
        stdout_c.contains("config_c"),
        "Config C should show config_c"
    );
}
