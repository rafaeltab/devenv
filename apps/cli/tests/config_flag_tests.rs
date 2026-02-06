mod common;

use common::CliCommandBuilder;
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

/// Test that the --config flag actually uses the specified config file
#[test]
pub fn test_config_flag_uses_specified_file() {
    let env = TestEnvironment::describe(|_root| {}).create();

    let config_path = env.root_path().join("config.json");

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

    std::fs::write(&config_path, input).expect("Failed to write config");

    let cmd = CliCommandBuilder::new()
        .with_config(&config_path)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Verify the output contains the unique workspace ID from our temp config
    assert!(
        result.stdout.contains("test_workspace_unique_123"),
        "Output should contain the workspace ID from the specified config file.\nGot: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("Test Workspace Unique"),
        "Output should contain the workspace name from the specified config file.\nGot: {}",
        result.stdout
    );
}

/// Test that --config flag isolates from the home config file
#[test]
pub fn test_config_flag_isolates_from_home_config() {
    let env = TestEnvironment::describe(|_root| {}).create();

    let config_path = env.root_path().join("config.json");

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

    std::fs::write(&config_path, input).expect("Failed to write config");

    let cmd = CliCommandBuilder::new()
        .with_config(&config_path)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Verify we only see the isolated workspace
    assert!(
        result.stdout.contains("isolated_workspace"),
        "Output should contain the isolated workspace.\nGot: {}",
        result.stdout
    );

    // The output should be a single line (plus newline) for our one workspace
    let lines: Vec<&str> = result.stdout.lines().collect();
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
    let env_a = TestEnvironment::describe(|_root| {}).create();
    let config_path_a = env_a.root_path().join("config.json");
    std::fs::write(&config_path_a, config_a).expect("Failed to write config A");
    let cmd_a = CliCommandBuilder::new()
        .with_config(&config_path_a)
        .args(&["workspace", "list"])
        .build();
    let result_a = env_a.testers().cmd().run(&cmd_a);
    assert!(
        result_a.stdout.contains("config_a"),
        "Config A should show config_a"
    );
    assert!(
        !result_a.stdout.contains("config_b"),
        "Config A should not show config_b"
    );
    assert!(
        !result_a.stdout.contains("config_c"),
        "Config A should not show config_c"
    );

    // Test config B
    let env_b = TestEnvironment::describe(|_root| {}).create();
    let config_path_b = env_b.root_path().join("config.json");
    std::fs::write(&config_path_b, config_b).expect("Failed to write config B");
    let cmd_b = CliCommandBuilder::new()
        .with_config(&config_path_b)
        .args(&["workspace", "list"])
        .build();
    let result_b = env_b.testers().cmd().run(&cmd_b);
    assert!(
        !result_b.stdout.contains("config_a"),
        "Config B should not show config_a"
    );
    assert!(
        result_b.stdout.contains("config_b"),
        "Config B should show config_b"
    );
    assert!(
        !result_b.stdout.contains("config_c"),
        "Config B should not show config_c"
    );

    // Test config C
    let env_c = TestEnvironment::describe(|_root| {}).create();
    let config_path_c = env_c.root_path().join("config.json");
    std::fs::write(&config_path_c, config_c).expect("Failed to write config C");
    let cmd_c = CliCommandBuilder::new()
        .with_config(&config_path_c)
        .args(&["workspace", "list"])
        .build();
    let result_c = env_c.testers().cmd().run(&cmd_c);
    assert!(
        !result_c.stdout.contains("config_a"),
        "Config C should not show config_a"
    );
    assert!(
        !result_c.stdout.contains("config_b"),
        "Config C should not show config_b"
    );
    assert!(
        result_c.stdout.contains("config_c"),
        "Config C should show config_c"
    );
}
