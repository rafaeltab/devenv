---
title: Tmux Client Testing Infrastructure - Overview
---

# Tmux Client Testing Infrastructure - Overview

This document describes the architecture and API design for a new testing infrastructure that enables running CLI commands and TUI tests inside tmux clients.

## Goals

1. Run commands inside a tmux client with proper `$TMUX` environment
2. Run TUI tests that capture tmux pane output (via `tmux capture-pane`)
3. Run TUI tests against the entire tmux client (including tmux UI)
4. Run commands outside tmux (existing behavior)
5. Run TUI tests outside tmux (existing behavior)
6. Verify client session attachment state
7. Decouple command building from execution context
8. Deprecate and replace the `tui-test` package

## Architecture

### Core Principles

1. **Separation of Concerns:**

   - `Command` - What to run (generic command with args, env vars, cwd)
   - `CliCommandBuilder` - Rafaeltab-specific command building (config path, tmux socket)
   - Testers - How to run it (PTY, subprocess, inside/outside tmux)
   - Output capture - What to return (stdout/stderr/exit_code vs TUI buffer)

2. **Declarative Client Setup:**

   - Client is defined in test environment via `s.with_client()`
   - Maximum of 1 client per test environment
   - Client lifecycle managed by `TestEnvironment`

3. **Unified Testing Interface:**
   - All TUI testers expose same methods: `find_text()`, `press_key()`, color assertions, etc.
   - All command testers return `CommandResult { stdout, stderr, exit_code, success }`

### Package Structure

```
packages/
  test-descriptors/
    src/
      testers/               # NEW: Tester trait definitions
        mod.rs               # Re-exports traits and public types
        traits.rs            # CommandTester, TuiTester traits
        tui_asserter.rs      # TuiAsserter trait definition
        command.rs           # Command struct and CommandResult
        keys.rs              # Key enum definition
        color.rs             # ColorMatcher, ColorAssertion types
        text_match.rs        # TextMatch struct
        cmd/                 # CmdTester implementation
          mod.rs
          cmd_tester.rs      # Standard subprocess execution
        pty/                 # PtyTester implementation
          mod.rs
          pty_tester.rs      # PTY execution outside tmux
          pty_asserter.rs    # TuiAsserter impl for PTY
        tmux_client_cmd/     # TmuxClientCmdTester implementation
          mod.rs
          tmux_client_cmd_tester.rs
        tmux_client_pty/     # TmuxClientPtyTester implementation
          mod.rs
          tmux_client_pty_tester.rs
          capture_pane_asserter.rs  # TuiAsserter impl for capture-pane
        tmux_full_client/    # TmuxFullClientTester implementation
          mod.rs
          tmux_full_client_tester.rs
          full_client_asserter.rs   # TuiAsserter impl for full client
        internal/            # Shared internal implementations
          mod.rs
          terminal_buffer.rs # Terminal emulation buffer
          pty_backend.rs     # Shared PTY reading/writing logic
          key_conversion.rs  # Key to bytes/tmux format conversion
      descriptor/
        tmux_client.rs       # NEW: TmuxClientDescriptor
        ... existing files
      builders/
        tmux.rs              # MODIFIED: Add with_client()
        ... existing files

apps/cli/
  tests/
    common/
      mod.rs                 # MODIFIED: CliCommandBuilder
      ... existing files
```

## API Design

### Test Environment Setup

```rust
let env = TestEnvironment::describe(|root| {
    root.rafaeltab_config(|c| {
        c.tmux_session("ws1", Some("my-session"), &[("shell", None)]);
    });
    root.workspace("ws1");

    root.tmux_session("my-session", |s| {
        s.window("shell");
        s.with_client(|c| {
            c.pty_size(40, 120);
        });
    });
}).create();
```

### Command Building

```rust
// Generic command (in test-descriptors)
let cmd = Command::new("my-binary")
    .args(&["arg1", "arg2"])
    .env("KEY", "value")
    .cwd("/some/path");

// Rafaeltab-specific command (in cli tests)
let cmd = CliCommandBuilder::new()
    .with_env(&env)  // Sets RAFAELTAB_TMUX_SOCKET + config path
    .args(&["tmux", "switch"])
    .build();
```

### Execution Contexts (Testers)

```rust
// Get testers from environment
let testers = env.testers();

// 1. Standard command outside tmux
let result = testers.cmd().run(&cmd);
assert!(result.success);

// 2. Standard command inside tmux client
let result = testers.tmux_client_cmd().run(&cmd);
assert!(result.success);

// 3. TUI outside tmux (direct PTY)
let mut asserter = testers.pty().run(&cmd);
asserter.find_text("Hello").assert_visible();

// 4. TUI inside tmux (capture-pane)
let mut asserter = testers.tmux_client_pty().run(&cmd);
asserter.find_text("Hello").assert_visible();

// 5. TUI including tmux client UI
let mut asserter = testers.tmux_full_client().run(&cmd);
asserter.find_text("Hello").assert_visible();
```

### Client Inspection

```rust
// Check if environment has a client
assert!(env.has_tmux_client());

// Get current session the client is attached to
let session_name = env.tmux_client().unwrap().current_session();
assert_eq!(session_name, "target-session");
```

## Tester Types Summary

