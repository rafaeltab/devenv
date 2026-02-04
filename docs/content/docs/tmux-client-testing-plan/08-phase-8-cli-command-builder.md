---
title: Phase 8 CliCommandBuilder Implementation
---

# Phase 8: CliCommandBuilder Implementation

**Goal:** Create a rafaeltab-specific command builder that wraps the generic `Command` and automatically configures rafaeltab CLI options.

**Prerequisite:** Phases 5-7 complete (all testers working with `Command`)

## Overview

While `Command` (from test-descriptors) is generic and works with any program, `CliCommandBuilder` is specifically designed for building rafaeltab CLI commands. It:

1. Automatically sets the rafaeltab binary path
2. Adds `--config` flag from TestEnvironment
3. Adds `RAFAELTAB_TMUX_SOCKET` environment variable
4. Provides rafaeltab-specific convenience methods

## Package Location

`CliCommandBuilder` lives in the CLI package, NOT in test-descriptors:

**File:** `apps/cli/tests/common/cli_command_builder.rs`

This keeps rafaeltab-specific code in the rafaeltab package while generic testing infrastructure remains in test-descriptors.

## Components to Implement

### 1. CliCommandBuilder

**File:** `apps/cli/tests/common/cli_command_builder.rs`

````rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use test_descriptors::testers::Command;
use test_descriptors::TestEnvironment;

/// Builder for rafaeltab CLI commands
///
/// This is a convenience wrapper around `Command` that automatically
/// configures rafaeltab-specific options like the binary path, config path,
/// and tmux socket.
///
/// # Example
///
/// ```ignore
/// let cmd = CliCommandBuilder::new()
///     .with_env(&env)
///     .args(&["tmux", "start"])
///     .build();
///
/// let result = env.testers().cmd().run(&cmd);
/// ```
pub struct CliCommandBuilder {
    args: Vec<String>,
    config_path: Option<PathBuf>,
    tmux_socket: Option<String>,
    cwd: Option<PathBuf>,
    extra_envs: HashMap<String, String>,
}

impl CliCommandBuilder {
    /// Create a new CLI command builder
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            config_path: None,
            tmux_socket: None,
            cwd: None,
            extra_envs: HashMap::new(),
        }
    }

    /// Configure using TestEnvironment (sets config path and tmux socket)
    ///
    /// This extracts the config path and tmux socket from the test environment
    /// and applies them to the command.
    pub fn with_env(mut self, env: &TestEnvironment) -> Self {
        if let Some(config_path) = env.context().config_path() {
            self.config_path = Some(config_path);
        }
        self.tmux_socket = Some(env.tmux_socket().to_string());
        self
    }

    /// Set config path explicitly (overrides with_env)
    pub fn with_config(mut self, path: impl AsRef<Path>) -> Self {
        self.config_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set tmux socket explicitly (overrides with_env)
    pub fn with_tmux_socket(mut self, socket: impl Into<String>) -> Self {
        self.tmux_socket = Some(socket.into());
        self
    }

    /// Set working directory for the command
    pub fn with_cwd(mut self, dir: impl AsRef<Path>) -> Self {
        self.cwd = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Add a custom environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_envs.insert(key.into(), value.into());
        self
    }

    /// Set the command arguments (the rafaeltab subcommand and its args)
    ///
    /// # Example
    ///
    /// ```ignore
    /// .args(&["tmux", "start"])
    /// .args(&["workspace", "list", "--json"])
    /// ```
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a single argument
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Build the final Command
    ///
    /// This creates a `Command` with:
    /// - The rafaeltab binary path
    /// - `--config <path>` prepended to args (if config path is set)
    /// - `RAFAELTAB_TMUX_SOCKET` environment variable (if tmux socket is set)
    /// - Any additional environment variables
    /// - Working directory (if set)
    pub fn build(self) -> Command {
        let binary_path = env!("CARGO_BIN_EXE_rafaeltab");

        let mut cmd = Command::new(binary_path);

        // Add --config flag before other args
        if let Some(ref config_path) = self.config_path {
            cmd = cmd.arg("--config");
            cmd = cmd.arg(config_path.to_string_lossy().to_string());
        }

        // Add the actual command arguments
        for arg in &self.args {
            cmd = cmd.arg(arg);
        }

        // Set RAFAELTAB_TMUX_SOCKET
        if let Some(ref socket) = self.tmux_socket {
            cmd = cmd.env("RAFAELTAB_TMUX_SOCKET", socket);
        }

        // Add extra environment variables
        for (key, value) in &self.extra_envs {
            cmd = cmd.env(key, value);
        }

        // Set working directory
        if let Some(ref cwd) = self.cwd {
            cmd = cmd.cwd(cwd);
        }

        cmd
    }
}

impl Default for CliCommandBuilder {
    fn default() -> Self {
        Self::new()
    }
}
````

### 2. Module Declaration

**File:** `apps/cli/tests/common/mod.rs`

Add the new module:

```rust
pub mod cli_command_builder;
pub use cli_command_builder::CliCommandBuilder;

// Keep existing code for backward compatibility during migration
// (will be removed in Phase 9)
```

## Usage Examples

### Basic Usage

