---
title: Phase 3 TmuxFullClientTester Implementation
---

# Phase 3: TmuxFullClientTester Implementation

**Goal:** Implement the `TmuxFullClientTester` which provides full TUI testing against the tmux client itself (including tmux UI like status bar).

**Prerequisite:** Phase 2 complete (interfaces defined)

## Overview

This tester:

1. Uses the existing tmux client PTY spawned by `with_client()`
2. Sends the command to execute inside the tmux session
3. Reads from the client's PTY to capture terminal output
4. Sends keystrokes to the client's PTY
5. The user sees the full tmux interface including status bar

## Components to Implement

### 1. TmuxFullClientTester

**File:** `packages/test-descriptors/src/testers/tmux_full_client/tmux_full_client_tester.rs`

**Responsibilities:**

- Hold reference to `TmuxClientHandle`
- Configure settle timeout
- Execute command inside the tmux pane
- Return a `FullClientAsserter` that implements `TuiAsserter`

**Implementation:**

```rust
impl TuiTester for TmuxFullClientTester<'_> {
    type Asserter = FullClientAsserter;

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

        // 2. Send the command to the active pane via tmux send-keys
        self.socket.run_tmux(&[
            "send-keys",
            "-t", self.client.session_name(),
            &full_cmd,
            "Enter"
        ]).expect("Failed to send command to pane");

        // 3. Create FullClientAsserter backed by client PTY
        FullClientAsserter::new(
            self.client.pty_reader(),
            self.client.pty_writer(),
            self.client.pty_size(),
            self.settle_timeout_ms,
        )
    }
}
```

### 2. TmuxClientHandle Extensions

**File:** `packages/test-descriptors/src/descriptor/tmux_client.rs`

Need to expose PTY reader/writer:

```rust
impl TmuxClientHandle {
    pub(crate) fn pty_reader(&self) -> Box<dyn Read + Send>;
    pub(crate) fn pty_writer(&self) -> Box<dyn Write + Send>;
}
```

### 3. FullClientAsserter (implements TuiAsserter)

**File:** `packages/test-descriptors/src/testers/tmux_full_client/full_client_asserter.rs`

Implement the PTY-backed asserter:

```rust
use crate::testers::internal::{TerminalBuffer, PtyBackend};

pub struct FullClientAsserter {
    backend: PtyBackend,
    terminal: TerminalBuffer,
    settle_timeout_ms: u64,
    exit_code: Option<i32>,
}

impl FullClientAsserter {
    pub(crate) fn new(
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
        size: (u16, u16),
        settle_timeout_ms: u64,
    ) -> Self {
        Self {
            backend: PtyBackend::new(reader, writer),
            terminal: TerminalBuffer::new(size.0, size.1),
            settle_timeout_ms,
            exit_code: None,
        }
    }
}

impl TuiAsserter for FullClientAsserter {
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

    fn type_text(&mut self, text: &str) {
        self.backend.write_bytes(text.as_bytes()).expect("Failed to write");
    }

    fn press_key(&mut self, key: Key) {
        let bytes = key_to_bytes(key);
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

    fn screen(&self) -> String {
        self.terminal.render()
    }

    // ... other trait methods
}

impl FullClientAsserter {
    fn read_pty_output(&mut self) {
        if let Some(bytes) = self.backend.read_available() {
            self.terminal.process_bytes(&bytes);
        }
    }
}
```

### 4. TerminalBuffer

**File:** `packages/test-descriptors/src/testers/internal/terminal_buffer.rs`

This is the terminal emulator that parses ANSI sequences. Options:

**Option A: Use `vte` + custom grid (simpler)**

```rust
use vte::{Parser, Perform};

pub struct TerminalBuffer {
    rows: u16,
    cols: u16,
    grid: Vec<Vec<Cell>>,
    cursor: (u16, u16),
    parser: Parser,
    // ... saved cursor, scroll region, etc.
}

struct Cell {
    character: char,
    fg: Color,
    bg: Color,
    attrs: CellAttributes,
}

impl Perform for TerminalBuffer {
    fn print(&mut self, c: char) { /* write char at cursor */ }
    fn execute(&mut self, byte: u8) { /* handle control chars */ }
    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, action: char) {
        /* handle CSI sequences like cursor movement, colors, etc. */
    }
    // ... other methods
}

impl TerminalBuffer {
    pub fn process_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.parser.advance(self, *byte);
        }
    }

    pub fn find_text(&self, text: &str) -> Option<(u16, u16)>;
    pub fn find_all_text(&self, text: &str) -> Vec<(u16, u16)>;
    pub fn render(&self) -> String;
    pub fn get_cell(&self, row: u16, col: u16) -> &Cell;
}
```

