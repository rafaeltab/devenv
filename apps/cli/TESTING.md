# Testing Guide

This document describes the testing patterns and infrastructure for the Rafaeltab CLI application.

## Table of Contents

- [Overview](#overview)
- [Test Types](#test-types)
- [Test Descriptor Framework](#test-descriptor-framework)
- [Writing Integration Tests](#writing-integration-tests)
- [Running Tests](#running-tests)
- [Best Practices](#best-practices)

## Overview

The CLI uses a combination of unit tests and integration tests to ensure correctness:

- **Unit Tests**: Located within source files using `#[cfg(test)]` modules
- **Integration Tests**: Located in `tests/` directory, test the CLI as a black box using the test descriptor framework

## Test Types

### Unit Tests

Unit tests are embedded in source files and test individual components in isolation.

**Location**: `src/**/*.rs` in `#[cfg(test)]` modules

**Example**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn mock_storage() -> impl WorkspaceStorage {
        MockWorkspaceStorage { data: vec![] }
    }

    #[test]
    fn test_workspace_repository_returns_all_workspaces() {
        let storage = mock_storage();
        let repo = ImplWorkspaceRepository { workspace_storage: &storage };
        let workspaces = repo.get_workspaces();
        assert_eq!(workspaces.len(), 0);
    }
}
```

**Pattern**:

1. Create factory functions for mock dependencies
2. Instantiate the system under test (SUT)
3. Perform actions
4. Assert expected behavior

### Integration Tests

Integration tests run the CLI binary as a subprocess and verify end-to-end behavior using the **test descriptor framework**.

**Location**: `tests/integration_tests/*.rs` and `tests/*_tests.rs`

**Key Characteristics**:

- Tests run against the compiled binary (`target/debug/rafaeltab`)
- Each test gets isolated configuration, directories, and tmux sessions
- Tests are fully isolated and don't interfere with each other or the user's environment
- Automatic cleanup of all test resources

## Test Descriptor Framework

The test descriptor framework provides a declarative API for setting up complex test environments. It automatically handles:

- Temporary directory creation and cleanup
- Isolated tmux server instances
- Git repository creation with branches and commits
- Config file generation
- Test resource cleanup (even on panic)

### Key Components

#### TestEnvironment

The main entry point for creating test environments:

```rust
use test_descriptors::TestEnvironment;

let env = TestEnvironment::describe(|root| {
    // Configure your test environment here
}).create();

// Environment is automatically cleaned up when `env` is dropped
```

#### Rafaeltab-Specific Descriptors

Located in `tests/common/rafaeltab_descriptors/`, these provide rafaeltab-specific functionality:

- **`RafaeltabRootMixin`**: Adds `rafaeltab_config()` to create config files
- **`RafaeltabDirMixin`**: Adds `rafaeltab_workspace()` to register workspaces at directories
- **`RafaeltabGitMixin`**: Adds `rafaeltab_workspace()` to register workspaces at git repos

## Writing Integration Tests

### Basic Structure

```rust
use crate::common::{rafaeltab_descriptors::RafaeltabRootMixin, run_cli};
use test_descriptors::TestEnvironment;

#[test]
fn test_my_feature() {
    // 1. Create test environment with config
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();  // Optional: add default tmux windows
        });

        root.test_dir(|td| {
            td.dir("my-workspace", |d| {
                d.rafaeltab_workspace("my_ws", "My Workspace", |w| {
                    w.tag("rust");
                });
            });
        });
    }).create();

    let config_path = env.context().config_path().unwrap();

    // 2. Run CLI
    let (stdout, stderr, success) = run_cli(
        &["workspace", "list"],
        config_path.to_str().unwrap()
    );

    // 3. Assert results
    assert!(success, "Command should succeed.\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);
    assert!(stdout.contains("my_ws"));
}
```

### Testing with Workspaces

```rust
use crate::common::rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabRootMixin};
use test_descriptors::TestEnvironment;

#[test]
fn test_workspace_creation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("projects", |d| {
                d.dir("project-a", |d| {
                    d.rafaeltab_workspace("project_a", "Project A", |w| {
                        w.tag("rust");
                        w.tag("cli");
                    });
                });
                d.dir("project-b", |d| {
                    d.rafaeltab_workspace("project_b", "Project B", |w| {
                        w.tag("javascript");
                    });
                });
            });
        });
    }).create();

    // Both workspaces are now registered in the config
    let config_path = env.context().config_path().unwrap();

    // Test your CLI commands...
}
```

### Testing with Git Repositories

```rust
use crate::common::rafaeltab_descriptors::{RafaeltabGitMixin, RafaeltabRootMixin};
use test_descriptors::TestEnvironment;

#[test]
fn test_workspace_with_git() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("my-project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# My Project");
                        });
                    });
                    // Register the git repo as a workspace
                    g.rafaeltab_workspace("my_project", "My Project", |_w| {});
                });
            });
        });
    }).create();

    // Verify git repo exists
    assert!(env.root_path().join("my-project/repo/.git").exists());
}
```

### Testing Tmux Commands

Tmux tests are automatically isolated using unique socket names:

```rust
use crate::common::{rafaeltab_descriptors::RafaeltabRootMixin, run_cli_with_tmux};
use test_descriptors::TestEnvironment;

#[test]
fn test_tmux_start() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws_1", |_d| {});
        });
    }).create();

    let config_path = env.context().config_path().unwrap();

    // Create a config with tmux session
    let config = format!(
        r#"{{
        "workspaces": [{{
            "id": "ws_1",
            "name": "test ws",
            "root": "{}",
            "tags": []
        }}],
        "tmux": {{
            "sessions": [{{
                "workspace": "ws_1",
                "name": "test-ws",
                "windows": [{{ "name": "shell" }}]
            }}],
            "defaultWindows": []
        }}
    }}"#,
        env.root_path().join("ws_1").display()
    );

    std::fs::write(&config_path, config).expect("Failed to write config");

    // Run CLI with isolated tmux socket
    let (stdout, stderr, success) = run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket()
    );

    assert!(success, "Command failed:\nstdout: {}\nstderr: {}", stdout, stderr);

    // Verify session exists in isolated tmux server
    assert!(env.tmux().session_exists("test ws"));
}
```

### Testing with Worktree Configuration

```rust
use crate::common::rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabRootMixin};
use test_descriptors::TestEnvironment;

#[test]
fn test_workspace_with_worktree() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("worktree-project", |d| {
                d.rafaeltab_workspace("worktree_project", "Worktree Project", |w| {
                    w.worktree(&["npm install", "npm run build"], &[".env", "node_modules"]);
                });
            });
        });
    }).create();

    // Verify worktree config is in the config file
    let config_path = env.context().config_path().unwrap();
    let config_content = std::fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("onCreate"));
    assert!(config_content.contains("npm install"));
}
```

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Integration Tests Only

```bash
cargo test --test integration_test
```

### Run Descriptor Tests

```bash
cargo test --test descriptor_tests
cargo test --test cli_integration_tests
cargo test --test rafaeltab_descriptor_tests
```

### Run Specific Test

```bash
cargo test test_start_creates_sessions
```

### Run with Output

```bash
cargo test -- --nocapture
```

### Run with Backtrace

```bash
RUST_BACKTRACE=1 cargo test
```

## Best Practices

### 1. Always Use Descriptors for Integration Tests

✅ **Good**:

```rust
let env = TestEnvironment::describe(|root| {
    root.rafaeltab_config(|_c| {});
}).create();
```

❌ **Bad**:

```rust
let temp_dir = TempDir::new()?;
let config_file = temp_dir.path().join("config.json");
// Manual cleanup required, no tmux isolation, etc.
```

### 2. Use Helper Functions

The `tests/common/mod.rs` provides helper functions:

- `run_cli(args, config_path)` - Run CLI with config
- `run_cli_with_tmux(args, config_path, tmux_socket)` - Run CLI with tmux isolation

### 3. Descriptive Test Names

Use names that explain what is being tested:

```rust
#[test]
fn test_workspace_list_shows_all_configured_workspaces() { ... }

#[test]
fn test_tmux_start_is_idempotent() { ... }
```

### 4. Clear Error Messages

Include context in assertions:

```rust
assert!(
    success,
    "Command should succeed.\nSTDOUT: {}\nSTDERR: {}",
    stdout, stderr
);
```

### 5. Test Isolation

- Each test creates its own `TestEnvironment`
- Never share environments between tests
- All resources are automatically cleaned up
- Tmux sessions are completely isolated from user's tmux server

### 6. Config Structure

When manually creating configs, always include required fields:

```json
{
    "workspaces": [...],
    "tmux": {
        "sessions": [...],
        "defaultWindows": [...]
    }
}
```

## Troubleshooting

### Tests See User's Workspaces or Tmux Sessions

This means the CLI is not using the test config. Make sure you're using the helper functions correctly:

```rust
// ✅ Correct
let (stdout, _, _) = run_cli(&["workspace", "list"], config_path.to_str().unwrap());

// ❌ Wrong - missing --config flag
Command::new("target/debug/rafaeltab").args(&["workspace", "list"]).output()
```

### Config Validation Errors

Check that:

1. The config includes the `tmux` field
2. All workspace fields match the schema (id, name, root, tags)
3. Paths use the test environment's root: `env.root_path()`

### Tmux Tests Fail

Ensure you're using the isolated socket:

```rust
run_cli_with_tmux(&["tmux", "start"], config_path, env.tmux_socket())
```

## Architecture

```
┌─────────────────────────────────────┐
│      Test Descriptor Framework      │
│   (packages/test-descriptors)       │
│                                     │
│  - TestEnvironment                  │
│  - Git/Tmux/Directory descriptors   │
│  - Automatic cleanup                │
└─────────────────────────────────────┘
              │
              │ uses
              ▼
┌─────────────────────────────────────┐
│   Rafaeltab Test Descriptors        │
│   (tests/common/rafaeltab_descriptors) │
│                                     │
│  - RafaeltabRootMixin               │
│  - RafaeltabDirMixin                │
│  - RafaeltabGitMixin                │
│  - ConfigBuilder                    │
│  - WorkspaceBuilder                 │
└─────────────────────────────────────┘
              │
              │ used by
              ▼
┌─────────────────────────────────────┐
│      Integration Tests              │
│                                     │
│  - tests/integration_tests/         │
│  - tests/cli_integration_tests.rs   │
│  - tests/descriptor_tests.rs        │
└─────────────────────────────────────┘
```

## References

- [Test Descriptors Package](../../packages/test-descriptors/EXAMPLES.md) - Full examples and API documentation
- [Tmux Manual](https://man.openbsd.org/OpenBSD-current/man1/tmux.1) - For tmux socket options
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
