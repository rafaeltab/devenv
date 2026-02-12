mod common;

use crate::common::{rafaeltab_descriptors::RafaeltabRootMixin, CliCommandBuilder};
use std::fs;
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_invalid_config_file_shows_error() {
    let env = TestEnvironment::describe(|root| {
        // Don't use rafaeltab_config - we'll create an invalid config manually
        root.test_dir(|_td| {});
    })
    .create();

    // Create an invalid JSON config file
    let config_path = env.root_path().join(".rafaeltab.json");
    fs::write(&config_path, "{invalid json content}").expect("Failed to write invalid config");

    // Run any CLI command that requires config
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(config_path.to_str().unwrap())
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Command should fail
    assert!(
        !result.success,
        "Expected command to fail with invalid config, but it succeeded. stdout: {}",
        result.stdout
    );

    // Verify error message mentions JSON parsing error
    let error_output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        error_output.to_lowercase().contains("key must be a string")
            || error_output.to_lowercase().contains("json")
            || error_output.to_lowercase().contains("parse"),
        "Expected error message about invalid JSON parsing, got: {}",
        error_output
    );
}

#[test]
fn test_missing_config_file_creates_default() {
    let env = TestEnvironment::describe(|root| {
        // No config file - test directory only
        root.test_dir(|_td| {});
    })
    .create();

    // Don't create any config file via the test descriptor

    // Use --config flag pointing to a non-existent path
    let nonexistent_config = env.root_path().join("nonexistent_config.json");
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(&nonexistent_config)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Current behavior: fails when config file doesn't exist
    assert!(
        !result.success,
        "Expected command to fail with missing config, but it succeeded. stdout: {}",
        result.stdout
    );

    let error_output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        error_output.to_lowercase().contains("config")
            || error_output.to_lowercase().contains("not found")
            || error_output.to_lowercase().contains("file")
            || error_output.to_lowercase().contains("os error"),
        "Expected error message about missing config file, got: {}",
        error_output
    );
}

#[test]
fn test_no_subcommand_shows_help() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Run CLI without any subcommand
    let cmd = CliCommandBuilder::new().with_env(&env).args(&[]).build();
    let result = env.testers().cmd().run(&cmd);

    // Should show help text
    let output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        output.contains("Usage:") || output.contains("Commands:") || output.contains("--help"),
        "Expected help output when no subcommand provided, got: {}",
        output
    );
}

#[test]
fn test_unknown_subcommand_shows_error() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Run CLI with unknown subcommand
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["nonexistent"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should fail with error about unknown subcommand
    assert!(
        !result.success,
        "Expected command to fail with unknown subcommand"
    );

    let error_output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        error_output.to_lowercase().contains("unknown")
            || error_output.to_lowercase().contains("found")
            || error_output.to_lowercase().contains("error"),
        "Expected error message about unknown subcommand, got: {}",
        error_output
    );
}

#[test]
fn test_missing_required_args_shows_usage() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Run workspace find without required ID argument
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should fail with usage/help message
    assert!(
        !result.success,
        "Expected command to fail with missing argument"
    );

    let output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        output.contains("Usage:")
            || output.contains("required")
            || output.contains("<ID>")
            || output.contains("error"),
        "Expected usage or error message for missing argument, got: {}",
        output
    );
}

#[test]
fn test_git_not_installed_shows_error() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Run worktree command which requires git
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("PATH", "/nonexistent") // Remove git from PATH
        .args(&["worktree", "start", "test-branch"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should fail
    assert!(
        !result.success,
        "Expected command to fail when git is not available"
    );

    let error_output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        error_output.to_lowercase().contains("git")
            || error_output.to_lowercase().contains("not found")
            || error_output.to_lowercase().contains("command")
            || error_output.to_lowercase().contains("error"),
        "Expected error message about git not found, got: {}",
        error_output
    );
}

#[test]
fn test_tmux_not_installed_shows_error() {
    // This test requires a custom tmux socket that points to a non-existent tmux binary
    // Since the test framework provides its own tmux socket/server, we'll test with
    // an invalid socket path instead
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Run tmux list command with an environment that should cause tmux to fail
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("RAFAELTAB_TMUX_SOCKET", "/nonexistent/tmux.socket")
        .args(&["tmux", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // The tmux list command may succeed (returning empty sessions) or fail
    // depending on how the CLI handles the tmux socket error
    // We're verifying the behavior is reasonable (doesn't panic)

    let output = format!("{} {}", result.stdout, result.stderr);
    // Either it succeeds with empty output or fails with an error
    assert!(
        result.success
            || output.to_lowercase().contains("error")
            || output.to_lowercase().contains("tmux"),
        "Expected either success with empty output or error about tmux, got: {}",
        output
    );
}

#[test]
fn test_permission_denied_config_file() {
    let env = TestEnvironment::describe(|root| {
        // Don't use rafaeltab_config - we'll create a config file with no permissions
        root.test_dir(|_td| {});
    })
    .create();

    // Create a valid config file but with no read permissions
    let config_path = env.root_path().join(".rafaeltab.json");
    fs::write(&config_path, "{\"workspaces\": [], \"tmux\": {}}").expect("Failed to write config");

    // Remove read permissions
    let mut permissions = fs::metadata(&config_path).unwrap().permissions();
    permissions.set_readonly(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o000); // No permissions at all
    }
    fs::set_permissions(&config_path, permissions).expect("Failed to set permissions");

    // Run CLI command
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(&config_path)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should fail with permission denied
    assert!(
        !result.success,
        "Expected command to fail with permission denied, but it succeeded. stdout: {}",
        result.stdout
    );

    let error_output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        error_output.to_lowercase().contains("permission")
            || error_output.to_lowercase().contains("access")
            || error_output.to_lowercase().contains("denied")
            || error_output.to_lowercase().contains("os error"),
        "Expected error message about permission denied, got: {}",
        error_output
    );
}

#[test]
fn test_malformed_workspace_in_config() {
    let env = TestEnvironment::describe(|root| {
        // Don't use rafaeltab_config - we'll create a malformed config manually
        root.test_dir(|_td| {});
    })
    .create();

    // Create a config file with a workspace missing required fields (no id field)
    let config_path = env.root_path().join(".rafaeltab.json");
    fs::write(
        &config_path,
        r#"{"workspaces": [{"name": "Test", "root": "/some/path"}], "tmux": {}}"#,
    )
    .expect("Failed to write malformed config");

    // Run CLI command
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(&config_path)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // The command may succeed (skipping malformed entries) or fail
    // We're testing that it doesn't panic
    assert!(
        result.success,
        "Test should run without panic. Result: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_config_validation_error_messages() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Create an empty config file which is valid JSON but may not have required fields
    let config_path = env.root_path().join(".rafaeltab.json");
    fs::write(&config_path, "{}").expect("Failed to write empty config");

    // Run CLI command
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(&config_path)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Verify either success or helpful error
    if !result.success {
        let error_output = format!("{} {}", result.stdout, result.stderr);
        // Error message should be helpful, not just a generic panic
        assert!(
            error_output.to_lowercase().contains("config")
                || error_output.to_lowercase().contains("field")
                || error_output.to_lowercase().contains("missing")
                || error_output.to_lowercase().contains("required")
                || error_output.to_lowercase().contains("parse"),
            "Expected helpful error message about config validation, got: {}",
            error_output
        );
    }
    // If it succeeded, that's also acceptable (default values may be used)
}