| Tester               | Input   | Environment                  | Output             |
| -------------------- | ------- | ---------------------------- | ------------------ |
| `cmd()`              | Command | Subprocess                   | `CommandResult`    |
| `tmux_client_cmd()`  | Command | Inside tmux via `run-shell`  | `CommandResult`    |
| `pty()`              | Command | Direct PTY                   | `impl TuiAsserter` |
| `tmux_client_pty()`  | Command | Tmux pane via `capture-pane` | `impl TuiAsserter` |
| `tmux_full_client()` | Command | Tmux client PTY              | `impl TuiAsserter` |

## Key Data Structures

### Command

```rust
pub struct Command {
    program: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
    cwd: Option<PathBuf>,
}
```

### CommandResult

```rust
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}
```

### TuiAsserter Trait

```rust
/// Trait defining the interface for TUI test assertions and interactions.
///
/// All TUI testers return types that implement this trait, allowing tests
/// to be written generically against any TUI execution backend.
pub trait TuiAsserter {
    // Lifecycle
    fn wait_for_settle(&mut self);
    fn wait_for_settle_ms(&mut self, timeout_ms: u64, max_wait_ms: u64);
    fn wait_ms(&mut self, ms: u64);
    fn expect_completion(&mut self) -> i32;
    fn expect_exit_code(&mut self, expected: i32);

    // Input
    fn type_text(&mut self, text: &str);
    fn press_key(&mut self, key: Key);
    fn send_keys(&mut self, keys: &[Key]);

    // Queries
    fn find_text(&self, text: &str) -> TextMatch;
    fn find_all_text(&self, text: &str) -> Vec<TextMatch>;
    fn screen(&self) -> String;

    // Debug
    fn dump_screen(&self);
}
```

### TextMatch

```rust
pub struct TextMatch {
    pub fg: ColorAssertion,
    pub bg: ColorAssertion,
}

impl TextMatch {
    pub fn position(&self) -> Option<(u16, u16)>;
    pub fn assert_visible(&self);
    pub fn assert_not_visible(&self);
}
```

### ColorMatcher

```rust
pub enum ColorMatcher {
    Grayscale,
    RedIsh,
    GreenIsh,
    BlueIsh,
    YellowIsh,
    CyanIsh,
    MagentaIsh,
    Hue { min: f32, max: f32 },
}
```

## Tmux Client Implementation Details

### Client Creation

The client is created via `portable_pty`:

```rust
// Spawn tmux client attached to session
let pty_system = native_pty_system();
let pty_pair = pty_system.openpty(PtySize { rows, cols, ... })?;

let mut cmd = CommandBuilder::new("tmux");
cmd.arg("-L").arg(&socket_name);
cmd.arg("attach-session");
cmd.arg("-t").arg(&session_name);

let child = pty_pair.slave.spawn_command(cmd)?;
```

### Command Execution Inside Client

Uses `tmux run-shell` with temp files for stdout/stderr capture:

```rust
let script = format!(r#"
    STDOUT=$(mktemp)
    STDERR=$(mktemp)
    {} >"$STDOUT" 2>"$STDERR"
    EXIT=$?
    cat "$STDOUT"
    echo "---STDERR---"
    cat "$STDERR"
    echo "---EXIT:$EXIT---"
    rm "$STDOUT" "$STDERR"
"#, command);

socket.run_tmux(&["run-shell", "-t", session_name, &script])
```

### TUI Capture via capture-pane

```rust
let output = socket.run_tmux(&[
    "capture-pane",
    "-t", &session_name,
    "-p",  // Print to stdout
    "-e",  // Include escape sequences (colors)
    "-J",  // Join wrapped lines
])?;
```

### Key Sending

```rust
fn key_to_tmux_format(key: Key) -> String {
    match key {
        Key::Enter => "Enter".to_string(),
        Key::Esc => "Escape".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Up => "Up".to_string(),
        Key::Down => "Down".to_string(),
        Key::Char(c) => c.to_string(),
        // ...
    }
}

socket.run_tmux(&["send-keys", "-t", session_name, &key_str])
```

## Constraints

1. **Maximum 1 client per test environment** - simplifies API and avoids confusion
2. **Client must be attached to existing session** - error if session doesn't exist
3. **Session must be created before client** - descriptor ordering matters
4. **$TMUX is NOT manually set** - it's automatically set by being inside the tmux client
5. **No support for multiple modifiers in capture-pane** - use direct PTY for complex key combos

## Implementation Phases

1. **Phase 1:** Write comprehensive test suite (tests won't compile)
2. **Phase 2:** Define interfaces with `todo!()` (tests compile but fail)
3. **Phase 3:** Implement `tmux_full_client()` tester
4. **Phase 4:** Implement `tmux_client_pty()` tester
5. **Phase 5:** Implement `pty()` tester
6. **Phase 6:** Implement `tmux_client_cmd()` tester
7. **Phase 7:** Implement `cmd()` tester
8. **Phase 8:** Create `CliCommandBuilder` for rafaeltab CLI
9. **Phase 9:** Remove old packages and helpers
10. **Phase 10:** Migrate existing CLI tests

## Files Being Deprecated

- `packages/tui-test/` - entire package
- `apps/cli/tests/common/mod.rs` - `CliTestRunner` and `TuiCliTestRunner`

## Dependencies

- `portable_pty` - PTY management
- `alacritty_terminal` or custom - Terminal emulation for buffer parsing
- `tempfile` - For stdout/stderr capture in run-shell