**Option B: Use `alacritty_terminal` (more complete)**

The existing `tui-test` likely uses something similar. Check if we can reuse.

### 5. PtyBackend (shared internal)

**File:** `packages/test-descriptors/src/testers/internal/pty_backend.rs`

```rust
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub struct PtyBackend {
    writer: Box<dyn Write + Send>,
    read_buffer: Arc<Mutex<Vec<u8>>>,
    reader_thread: Option<JoinHandle<()>>,
}

impl PtyBackend {
    pub fn new(mut reader: Box<dyn Read + Send>, writer: Box<dyn Write + Send>) -> Self {
        let read_buffer = Arc::new(Mutex::new(Vec::new()));
        let read_buffer_clone = Arc::clone(&read_buffer);

        let reader_thread = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut buf) = read_buffer_clone.lock() {
                            buf.extend_from_slice(&buffer[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            writer,
            read_buffer,
            reader_thread: Some(reader_thread),
        }
    }

    pub fn read_available(&self) -> Option<Vec<u8>> {
        if let Ok(mut buf) = self.read_buffer.lock() {
            if !buf.is_empty() {
                let bytes = buf.clone();
                buf.clear();
                return Some(bytes);
            }
        }
        None
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }
}
```

### 6. TuiAsserter Method Implementations

For the PTY backend (FullClientAsserter), most methods are already shown above. The key pattern is:

- Use `PtyBackend` for reading/writing
- Use `TerminalBuffer` for parsing and querying
- Implement all `TuiAsserter` trait methods

### 7. Key to Bytes Conversion

**File:** `packages/test-descriptors/src/testers/internal/key_conversion.rs`

```rust
pub(crate) fn key_to_bytes(key: Key) -> Vec<u8> {
    match key {
        Key::Char(c) => c.to_string().into_bytes(),
        Key::Enter => b"\r".to_vec(),
        Key::Esc => b"\x1b".to_vec(),
        Key::Tab => b"\t".to_vec(),
        Key::Backspace => b"\x7f".to_vec(),
        Key::Up => b"\x1b[A".to_vec(),
        Key::Down => b"\x1b[B".to_vec(),
        Key::Right => b"\x1b[C".to_vec(),
        Key::Left => b"\x1b[D".to_vec(),
        Key::Home => b"\x1b[H".to_vec(),
        Key::End => b"\x1b[F".to_vec(),
        Key::PageUp => b"\x1b[5~".to_vec(),
        Key::PageDown => b"\x1b[6~".to_vec(),
        Key::Ctrl | Key::Alt | Key::Shift | Key::Super => {
            panic!("Modifier keys must be used with send_keys()")
        }
    }
}

pub(crate) fn build_key_sequence(key: Key, modifiers: &[&Key]) -> Vec<u8> {
    // Handle modifier combinations
    // See existing tui-test implementation for reference
}
```

### 8. TextMatch Implementation

**File:** `packages/test-descriptors/src/testers/text_match.rs`

```rust
pub struct TextMatch {
    text: String,
    position: Option<(u16, u16)>,
    terminal_snapshot: TerminalBuffer,  // Snapshot for color queries
    pub fg: ColorAssertion,
    pub bg: ColorAssertion,
}

impl TextMatch {
    pub(crate) fn not_found(text: &str, terminal: &TerminalBuffer) -> Self {
        Self {
            text: text.to_string(),
            position: None,
            terminal_snapshot: terminal.clone(),
            fg: ColorAssertion::not_found(),
            bg: ColorAssertion::not_found(),
        }
    }

    pub(crate) fn found(text: &str, pos: (u16, u16), terminal: &TerminalBuffer) -> Self {
        let cell = terminal.get_cell(pos.0, pos.1);
        Self {
            text: text.to_string(),
            position: Some(pos),
            terminal_snapshot: terminal.clone(),
            fg: ColorAssertion::new(cell.fg),
            bg: ColorAssertion::new(cell.bg),
        }
    }

    pub fn position(&self) -> Option<(u16, u16)> {
        self.position
    }

    pub fn assert_visible(&self) {
        if self.position.is_none() {
            panic!("'{}' should be visible on screen but was not found.\n\nScreen:\n{}",
                   self.text, self.terminal_snapshot.render());
        }
    }

    pub fn assert_not_visible(&self) {
        if self.position.is_some() {
            panic!("'{}' should NOT be visible on screen but was found at {:?}.\n\nScreen:\n{}",
                   self.text, self.position, self.terminal_snapshot.render());
        }
    }
}
```

