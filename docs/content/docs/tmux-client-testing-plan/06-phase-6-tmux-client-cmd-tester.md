---
title: Phase 6 TmuxClientCmdTester Implementation
---

# Phase 6: TmuxClientCmdTester Implementation

**Goal:** Implement the `TmuxClientCmdTester` which runs standard commands inside a tmux client (not TUI) and captures stdout/stderr/exit_code.

**Prerequisite:** Phase 3 complete (TmuxClientHandle working)

## Overview

This tester:

1. Runs command inside tmux session via `tmux run-shell`
2. Captures stdout and stderr separately using temp files
3. Returns exit code of the command
4. The `$TMUX` environment variable is automatically set by tmux
5. Used for testing CLI commands that need to know they're inside tmux

## Key Differences from Other Testers

| Aspect    | CmdTester         | TmuxClientCmdTester      |
| --------- | ----------------- | ------------------------ |
| Execution | Direct subprocess | Via `tmux run-shell`     |
| $TMUX env | Not set           | Set (by tmux)            |
| Output    | Direct capture    | Temp file capture        |
| Use case  | General CLI       | CLI needing tmux context |

## The run-shell Challenge

`tmux run-shell` executes a shell command but doesn't easily separate stdout and stderr. The solution is to use temp files:

```bash
STDOUT_FILE=$(mktemp)
STDERR_FILE=$(mktemp)
<command> >"$STDOUT_FILE" 2>"$STDERR_FILE"
EXIT_CODE=$?
cat "$STDOUT_FILE"
echo "===SEPARATOR==="
cat "$STDERR_FILE"
echo "===EXIT:$EXIT_CODE==="
rm "$STDOUT_FILE" "$STDERR_FILE"
```

## Components to Implement

### 1. TmuxClientCmdTester

**File:** `packages/test-descriptors/src/testers/tmux_client_cmd/tmux_client_cmd_tester.rs`

```rust
use crate::descriptor::{TmuxClientHandle, TmuxSocket};
use crate::testers::{Command, CommandResult, CommandTester};
use uuid::Uuid;

pub struct TmuxClientCmdTester<'a> {
    client: &'a TmuxClientHandle,
    socket: &'a TmuxSocket,
}

impl<'a> TmuxClientCmdTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle, socket: &'a TmuxSocket) -> Self {
        Self { client, socket }
    }
}

impl CommandTester for TmuxClientCmdTester<'_> {
    fn run(&self, cmd: &Command) -> CommandResult {
        // Generate unique separator to avoid conflicts with command output
        let separator = format!("===SEP_{}===", Uuid::new_v4().simple());
        let exit_marker = format!("===EXIT_{}===", Uuid::new_v4().simple());

        // Build the command with environment and cwd
        let mut cmd_parts = vec![];

        // Export environment variables
        for (key, value) in cmd.get_envs() {
            // Escape single quotes in values
            let escaped_value = value.replace("'", "'\\''");
            cmd_parts.push(format!("export {}='{}'", key, escaped_value));
        }

        // Change directory if specified
        if let Some(cwd) = cmd.get_cwd() {
            let cwd_str = cwd.to_string_lossy();
            cmd_parts.push(format!("cd '{}'", cwd_str));
        }

        // Build the actual command
        let program = cmd.program();
        let args = cmd.get_args()
            .iter()
            .map(|a| {
                // Escape single quotes in arguments
                let escaped = a.replace("'", "'\\''");
                format!("'{}'", escaped)
            })
            .collect::<Vec<_>>()
            .join(" ");

        let full_command = if args.is_empty() {
            program.to_string()
        } else {
            format!("{} {}", program, args)
        };

        // Build the wrapper script that captures stdout/stderr separately
        let script = format!(
            r#"
STDOUT_FILE=$(mktemp)
STDERR_FILE=$(mktemp)
{}
{} >"$STDOUT_FILE" 2>"$STDERR_FILE"
EXIT_CODE=$?
cat "$STDOUT_FILE"
printf '%s\n' '{}'
cat "$STDERR_FILE"
printf '%s%d\n' '{}' "$EXIT_CODE"
rm -f "$STDOUT_FILE" "$STDERR_FILE"
"#,
            cmd_parts.join("\n"),
            full_command,
            separator,
            exit_marker
        );

        // Execute via tmux run-shell
        let output = self.socket.run_tmux(&[
            "run-shell",
            "-t", self.client.session_name(),
            &script,
        ]);

        match output {
            Ok(raw_output) => self.parse_output(&raw_output, &separator, &exit_marker),
            Err(e) => CommandResult {
                stdout: String::new(),
                stderr: format!("Failed to run command in tmux: {}", e),
                exit_code: -1,
                success: false,
            }
        }
    }
}

impl TmuxClientCmdTester<'_> {
    fn parse_output(&self, raw: &str, separator: &str, exit_marker: &str) -> CommandResult {
        // Split by separator to get stdout and the rest
        let parts: Vec<&str> = raw.splitn(2, separator).collect();

        let stdout = parts.first()
            .map(|s| s.trim_end_matches('\n').to_string())
            .unwrap_or_default();

        if parts.len() < 2 {
            return CommandResult {
                stdout,
                stderr: String::new(),
                exit_code: -1,
                success: false,
            };
        }

        let rest = parts[1];

        // Find exit marker and extract exit code
        if let Some(exit_pos) = rest.find(exit_marker) {
            let stderr = rest[..exit_pos].trim_start_matches('\n').trim_end_matches('\n').to_string();
            let exit_str = &rest[exit_pos + exit_marker.len()..];
            let exit_code: i32 = exit_str.trim().parse().unwrap_or(-1);

            CommandResult {
                stdout,
                stderr,
                exit_code,
                success: exit_code == 0,
            }
        } else {
            CommandResult {
                stdout,
                stderr: rest.to_string(),
                exit_code: -1,
                success: false,
            }
        }
    }
}
```

