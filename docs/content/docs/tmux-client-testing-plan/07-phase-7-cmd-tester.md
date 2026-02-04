---
title: Phase 7 CmdTester Implementation
---

# Phase 7: CmdTester Implementation

**Goal:** Implement the `CmdTester` which runs standard commands outside of tmux as a simple subprocess.

**Prerequisite:** Phase 2 complete (interfaces defined)

## Overview

This tester:

1. Spawns command directly as a subprocess using `std::process::Command`
2. Captures stdout and stderr directly
3. Returns exit code from process status
4. No `$TMUX` environment variable is set
5. Simplest and fastest command execution

This is the "baseline" tester - the simplest way to run a command.

## Key Differences from TmuxClientCmdTester

| Aspect       | CmdTester           | TmuxClientCmdTester      |
| ------------ | ------------------- | ------------------------ |
| Execution    | Direct subprocess   | Via `tmux run-shell`     |
| $TMUX env    | Not set             | Set (by tmux)            |
| Output       | Direct capture      | Temp file capture        |
| Performance  | Fastest             | Slower (tmux overhead)   |
| Dependencies | None                | Requires tmux + client   |
| Use case     | General CLI testing | CLI needing tmux context |

## Components to Implement

### 1. CmdTester

**File:** `packages/test-descriptors/src/testers/cmd/cmd_tester.rs`

```rust
use crate::testers::{Command, CommandResult, CommandTester};
use std::process::Command as StdCommand;

/// Tester that runs commands as direct subprocesses (outside tmux)
pub struct CmdTester;

impl CmdTester {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl CommandTester for CmdTester {
    fn run(&self, cmd: &Command) -> CommandResult {
        // Build the std::process::Command
        let mut process = StdCommand::new(cmd.program());

        // Add arguments
        for arg in cmd.get_args() {
            process.arg(arg);
        }

        // Add environment variables
        for (key, value) in cmd.get_envs() {
            process.env(key, value);
        }

        // Set working directory if specified
        if let Some(cwd) = cmd.get_cwd() {
            process.current_dir(cwd);
        }

        // Execute and capture output
        match process.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                CommandResult {
                    stdout,
                    stderr,
                    exit_code,
                    success: output.status.success(),
                }
            }
            Err(e) => {
                CommandResult {
                    stdout: String::new(),
                    stderr: format!("Failed to execute command: {}", e),
                    exit_code: -1,
                    success: false,
                }
            }
        }
    }
}
```

### 2. TesterFactory Integration

**File:** `packages/test-descriptors/src/testers/factory.rs`

Update the `cmd()` method:

```rust
impl<'a> TesterFactory<'a> {
    /// Get a tester that runs commands as direct subprocesses (outside tmux)
    pub fn cmd(&self) -> CmdTester {
        CmdTester::new()
    }
}
```

## Usage Examples

### Basic Command Execution

```rust
#[test]
fn run_simple_command() {
    let env = TestEnvironment::new().create();

    let cmd = Command::new("echo")
        .arg("hello")
        .arg("world");

    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert_eq!(result.stdout.trim(), "hello world");
    assert!(result.stderr.is_empty());
    assert_eq!(result.exit_code, 0);
}
```

### Capturing stderr

```rust
#[test]
fn run_command_captures_stderr() {
    let env = TestEnvironment::new().create();

    let cmd = Command::new("sh")
        .arg("-c")
        .arg("echo 'error message' >&2");

    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert!(result.stdout.is_empty());
    assert_eq!(result.stderr.trim(), "error message");
}
```

### Exit Code Handling

```rust
#[test]
fn run_command_captures_exit_code() {
    let env = TestEnvironment::new().create();

    let cmd = Command::new("sh")
        .arg("-c")
        .arg("exit 42");

    let result = env.testers().cmd().run(&cmd);

    assert!(!result.success);
    assert_eq!(result.exit_code, 42);
}
```

### Environment Variables

```rust
#[test]
fn run_command_with_env_var() {
    let env = TestEnvironment::new().create();

    let cmd = Command::new("printenv")
        .arg("MY_VAR")
        .env("MY_VAR", "my_value");

    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert_eq!(result.stdout.trim(), "my_value");
}
```

### Working Directory

```rust
#[test]
fn run_command_with_cwd() {
    let env = TestEnvironment::new().create();

    let cmd = Command::new("pwd")
        .cwd("/tmp");

    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert_eq!(result.stdout.trim(), "/tmp");
}
```

## Environment Verification

Key test to verify CmdTester runs outside tmux:

```rust
#[test]
fn cmd_tester_runs_outside_tmux() {
    let env = TestEnvironment::new().create();

    let cmd = Command::new("printenv")
        .arg("TMUX");

    let result = env.testers().cmd().run(&cmd);

    // Command succeeds but TMUX is not set (empty output)
    // Note: printenv returns exit code 1 if variable is not set
    assert!(!result.success || result.stdout.trim().is_empty());
}
```

## Error Handling

### Command Not Found

```rust
#[test]
fn run_nonexistent_command() {
    let env = TestEnvironment::new().create();

    let cmd = Command::new("this-command-does-not-exist");

    let result = env.testers().cmd().run(&cmd);

    assert!(!result.success);
    assert_eq!(result.exit_code, -1);
    assert!(result.stderr.contains("Failed to execute"));
}
```

### Permission Denied

```rust
#[test]
fn run_non_executable() {
    let env = TestEnvironment::new().create();

    // Create a non-executable file
    let file_path = env.root_path().join("not_executable");
    std::fs::write(&file_path, "#!/bin/sh\necho hello").unwrap();
    // Note: Don't chmod +x

    let cmd = Command::new(file_path.to_string_lossy().to_string());

    let result = env.testers().cmd().run(&cmd);

    assert!(!result.success);
}
```

## Performance Characteristics

CmdTester is the fastest option because:

1. No tmux overhead
2. No temp file creation
3. Direct subprocess execution
4. Direct stdout/stderr capture

Use CmdTester when:

- Command doesn't need tmux context
- Performance is important
- Testing basic CLI functionality

## Comparison with Existing CliTestRunner

The existing `CliTestRunner` in `apps/cli/tests/common/mod.rs` does essentially the same thing but is specific to the rafaeltab binary. `CmdTester` is generic and works with any command.

```rust
// Old approach (rafaeltab-specific)
let (stdout, stderr, success) = CliTestRunner::new()
    .with_env(&env)
    .run(&["tmux", "start"]);

// New approach (generic)
let cmd = Command::new(env!("CARGO_BIN_EXE_rafaeltab"))
    .args(&["tmux", "start"])
    .env("RAFAELTAB_TMUX_SOCKET", env.tmux_socket());

let result = env.testers().cmd().run(&cmd);
```

The new approach separates command building from execution, allowing the same command to be run through different testers.

## Tests That Should Pass After This Phase

From Phase 1 test list:

- All tests in `cmd_tester_tests.rs`:
  - `cmd_tester_runs_outside_tmux`
  - `cmd_tester_inherits_env`
- All tests in `command_tests.rs`:
  - `run_simple_command`
  - `run_command_with_args`
  - `run_command_captures_stderr`
  - `run_command_captures_exit_code`
  - `run_command_with_env_var`
  - `run_command_with_cwd`
  - `run_command_success_flag`
  - `run_command_failure_flag`

## Deliverables

1. Working `CmdTester` struct
2. `CommandTester` trait implementation
3. Direct subprocess execution
4. stdout/stderr/exit_code capture
5. Environment variable and cwd support
6. All `cmd` tests passing
