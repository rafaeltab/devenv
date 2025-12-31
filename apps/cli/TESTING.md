# Testing Guide

This document describes the testing patterns and infrastructure for the Rafaeltab CLI application.

## Table of Contents

- [Overview](#overview)
- [Test Types](#test-types)
- [Tmux Test Isolation](#tmux-test-isolation)
- [Writing Integration Tests](#writing-integration-tests)
- [Test Helpers](#test-helpers)
- [Running Tests](#running-tests)

## Overview

The CLI uses a combination of unit tests and integration tests to ensure correctness:

- **Unit Tests**: Located within source files using `#[cfg(test)]` modules
- **Integration Tests**: Located in `tests/` directory, test the CLI as a black box

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

Integration tests run the CLI binary as a subprocess and verify end-to-end behavior.

**Location**: `tests/integration_tests/*.rs`

**Key Characteristics**:

- Tests run against the compiled binary (`target/debug/rafaeltab`)
- Each test gets isolated configuration and state
- Tests should not interfere with each other or the user's environment

## Tmux Test Isolation

Testing tmux commands requires special care to avoid interfering with the user's running tmux server. We use **socket isolation** via the `-L` flag.

### How It Works

1. **TmuxConnection Abstraction**: The `TmuxConnection` struct encapsulates tmux server connection configuration
2. **Environment Variable**: Tests set `RAFAELTAB_TMUX_SOCKET` to specify a unique socket name
3. **Isolated Servers**: Each test uses a unique tmux server socket (e.g., `rafaeltab_test_uuid`)
4. **Automatic Cleanup**: Test servers are killed when tests complete

### Architecture

```
┌─────────────────────────────────────┐
│         Production Code             │
│                                     │
│  TmuxConnection::default()          │
│    ↓                                │
│  Uses default tmux socket           │
│  (connects to user's tmux server)   │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│            Test Code                │
│                                     │
│  TmuxConnection::with_socket("...")  │
│    ↓                                │
│  Uses custom socket (-L flag)       │
│  (isolated test tmux server)        │
└─────────────────────────────────────┘
```

## Writing Integration Tests

### Basic Structure

```rust
use crate::common::helpers::TestContext;
use std::process::Command;

#[test]
fn test_my_feature() {
    // 1. Create test config
    let config = r#"{
        "workspaces": [],
        "tmux": {
            "sessions": [],
            "defaultWindows": []
        }
    }"#;

    let config_ctx = TestContext::new(config)
        .expect("Failed to create config context");

    // 2. Run CLI
    let output = Command::new("target/debug/rafaeltab")
        .args(["--config", config_ctx.config_path()])
        .args(["workspace", "list"])
        .output()
        .expect("Failed to run CLI");

    // 3. Assert results
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("expected output"));
}
```

### Testing Tmux Commands

When testing tmux commands, use `TmuxTestContext` for isolation:

```rust
use crate::common::helpers::TestContext;
use crate::common::tmux_helpers::TmuxTestContext;
use std::process::Command;

#[test]
fn test_tmux_start_creates_session() {
    // 1. Create isolated tmux server context
    let tmux_ctx = TmuxTestContext::new()
        .expect("Failed to create tmux test context");

    // 2. Create config with workspace
    let config = format!(
        r#"{{
        "workspaces": [{{
            "id": "test_ws",
            "name": "test workspace",
            "root": "{}",
            "tags": []
        }}],
        "tmux": {{
            "sessions": [{{
                "workspace": "test_ws",
                "name": "my-session",
                "windows": [{{ "name": "shell" }}]
            }}],
            "defaultWindows": []
        }}
    }}"#,
        tmux_ctx.temp_dir_path().display()
    );

    let config_ctx = TestContext::new(&config)
        .expect("Failed to create config context");

    // 3. Run CLI with isolated tmux socket
    let output = Command::new("target/debug/rafaeltab")
        .args(["--config", config_ctx.config_path()])
        .env("RAFAELTAB_TMUX_SOCKET", tmux_ctx.socket_name())
        .args(["tmux", "start"])
        .output()
        .expect("Failed to run CLI");

    // 4. Assert command succeeded
    assert!(
        output.status.success(),
        "Command failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // 5. Verify sessions using tmux helper
    let sessions = tmux_ctx.list_sessions();
    assert!(sessions.contains(&"test workspace".to_string()));

    // Cleanup happens automatically via Drop trait
}
```

## Test Helpers

### TestContext (Config Isolation)

**Location**: `tests/common/helpers.rs`

**Purpose**: Creates temporary config files for isolated testing

**Methods**:

- `new(content: &str) -> io::Result<Self>` - Creates temp config with JSON content
- `config_path() -> &str` - Returns path to temp config file

**Usage**:

```rust
let config_ctx = TestContext::new(r#"{"workspaces": []}"#)?;
// Use config_ctx.config_path() with --config flag
// Temp file is automatically cleaned up when config_ctx is dropped
```

### TmuxTestContext (Tmux Isolation)

**Location**: `tests/common/tmux_helpers.rs`

**Purpose**: Creates isolated tmux servers for testing

**Methods**:

- `new() -> io::Result<Self>` - Creates context with unique socket name
- `socket_name() -> &str` - Returns the unique socket name for this test
- `temp_dir_path() -> &Path` - Returns path to temp directory for test files
- `tmux(args: &[&str]) -> String` - Runs tmux command on isolated server
- `list_sessions() -> Vec<String>` - Lists sessions on isolated server
- `session_exists(name: &str) -> bool` - Checks if session exists
- `kill_server()` - Manually kills the test server (auto-called on drop)

**Usage**:

```rust
let tmux_ctx = TmuxTestContext::new()?;

// Pass socket to CLI
.env("RAFAELTAB_TMUX_SOCKET", tmux_ctx.socket_name())

// Verify tmux state
let sessions = tmux_ctx.list_sessions();
assert!(sessions.contains(&"my-session".to_string()));
```

### Key Features

1. **Automatic Cleanup**: Both contexts implement `Drop` to clean up resources
2. **Unique Isolation**: Each test gets a unique socket/config
3. **No Interference**: Tests don't affect user's tmux or config files
4. **Parallel Safe**: Tests can run in parallel without conflicts

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Integration Tests Only

```bash
cargo test --test integration_test
```

### Run Specific Test File

```bash
cargo test --test integration_test tmux_start
```

### Run Specific Test

```bash
cargo test --test integration_test test_start_is_idempotent
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

### 1. Config Structure

Always include required fields in test configs:

```json
{
    "workspaces": [...],
    "tmux": {
        "sessions": [...],
        "defaultWindows": [...]
    }
}
```

**Workspace fields**:

- `id` (required): Pattern `^[a-z_]{3,20}$`
- `name` (required): Pattern `^[a-zA-Z ]{3,20}$`
- `root` (required): Path to workspace directory
- `tags` (required): Array of tag strings

**Session fields**:

- `windows` (required): Array of window definitions
- Either `workspace` (for workspace-based) OR `path` + `name` (for path-based)

### 2. Error Messages

Include helpful diagnostics in assertions:

```rust
assert!(
    output.status.success(),
    "Command failed:\nstdout: {}\nstderr: {}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
);
```

### 3. Test Isolation

- Never rely on global state
- Always use `TestContext` for config
- Always use `TmuxTestContext` for tmux tests
- Don't hardcode paths - use `temp_dir_path()`

### 4. Test Naming

Use descriptive names that explain what is being tested:

```rust
#[test]
fn test_start_creates_sessions_from_workspace_config() { ... }

#[test]
fn test_start_is_idempotent() { ... }
```

### 5. Session Names

Be aware that session names come from the workspace `name` field (or session `name` for path-based sessions), which can contain spaces:

```rust
// Config has workspace.name = "test workspace"
let sessions = tmux_ctx.list_sessions();
assert!(sessions.contains(&"test workspace".to_string())); // Correct
```

## Troubleshooting

### Tests Hang

If tests hang, a tmux server might not have been cleaned up:

```bash
# List all tmux servers
tmux list-sessions -a

# Kill test servers
tmux -L rafaeltab_test_<uuid> kill-server
```

### Config Validation Errors

Check the schema at `schemas/config-schema.json` for required fields and patterns.

### Socket Permission Issues

Ensure `/tmp` is writable. Test sockets are created in `/tmp/tmux-<uid>/`.

## Future Improvements

- Add test coverage metrics
- Add property-based testing for complex scenarios
- Add performance benchmarks
- Add tests for error handling and edge cases
- Consider adding mocks for external dependencies (git, filesystem, etc.)

## References

- [Tmux Manual](https://man.openbsd.org/OpenBSD-current/man1/tmux.1) - For tmux socket options
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