### 9. ColorAssertion Implementation

**File:** `packages/test-descriptors/src/testers/color.rs`

```rust
pub struct ColorAssertion {
    color: Option<Color>,  // None if text not found
}

impl ColorAssertion {
    pub(crate) fn not_found() -> Self {
        Self { color: None }
    }

    pub(crate) fn new(color: Color) -> Self {
        Self { color: Some(color) }
    }

    pub fn assert(&self, matcher: ColorMatcher) {
        let color = self.color.expect("Cannot check color: text not found");
        if !matcher.matches(color.r, color.g, color.b) {
            panic!("Color ({}, {}, {}) does not match {:?}",
                   color.r, color.g, color.b, matcher);
        }
    }

    pub fn exact(&self, r: u8, g: u8, b: u8) {
        let color = self.color.expect("Cannot check color: text not found");
        if color.r != r || color.g != g || color.b != b {
            panic!("Expected color ({}, {}, {}), got ({}, {}, {})",
                   r, g, b, color.r, color.g, color.b);
        }
    }
}
```

## TmuxClientHandle Creation (from with_client)

**File:** `packages/test-descriptors/src/descriptor/tmux_client.rs`

```rust
impl Descriptor for TmuxClientDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        let socket_name = context.tmux_socket()
            .ok_or_else(|| CreateError::InvalidDescriptor("No tmux socket".to_string()))?;

        let socket = TmuxSocket::from_name(socket_name.clone());

        // Verify session exists
        if !socket.session_exists(&self.session_name) {
            return Err(CreateError::InvalidDescriptor(
                format!("Cannot attach client to non-existent session: {}", self.session_name)
            ));
        }

        // Create PTY
        let pty_system = portable_pty::native_pty_system();
        let pty_pair = pty_system.openpty(portable_pty::PtySize {
            rows: self.pty_rows,
            cols: self.pty_cols,
            pixel_width: 0,
            pixel_height: 0,
        }).map_err(|e| CreateError::TmuxError(e.to_string()))?;

        // Spawn tmux client
        let mut cmd = portable_pty::CommandBuilder::new("tmux");
        cmd.arg("-L").arg(&socket_name);
        cmd.arg("attach-session");
        cmd.arg("-t").arg(&self.session_name);

        let child = pty_pair.slave.spawn_command(cmd)
            .map_err(|e| CreateError::TmuxError(e.to_string()))?;

        // Wait for client to attach
        std::thread::sleep(std::time::Duration::from_millis(150));

        // Store the client
        context.register_tmux_client(TmuxClientHandle {
            session_name: self.session_name.clone(),
            pty_pair,
            child,
            pty_size: (self.pty_rows, self.pty_cols),
            socket: socket.clone(),
        })?;

        Ok(())
    }
}
```

## Tests That Should Pass After This Phase

From Phase 1 test list:

- All tests in `tmux_full_client_tests.rs`
- Basic lifecycle tests when run via `tmux_full_client()`
- Input tests when run via `tmux_full_client()`
- Text finding tests when run via `tmux_full_client()`
- Color tests when run via `tmux_full_client()`
- Terminal sequence tests when run via `tmux_full_client()`

## Dependencies

Add to `Cargo.toml`:

- `portable-pty` - PTY management
- `vte` - Terminal escape sequence parsing (or alternative)

## Deliverables

1. Working `TmuxFullClientTester`
2. Working `FullClientAsserter` implementing `TuiAsserter`
3. Working `TerminalBuffer` with ANSI parsing (in `internal/`)
4. Working `PtyBackend` shared logic (in `internal/`)
5. Working `TextMatch` and `ColorAssertion`
6. Working `TmuxClientHandle` creation
7. All `tmux_full_client` tests passing
