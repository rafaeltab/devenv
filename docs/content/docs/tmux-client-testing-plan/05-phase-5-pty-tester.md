---
title: Phase 5 PtyTester Implementation
---

# Phase 5: PtyTester Implementation

**Goal:** Implement the `PtyTester` which runs TUI commands outside of tmux in a direct PTY.

**Prerequisite:** Phase 3 and 4 complete (TuiSession, TerminalBuffer, PTY backend working)

## Overview

This tester:

1. Spawns the command directly in a new PTY (no tmux)
2. Reads from the PTY to capture terminal output
3. Sends keystrokes directly to the PTY
4. No `$TMUX` environment variable is set
5. Simplest TUI testing scenario

## Key Differences from Other TUI Testers

| Aspect        | TmuxFullClientTester  | TmuxClientPtyTester   | PtyTester     |
| ------------- | --------------------- | --------------------- | ------------- |
| Tmux involved | Yes (client PTY)      | Yes (capture-pane)    | No            |
| Output source | Client PTY            | `capture-pane`        | Direct PTY    |
| Input method  | Write to client PTY   | `send-keys`           | Write to PTY  |
| $TMUX env     | Set (by tmux)         | Set (by tmux)         | Not set       |
| Setup needed  | Tmux session + client | Tmux session + client | None          |
| Performance   | Medium                | Slower (polling)      | Fastest       |
| Asserter type | `FullClientAsserter`  | `CapturePaneAsserter` | `PtyAsserter` |

## Components to Implement

### 1. PtyTester

**File:** `packages/test-descriptors/src/testers/pty/pty_tester.rs`

```rust
use crate::testers::{Command, TuiTester};
use crate::testers::pty::PtyAsserter;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

pub struct PtyTester {
    rows: u16,
    cols: u16,
    settle_timeout_ms: u64,
}

impl PtyTester {
    pub(crate) fn new() -> Self {
        Self {
            rows: 40,
            cols: 120,
            settle_timeout_ms: 300,
        }
    }

    /// Set terminal dimensions for the PTY
    pub fn terminal_size(mut self, rows: u16, cols: u16) -> Self {
        self.rows = rows;
        self.cols = cols;
        self
    }

    /// Set the settle timeout for wait_for_settle operations
    pub fn settle_timeout(mut self, ms: u64) -> Self {
        self.settle_timeout_ms = ms;
        self
    }
}

impl TuiTester for PtyTester {
    type Asserter = PtyAsserter;

    fn run(&self, cmd: &Command) -> Self::Asserter {
        // 1. Create PTY
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: self.rows,
                cols: self.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to create PTY");

        // 2. Build command
        let mut pty_cmd = CommandBuilder::new(cmd.program());
        for arg in cmd.get_args() {
            pty_cmd.arg(arg);
        }
        for (key, value) in cmd.get_envs() {
            pty_cmd.env(key, value);
        }
        if let Some(cwd) = cmd.get_cwd() {
            pty_cmd.cwd(cwd);
        }

        // 3. Spawn command in PTY
        let child = pty_pair
            .slave
            .spawn_command(pty_cmd)
            .expect("Failed to spawn command in PTY");

        // 4. Get reader and writer
        let reader = pty_pair
            .master
            .try_clone_reader()
            .expect("Failed to clone PTY reader");

        let writer = pty_pair
            .master
            .take_writer()
            .expect("Failed to take PTY writer");

        // 5. Create PtyAsserter
        PtyAsserter::new(
            reader,
            writer,
            child,
            (self.rows, self.cols),
            self.settle_timeout_ms,
        )
    }
}
```

### 2. PtyAsserter

**File:** `packages/test-descriptors/src/testers/pty/pty_asserter.rs`