```rust
use common::CliCommandBuilder;
use test_descriptors::TestEnvironment;

#[test]
fn test_tmux_start() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", Some("test-session"), &[("shell", None)]);
        });
        root.workspace("ws1");
    }).create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();

    let result = env.testers().cmd().run(&cmd);

    assert!(result.success, "tmux start failed: {}", result.stderr);
    assert!(env.tmux().session_exists("test-session"));
}
```

### With Custom Environment

```rust
#[test]
fn test_with_custom_env() {
    let env = /* ... */;

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("DEBUG", "1")
        .args(&["workspace", "list"])
        .build();

    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
}
```

### With Custom Working Directory

```rust
#[test]
fn test_worktree_current() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| c.defaults());
        root.workspace("my-workspace");
        root.git_repo("my-repo", |r| {
            r.worktree("feature-branch");
        });
    }).create();

    let worktree_path = env.find_worktree("my-repo", "feature-branch")
        .unwrap()
        .path()
        .to_path_buf();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&worktree_path)
        .args(&["worktree", "current"])
        .build();

    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert!(result.stdout.contains("feature-branch"));
}
```

### Inside Tmux Client

```rust
#[test]
fn test_switch_client() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("ws1", Some("source"), &[("shell", None)]);
            c.tmux_session("ws2", Some("target"), &[("shell", None)]);
        });
        root.workspace("ws1");
        root.workspace("ws2");

        root.tmux_session("source", |s| {
            s.window("shell");
            s.with_client(|c| c.pty_size(40, 120));
        });
        root.tmux_session("target", |s| {
            s.window("shell");
        });
    }).create();

    // Create sessions first
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    env.testers().cmd().run(&start_cmd);

    // Now switch inside the client
    let switch_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "switch", "--session", "target"])
        .build();

    let result = env.testers().tmux_client_cmd().run(&switch_cmd);

    assert!(result.success);
    assert_eq!(env.tmux_client().unwrap().current_session(), "target");
}
```

### TUI Testing

```rust
#[test]
fn test_tmux_switch_tui() {
    let env = /* ... with sessions and client ... */;

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "switch"])
        .build();

    // Test TUI outside tmux
    let mut session = env.testers().pty().run(&cmd);
    session.wait_for_settle();

    session.find_text("Select session").assert_visible();
    session.press_key(Key::Down);
    session.press_key(Key::Enter);

    session.expect_exit_code(0);
}
```

## Comparison: Old vs New

### Old Approach (CliTestRunner)

```rust
let (stdout, stderr, success) = CliTestRunner::new()
    .with_env(&env)
    .with_cwd(&some_path)
    .run(&["tmux", "start"]);

assert!(success, "Failed: {}", stderr);
```

### New Approach (CliCommandBuilder + Testers)

```rust
let cmd = CliCommandBuilder::new()
    .with_env(&env)
    .with_cwd(&some_path)
    .args(&["tmux", "start"])
    .build();

let result = env.testers().cmd().run(&cmd);

assert!(result.success, "Failed: {}", result.stderr);
```

The key difference is **separation of command building from execution**. The same command can be run through different testers:

```rust
let cmd = CliCommandBuilder::new()
    .with_env(&env)
    .args(&["tmux", "switch"])
    .build();

// Same command, different execution contexts:
let result1 = env.testers().cmd().run(&cmd);           // Outside tmux
let result2 = env.testers().tmux_client_cmd().run(&cmd); // Inside tmux client
let session1 = env.testers().pty().run(&cmd);          // TUI outside tmux
let session2 = env.testers().tmux_client_pty().run(&cmd); // TUI inside tmux
```

## Tests for CliCommandBuilder

Create test file: `apps/cli/tests/cli_command_builder_tests.rs`

```rust
use common::CliCommandBuilder;
use test_descriptors::TestEnvironment;

#[test]
fn builder_sets_binary_path() {
    let cmd = CliCommandBuilder::new()
        .args(&["--version"])
        .build();

    assert_eq!(cmd.program(), env!("CARGO_BIN_EXE_rafaeltab"));
}

#[test]
fn builder_adds_config_flag() {
    let cmd = CliCommandBuilder::new()
        .with_config("/path/to/config.json")
        .args(&["tmux", "start"])
        .build();

    let args = cmd.get_args();
    assert_eq!(args[0], "--config");
    assert_eq!(args[1], "/path/to/config.json");
    assert_eq!(args[2], "tmux");
    assert_eq!(args[3], "start");
}

#[test]
fn builder_sets_tmux_socket_env() {
    let cmd = CliCommandBuilder::new()
        .with_tmux_socket("test-socket")
        .build();

    let envs = cmd.get_envs();
    assert_eq!(envs.get("RAFAELTAB_TMUX_SOCKET"), Some(&"test-socket".to_string()));
}

#[test]
fn builder_with_env_extracts_from_environment() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| c.defaults());
    }).create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .build();

    // Should have config path set
    let args = cmd.get_args();
    assert!(args.contains(&"--config".to_string()));

    // Should have tmux socket set
    let envs = cmd.get_envs();
    assert!(envs.contains_key("RAFAELTAB_TMUX_SOCKET"));
}
```

## Deliverables

1. `CliCommandBuilder` struct in `apps/cli/tests/common/`
2. Builder methods: `with_env`, `with_config`, `with_tmux_socket`, `with_cwd`, `with_env_var`, `args`, `arg`
3. `build()` method that creates `Command`
4. Module declaration in `common/mod.rs`
5. Unit tests for the builder
6. Documentation with examples
