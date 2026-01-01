use crate::pty_manager::PtyManager;
use crate::terminal::TerminalBuffer;
use crate::{Key, TextMatch, TuiError};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub fn spawn_tui(command: &str, args: &[&str]) -> TuiSessionBuilder {
    TuiSessionBuilder::new(command, args)
}

pub struct TuiSessionBuilder {
    command: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
    rows: u16,
    cols: u16,
    settle_timeout_ms: u64,
    dump_on_fail: bool,
}

impl TuiSessionBuilder {
    pub fn new(command: &str, args: &[&str]) -> Self {
        Self {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            envs: HashMap::new(),
            rows: 40,
            cols: 120,
            settle_timeout_ms: Self::default_settle_timeout(),
            dump_on_fail: Self::default_dump_on_fail(),
        }
    }

    fn default_settle_timeout() -> u64 {
        std::env::var("TUI_TEST_SETTLE_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300)
    }

    fn default_dump_on_fail() -> bool {
        std::env::var("TUI_TEST_DUMP_ON_FAIL")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.envs.insert(key.to_string(), value.to_string());
        self
    }

    pub fn terminal_size(mut self, rows: u16, cols: u16) -> Self {
        self.rows = rows;
        self.cols = cols;
        self
    }

    pub fn settle_timeout(mut self, ms: u64) -> Self {
        self.settle_timeout_ms = ms;
        self
    }

    pub fn dump_on_fail(mut self, enabled: bool) -> Self {
        self.dump_on_fail = enabled;
        self
    }

    pub fn spawn(self) -> Result<TuiSession, TuiError> {
        let pty = PtyManager::spawn(&self.command, &self.args, &self.envs, self.rows, self.cols)?;
        let terminal = TerminalBuffer::new(self.rows, self.cols);

        Ok(TuiSession {
            pty,
            terminal,
            settle_timeout_ms: self.settle_timeout_ms,
            dump_on_fail: self.dump_on_fail,
            exit_code: None,
        })
    }
}

pub struct TuiSession {
    pty: PtyManager,
    terminal: TerminalBuffer,
    settle_timeout_ms: u64,
    dump_on_fail: bool,
    exit_code: Option<i32>,
}

impl TuiSession {
    // === Internal helpers ===
    fn read_pty_output(&mut self) {
        if let Ok(bytes) = self.pty.read_available() {
            if !bytes.is_empty() {
                self.terminal.process_bytes(&bytes);
            }
        }

        // Check if process exited
        if self.exit_code.is_none() {
            self.exit_code = self.pty.check_exit();
        }
    }

    // === Lifecycle ===
    pub fn wait_for_settle(&mut self) {
        self.wait_for_settle_ms(self.settle_timeout_ms, 1000);
    }

    pub fn wait_for_settle_ms(&mut self, timeout_ms: u64, max_wait_ms: u64) {
        const CHECK_INTERVAL_MS: u64 = 16; // ~60fps

        let start = Instant::now();
        let mut last_screen = self.terminal.clone();
        let mut stable_duration = 0u64;

        loop {
            std::thread::sleep(Duration::from_millis(CHECK_INTERVAL_MS));
            self.read_pty_output();

            if self.terminal == last_screen {
                stable_duration += CHECK_INTERVAL_MS;

                if stable_duration >= timeout_ms {
                    // Screen has been stable for timeout period
                    return;
                }
            } else {
                // Screen changed, reset counter
                stable_duration = 0;
                last_screen = self.terminal.clone();
            }

            // Max wait timeout
            if start.elapsed().as_millis() as u64 >= max_wait_ms {
                return;
            }
        }
    }

    pub fn wait_ms(&mut self, ms: u64) {
        std::thread::sleep(Duration::from_millis(ms));
        self.read_pty_output();
    }

    pub fn expect_completion(&mut self) -> i32 {
        // Wait for process to exit if it hasn't already
        if self.exit_code.is_none() {
            self.exit_code = self.pty.wait_exit();
        }

        self.exit_code
            .expect("Process did not exit or exit code unavailable")
    }

    pub fn expect_exit_code(&mut self, expected: i32) {
        let actual = self.expect_completion();
        if actual != expected {
            panic!("expected exit code {}, but got {}", expected, actual);
        }
    }

    // === Input ===
    pub fn type_text(&mut self, text: &str) {
        self.pty
            .write(text.as_bytes())
            .expect("Failed to write to PTY");
    }

    pub fn press_key(&mut self, key: Key) {
        let bytes = key_to_bytes(key);
        self.pty.write(&bytes).expect("Failed to write to PTY");
    }

    pub fn send_keys(&mut self, keys: &[Key]) {
        // Separate modifiers from regular keys
        let (modifiers, regular_keys): (Vec<&Key>, Vec<&Key>) = keys
            .iter()
            .partition(|k| matches!(k, Key::Ctrl | Key::Alt | Key::Shift | Key::Super));

        if regular_keys.is_empty() {
            panic!("send_keys requires at least one non-modifier key");
        }

        if regular_keys.len() > 1 {
            panic!("send_keys can only send one non-modifier key at a time");
        }

        let bytes = build_key_sequence(*regular_keys[0], &modifiers);
        self.pty.write(&bytes).expect("Failed to write to PTY");
    }

    // === Queries ===
    pub fn find_text(&self, text: &str) -> TextMatch {
        TextMatch::new(text, &self.terminal, self.dump_on_fail)
    }

    pub fn find_all_text(&self, text: &str) -> Vec<TextMatch> {
        let positions = self.terminal.find_all_text(text);
        positions
            .into_iter()
            .map(|pos| {
                TextMatch::new_with_position(text, Some(pos), &self.terminal, self.dump_on_fail)
            })
            .collect()
    }

    pub fn screen(&self) -> String {
        self.terminal.render()
    }

    // === Debug ===
    pub fn dump_screen(&self) {
        eprintln!("\n=== Screen Dump ===");
        eprintln!("{}", self.terminal.render());
        eprintln!("===================\n");
    }
}

// === Key conversion helpers ===

fn key_to_bytes(key: Key) -> Vec<u8> {
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

fn build_key_sequence(key: Key, modifiers: &[&Key]) -> Vec<u8> {
    // Handle Ctrl combinations specially
    if modifiers.iter().any(|k| matches!(k, Key::Ctrl)) {
        if let Key::Char(c) = key {
            // Ctrl+letter produces the control code
            let ctrl_code = match c.to_ascii_lowercase() {
                'a'..='z' => (c.to_ascii_lowercase() as u8) - b'a' + 1,
                '@' => 0,
                '[' => 27,
                '\\' => 28,
                ']' => 29,
                '^' => 30,
                '_' => 31,
                _ => return key_to_bytes(key), // Fallback
            };
            return vec![ctrl_code];
        }
    }

    // For other combinations, just send the key
    // TODO: Implement proper modifier encoding for Alt, Shift, Super
    key_to_bytes(key)
}
