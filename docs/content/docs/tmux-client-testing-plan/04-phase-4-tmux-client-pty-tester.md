---
title: Phase 4 TmuxClientPtyTester Implementation
---

# Phase 4: TmuxClientPtyTester Implementation

**Goal:** Implement the `TmuxClientPtyTester` which captures TUI output via `tmux capture-pane` (excludes tmux UI).

**Prerequisite:** Phase 3 complete (TerminalBuffer, TextMatch, etc. working)

## Overview

This tester:

1. Runs command inside tmux session via `send-keys`
2. Captures pane content via `tmux capture-pane -p -e` (with ANSI codes)
3. Sends keys via `tmux send-keys`
4. Only shows the application output, NOT the tmux status bar

## Key Differences from TmuxFullClientTester

| Aspect        | TmuxFullClientTester            | TmuxClientPtyTester    |
| ------------- | ------------------------------- | ---------------------- |
| Output source | Client PTY                      | `capture-pane` command |
| Input method  | Write to PTY                    | `send-keys` command    |
| Shows tmux UI | Yes                             | No                     |
| Color support | Full                            | Via `-e` flag          |
| Key support   | Full (including complex combos) | Limited by `send-keys` |
| Asserter type | `FullClientAsserter`            | `CapturePaneAsserter`  |

## Components to Implement

### 1. TmuxClientPtyTester

**File:** `packages/test-descriptors/src/testers/tmux_client_pty/tmux_client_pty_tester.rs`

```rust
pub struct TmuxClientPtyTester<'a> {
    client: &'a TmuxClientHandle,
    socket: &'a TmuxSocket,
    settle_timeout_ms: u64,
}

impl<'a> TmuxClientPtyTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle, socket: &'a TmuxSocket) -> Self {
        Self {
            client,
            socket,
            settle_timeout_ms: 300,
        }
    }

    pub fn settle_timeout(mut self, ms: u64) -> Self {
        self.settle_timeout_ms = ms;
        self
    }
}

impl TuiTester for TmuxClientPtyTester<'_> {
    type Asserter = CapturePaneAsserter;

    fn run(&self, cmd: &Command) -> Self::Asserter {
        // 1. Build the command string with env vars
        let mut cmd_parts = vec![];
        for (k, v) in cmd.get_envs() {
            cmd_parts.push(format!("export {}='{}'", k, v));
        }
        if let Some(cwd) = cmd.get_cwd() {
            cmd_parts.push(format!("cd '{}'", cwd.display()));
        }
        cmd_parts.push(format!("{} {}",
            cmd.program(),
            cmd.get_args().join(" ")
        ));
        let full_cmd = cmd_parts.join("; ");

        // 2. Send the command to the active pane
        self.socket.run_tmux(&[
            "send-keys",
            "-t", self.client.session_name(),
            &full_cmd,
            "Enter"
        ]).expect("Failed to send command to pane");

        // 3. Create CapturePaneAsserter
        CapturePaneAsserter::new(
            self.socket.clone(),
            self.client.session_name().to_string(),
            self.client.pty_size(),
            self.settle_timeout_ms,
        )
    }
}
```

### 2. CapturePaneAsserter

**File:** `packages/test-descriptors/src/testers/tmux_client_pty/capture_pane_asserter.rs`

```rust
use crate::testers::internal::TerminalBuffer;

pub struct CapturePaneAsserter {
    socket: TmuxSocket,
    session_name: String,
    terminal: TerminalBuffer,
    settle_timeout_ms: u64,
    exit_code: Option<i32>,
}

impl CapturePaneAsserter {
    pub(crate) fn new(
        socket: TmuxSocket,
        session_name: String,
        size: (u16, u16),
        settle_timeout_ms: u64,
    ) -> Self {
        Self {
            socket,
            session_name,
            terminal: TerminalBuffer::new(size.0, size.1),
            settle_timeout_ms,
            exit_code: None,
        }
    }

    fn capture_pane(&mut self) {
        let output = self.socket.run_tmux(&[
            "capture-pane",
            "-t", &self.session_name,
            "-p",  // Print to stdout
            "-e",  // Include escape sequences (colors)
            "-J",  // Join wrapped lines
        ]).expect("Failed to capture pane");

        // Clear and reprocess the entire buffer
        self.terminal.clear();
        self.terminal.process_bytes(output.as_bytes());
    }
}

impl TuiAsserter for CapturePaneAsserter {
    fn wait_for_settle(&mut self) {
        self.wait_for_settle_ms(self.settle_timeout_ms, 1000);
    }

    fn wait_for_settle_ms(&mut self, timeout_ms: u64, max_wait_ms: u64) {
        const CHECK_INTERVAL_MS: u64 = 16;
        let start = Instant::now();
        let mut last_screen = String::new();
        let mut stable_duration = 0u64;

        loop {
            thread::sleep(Duration::from_millis(CHECK_INTERVAL_MS));
            self.capture_pane();
            let current_screen = self.terminal.render();

            if current_screen == last_screen {
                stable_duration += CHECK_INTERVAL_MS;
                if stable_duration >= timeout_ms {
                    return;
                }
            } else {
                stable_duration = 0;
                last_screen = current_screen;
            }

            if start.elapsed().as_millis() as u64 >= max_wait_ms {
                return;
            }
        }
    }

    fn type_text(&mut self, text: &str) {
        self.socket.run_tmux(&[
            "send-keys",
            "-t", &self.session_name,
            "-l",  // Literal (disable key lookup)
            text,
        ]).expect("Failed to send text");
    }

    fn press_key(&mut self, key: Key) {
        let key_name = key_to_tmux_name(key);
        self.socket.run_tmux(&[
            "send-keys",
            "-t", &self.session_name,
            &key_name,
        ]).expect("Failed to send key");
    }

    fn send_keys(&mut self, keys: &[Key]) {
        // Validate: at least one non-modifier key
        let (modifiers, regular): (Vec<&Key>, Vec<&Key>) = keys
            .iter()
            .partition(|k| matches!(k, Key::Ctrl | Key::Alt | Key::Shift | Key::Super));

        if regular.is_empty() {
            panic!("send_keys requires at least one non-modifier key");
        }
        if regular.len() > 1 {
            panic!("send_keys can only send one non-modifier key at a time");
        }

        let key_name = build_tmux_key_name(*regular[0], &modifiers);
        self.socket.run_tmux(&[
            "send-keys",
            "-t", &self.session_name,
            &key_name,
        ]).expect("Failed to send keys");
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

    // ... other trait methods
}
```