```rust
use crate::testers::internal::{TerminalBuffer, PtyBackend};
use crate::testers::{TuiAsserter, Key, TextMatch};

pub struct PtyAsserter {
    backend: PtyBackend,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    terminal: TerminalBuffer,
    settle_timeout_ms: u64,
    exit_code: Option<i32>,
}

impl PtyAsserter {
    pub(crate) fn new(
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
        child: Box<dyn portable_pty::Child + Send + Sync>,
        size: (u16, u16),
        settle_timeout_ms: u64,
    ) -> Self {
        Self {
            backend: PtyBackend::new(reader, writer),
            child,
            terminal: TerminalBuffer::new(size.0, size.1),
            settle_timeout_ms,
            exit_code: None,
        }
    }

    fn read_pty_output(&mut self) {
        if let Some(bytes) = self.backend.read_available() {
            self.terminal.process_bytes(&bytes);
        }
    }

    fn check_exit(&mut self) {
        if self.exit_code.is_none() {
            if let Ok(Some(status)) = self.child.try_wait() {
                self.exit_code = Some(status.exit_code() as i32);
            }
        }
    }
}

impl TuiAsserter for PtyAsserter {
    fn wait_for_settle(&mut self) {
        self.wait_for_settle_ms(self.settle_timeout_ms, 1000);
    }

    fn wait_for_settle_ms(&mut self, timeout_ms: u64, max_wait_ms: u64) {
        const CHECK_INTERVAL_MS: u64 = 16;
        let start = Instant::now();
        let mut last_screen = self.terminal.clone();
        let mut stable_duration = 0u64;

        loop {
            thread::sleep(Duration::from_millis(CHECK_INTERVAL_MS));
            self.read_pty_output();

            if self.terminal == last_screen {
                stable_duration += CHECK_INTERVAL_MS;
                if stable_duration >= timeout_ms {
                    return;
                }
            } else {
                stable_duration = 0;
                last_screen = self.terminal.clone();
            }

            if start.elapsed().as_millis() as u64 >= max_wait_ms {
                return;
            }
        }
    }

    fn wait_ms(&mut self, ms: u64) {
        thread::sleep(Duration::from_millis(ms));
        self.read_pty_output();
    }

    fn expect_completion(&mut self) -> i32 {
        match self.child.wait() {
            Ok(status) => status.exit_code() as i32,
            Err(_) => -1,
        }
    }

    fn expect_exit_code(&mut self, expected: i32) {
        let actual = self.expect_completion();
        if actual != expected {
            panic!("Expected exit code {}, got {}", expected, actual);
        }
    }

    fn type_text(&mut self, text: &str) {
        self.backend.write_bytes(text.as_bytes()).expect("Failed to write");
    }

    fn press_key(&mut self, key: Key) {
        let bytes = key_to_bytes(key);
        self.backend.write_bytes(&bytes).expect("Failed to write");
    }

    fn send_keys(&mut self, keys: &[Key]) {
        let (modifiers, regular): (Vec<&Key>, Vec<&Key>) = keys
            .iter()
            .partition(|k| matches!(k, Key::Ctrl | Key::Alt | Key::Shift | Key::Super));

        if regular.is_empty() {
            panic!("send_keys requires at least one non-modifier key");
        }
        if regular.len() > 1 {
            panic!("send_keys can only send one non-modifier key at a time");
        }

        let bytes = build_key_sequence(*regular[0], &modifiers);
        self.backend.write_bytes(&bytes).expect("Failed to write");
    }

    fn find_text(&self, text: &str) -> TextMatch {
        let positions = self.terminal.find_all_text(text);

        match positions.len() {
            0 => TextMatch::not_found(text, &self.terminal),
            1 => TextMatch::found(text, positions[0], &self.terminal),
            _ => panic!("'{}' found multiple occurrences ({}). Use find_all_text() instead.",
                        text, positions.len()),
        }
    }

    fn find_all_text(&self, text: &str) -> Vec<TextMatch> {
        self.terminal.find_all_text(text)
            .into_iter()
            .map(|pos| TextMatch::found(text, pos, &self.terminal))
            .collect()
    }

    fn screen(&self) -> String {
        self.terminal.render()
    }

    fn dump_screen(&self) {
        eprintln!("{}", self.terminal.render());
    }
}

impl Drop for PtyAsserter {
    fn drop(&mut self) {
        // Kill the child process if still running
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
```

### 3. TesterFactory Integration

**File:** `packages/test-descriptors/src/testers/factory.rs`

The `pty()` method should already be defined from Phase 2. Update implementation:

```rust
impl<'a> TesterFactory<'a> {
    pub fn pty(&self) -> PtyTester {
        PtyTester::new()
    }
}
```

## Environment Verification

One key aspect of PtyTester is that `$TMUX` should NOT be set. Tests should verify this:

```rust
#[test]
fn pty_tester_runs_outside_tmux() {
    let env = TestEnvironment::new().create();

    let cmd = Command::new("printenv")
        .arg("TMUX");

    // Run via pty - should not have TMUX set
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // TMUX should not appear in output (or be empty)
    asserter.find_text("TMUX=").assert_not_visible();
}
```

## Refactoring Note

Since `PtyAsserter` and `FullClientAsserter` share much of the same logic via `PtyBackend`, the shared internal module keeps this code DRY. Both use:

- `PtyBackend` for reading/writing
- `TerminalBuffer` for parsing and querying
- Same `TuiAsserter` trait implementation patterns

The key difference is that `PtyAsserter` owns the child process and can get its exit code directly.

## Tests That Should Pass After This Phase

From Phase 1 test list:

- All tests in `pty_tester_tests.rs`
- All lifecycle tests when run via `pty()`
- All input tests when run via `pty()`
- All text finding tests when run via `pty()`
- All color tests when run via `pty()`
- All terminal sequence tests when run via `pty()`
- All integration tests when run via `pty()`

## Key Advantages of PtyTester

1. **No tmux dependency** - Tests run even if tmux is not installed
2. **Fastest execution** - No tmux overhead
3. **Full key support** - All key combinations work
4. **Clean exit code handling** - Direct access to process exit
5. **Simplest debugging** - Fewer layers to debug

## Deliverables

1. Working `PtyTester` struct (in `testers/pty/`)
2. Working `PtyAsserter` implementing `TuiAsserter` (in `testers/pty/`)
3. Reuse of shared `PtyBackend` from `internal/`
4. Exit code handling via owned child process
5. All `pty` tests passing