### 2. TesterFactory Integration

**File:** `packages/test-descriptors/src/testers/factory.rs`

Update the `tmux_client_cmd()` method:

```rust
impl<'a> TesterFactory<'a> {
    pub fn tmux_client_cmd(&self) -> TmuxClientCmdTester {
        let client = self.env.tmux_client()
            .expect("No tmux client attached. Did you forget to call `s.with_client()` in your test setup?");

        TmuxClientCmdTester::new(client, self.env.tmux_socket())
    }
}
```

### 3. TmuxSocket Extension

**File:** `packages/test-descriptors/src/descriptor/tmux_socket.rs`

Ensure `run_tmux` can handle the run-shell command properly:

```rust
impl TmuxSocket {
    /// Run a tmux command and return stdout
    pub fn run_tmux(&self, args: &[&str]) -> Result<String, CreateError> {
        let output = std::process::Command::new("tmux")
            .arg("-L")
            .arg(&self.name)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::TmuxError(format!(
                "Tmux command failed: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
```

## Environment Variable Verification

Commands run via `tmux run-shell` automatically have `$TMUX` set because they're running inside the tmux environment. This is the key difference from `CmdTester`:

```rust
#[test]
fn tmux_client_cmd_runs_inside_client() {
    let env = TestEnvironment::describe(|root| {
        root.tmux_session("test-session", |s| {
            s.window("shell");
            s.with_client(|c| c.pty_size(40, 120));
        });
    }).create();

    let cmd = Command::new("printenv")
        .arg("TMUX");

    let result = env.testers().tmux_client_cmd().run(&cmd);

    // TMUX should be set and non-empty
    assert!(result.success);
    assert!(!result.stdout.trim().is_empty(), "TMUX env var should be set");
}
```

## Error Handling

### No Client Attached

```rust
#[test]
#[should_panic(expected = "No tmux client attached")]
fn tmux_client_cmd_panics_without_client() {
    let env = TestEnvironment::describe(|root| {
        root.tmux_session("test-session", |s| {
            s.window("shell");
            // Note: NO with_client() call
        });
    }).create();

    let cmd = Command::new("echo").arg("hello");

    // This should panic
    env.testers().tmux_client_cmd().run(&cmd);
}
```

### Command Failure

```rust
#[test]
fn tmux_client_cmd_captures_exit_code() {
    let env = /* ... with client ... */;

    let cmd = Command::new("sh")
        .arg("-c")
        .arg("exit 42");

    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(!result.success);
    assert_eq!(result.exit_code, 42);
}
```

## Edge Cases

### 1. Special Characters in Arguments

The implementation must handle special characters in arguments:

```rust
#[test]
fn tmux_client_cmd_handles_special_chars() {
    let env = /* ... with client ... */;

    let cmd = Command::new("echo")
        .arg("hello 'world'")
        .arg("with \"quotes\"")
        .arg("and $variables");

    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(result.success);
    assert!(result.stdout.contains("hello 'world'"));
}
```

### 2. Binary Output

Note: Binary output may not be perfectly preserved through the temp file + cat pipeline. For binary data, use `CmdTester` or handle encoding explicitly.

### 3. Large Output

For very large outputs, temp files handle this better than piping:

```rust
#[test]
fn tmux_client_cmd_handles_large_output() {
    let env = /* ... with client ... */;

    // Generate 1MB of output
    let cmd = Command::new("dd")
        .args(&["if=/dev/zero", "bs=1024", "count=1024", "status=none"])
        .args(&["|", "base64"]);  // Note: this won't work, use sh -c

    // Better approach:
    let cmd = Command::new("sh")
        .arg("-c")
        .arg("dd if=/dev/zero bs=1024 count=1024 status=none | base64");

    let result = env.testers().tmux_client_cmd().run(&cmd);

    assert!(result.success);
    assert!(result.stdout.len() > 1_000_000);
}
```

## Performance Considerations

The `run-shell` approach has overhead:

1. Creating temp files
2. Running through shell
3. Parsing output

For performance-critical tests, consider using `CmdTester` if tmux context isn't needed.

## Tests That Should Pass After This Phase

From Phase 1 test list:

- All tests in `tmux_client_cmd_tests.rs`:
  - `tmux_client_cmd_runs_inside_client`
  - `tmux_client_cmd_panics_without_client`
  - `tmux_client_cmd_separates_stdout_stderr`
  - `tmux_client_cmd_captures_exit_code`
- Command tests when run via `tmux_client_cmd()`

## Deliverables

1. Working `TmuxClientCmdTester` struct
2. `run-shell` wrapper script with output parsing
3. Proper escaping for special characters
4. Error handling for missing client
5. All `tmux_client_cmd` tests passing