### 3. Key Name Conversion for tmux send-keys

**File:** `packages/test-descriptors/src/testers/internal/key_conversion.rs`

```rust
/// Convert a Key to tmux send-keys format
pub(crate) fn key_to_tmux_name(key: Key) -> String {
    match key {
        Key::Char(c) => c.to_string(),
        Key::Enter => "Enter".to_string(),
        Key::Esc => "Escape".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Backspace => "BSpace".to_string(),
        Key::Up => "Up".to_string(),
        Key::Down => "Down".to_string(),
        Key::Left => "Left".to_string(),
        Key::Right => "Right".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        Key::Ctrl | Key::Alt | Key::Shift | Key::Super => {
            panic!("Modifier keys cannot be sent alone via tmux")
        }
    }
}

/// Build tmux key name with modifiers (e.g., "C-a" for Ctrl+a)
pub(crate) fn build_tmux_key_name(key: Key, modifiers: &[&Key]) -> String {
    let has_ctrl = modifiers.iter().any(|k| matches!(k, Key::Ctrl));
    let has_alt = modifiers.iter().any(|k| matches!(k, Key::Alt));
    let has_shift = modifiers.iter().any(|k| matches!(k, Key::Shift));

    let base = match key {
        Key::Char(c) => c.to_string(),
        Key::Enter => "Enter".to_string(),
        Key::Esc => "Escape".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::Backspace => "BSpace".to_string(),
        Key::Up => "Up".to_string(),
        Key::Down => "Down".to_string(),
        Key::Left => "Left".to_string(),
        Key::Right => "Right".to_string(),
        Key::Home => "Home".to_string(),
        Key::End => "End".to_string(),
        Key::PageUp => "PageUp".to_string(),
        Key::PageDown => "PageDown".to_string(),
        _ => panic!("Invalid key"),
    };

    // Build modifier prefix (tmux format: C- for Ctrl, M- for Alt, S- for Shift)
    let mut prefix = String::new();
    if has_ctrl {
        prefix.push_str("C-");
    }
    if has_alt {
        prefix.push_str("M-");
    }
    if has_shift {
        prefix.push_str("S-");
    }

    format!("{}{}", prefix, base)
}
```

### 4. TerminalBuffer.clear()

**File:** `packages/test-descriptors/src/testers/internal/terminal_buffer.rs`

Add method to clear buffer (for capture-pane which returns full content each time):

```rust
impl TerminalBuffer {
    pub fn clear(&mut self) {
        for row in &mut self.grid {
            for cell in row {
                *cell = Cell::default();
            }
        }
        self.cursor = (0, 0);
    }
}
```

## capture-pane Considerations

### Flags Used

- `-p` - Print to stdout instead of saving to buffer
- `-e` - Include escape sequences (preserves colors)
- `-J` - Join any wrapped lines and preserve trailing whitespace

### ANSI Code Handling

The `-e` flag makes `capture-pane` output ANSI escape sequences. The `TerminalBuffer` from Phase 3 already handles these via the VTE parser.

### Limitations

1. **No complex key combos** - Some modifier combinations may not work via `send-keys`
2. **Polling-based** - We capture the full pane each time (less efficient than streaming)
3. **Snapshot timing** - There may be slight delays between `send-keys` and visible changes

### Performance

For better performance during `wait_for_settle`:

- Consider caching the last capture
- Only re-capture on timeout check
- Compare raw string before parsing ANSI (faster equality check)

## Tests That Should Pass After This Phase

From Phase 1 test list:

- All tests in `tmux_client_pty_tests.rs`
- Basic lifecycle tests when run via `tmux_client_pty()`
- Input tests (basic keys) when run via `tmux_client_pty()`
- Text finding tests when run via `tmux_client_pty()`
- Color tests when run via `tmux_client_pty()`
- Terminal sequence tests when run via `tmux_client_pty()`

Note: Some complex key combo tests (like `send_keys_ctrl_shift_r`) may need to be skipped or marked as expected-to-fail for this tester type.

## Deliverables

1. Working `TmuxClientPtyTester`
2. Working `CapturePaneAsserter` implementing `TuiAsserter`
3. `key_to_tmux_name` and `build_tmux_key_name` functions (in `internal/key_conversion.rs`)
4. `TerminalBuffer.clear()` method
5. All `tmux_client_pty` tests passing (except known limitations)
