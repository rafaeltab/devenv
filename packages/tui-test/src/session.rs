use crate::pty_manager::PtyManager;
use crate::terminal::TerminalBuffer;
use crate::{Key, TextMatch, TuiError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub fn spawn_tui(command: &str, args: &[&str]) -> TuiSessionBuilder {
    TuiSessionBuilder::new(command, args)
}

pub struct TuiSessionBuilder {
    command: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
    cwd: Option<PathBuf>,
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
            cwd: None,
            rows: 40,
            cols: 120,
            settle_timeout_ms: Self::default_settle_timeout(),
            dump_on_fail: Self::default_dump_on_fail(),
        }
    }

    pub fn cwd<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.cwd = Some(dir.as_ref().to_path_buf());
        self
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
        let pty = PtyManager::spawn(
            &self.command,
            &self.args,
            &self.envs,
            self.rows,
            self.cols,
            self.cwd,
        )?;
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
    // Calculate the xterm modifier code
    // Shift = 2, Alt = 3, Shift+Alt = 4, Ctrl = 5, Ctrl+Shift = 6, Ctrl+Alt = 7, Ctrl+Shift+Alt = 8
    let has_shift = modifiers.iter().any(|k| matches!(k, Key::Shift));
    let has_alt = modifiers.iter().any(|k| matches!(k, Key::Alt));
    let has_ctrl = modifiers.iter().any(|k| matches!(k, Key::Ctrl));
    let has_super = modifiers.iter().any(|k| matches!(k, Key::Super));

    // Handle simple Alt+character combinations (traditional ESC prefix method)
    if modifiers.len() == 1 && has_alt {
        if let Key::Char(c) = key {
            // Alt+char is sent as ESC followed by the character
            let mut bytes = vec![0x1b]; // ESC
            bytes.extend_from_slice(c.to_string().as_bytes());
            return bytes;
        }
    }

    // Handle Ctrl+character combinations (without other modifiers)
    if modifiers.len() == 1 && has_ctrl {
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

    // For combinations with multiple modifiers or special keys with modifiers,
    // use xterm-style CSI sequences with modifier parameters
    if !modifiers.is_empty() {
        let modifier_code = calculate_modifier_code(has_shift, has_alt, has_ctrl, has_super);

        // Handle special keys (arrows, function keys, etc.) with modifiers
        match key {
            Key::Up => return format!("\x1b[1;{}A", modifier_code).into_bytes(),
            Key::Down => return format!("\x1b[1;{}B", modifier_code).into_bytes(),
            Key::Right => return format!("\x1b[1;{}C", modifier_code).into_bytes(),
            Key::Left => return format!("\x1b[1;{}D", modifier_code).into_bytes(),
            Key::Home => return format!("\x1b[1;{}H", modifier_code).into_bytes(),
            Key::End => return format!("\x1b[1;{}F", modifier_code).into_bytes(),
            Key::PageUp => return format!("\x1b[5;{}~", modifier_code).into_bytes(),
            Key::PageDown => return format!("\x1b[6;{}~", modifier_code).into_bytes(),
            Key::Char(c) => {
                // For character keys with multiple modifiers, use CSI u encoding
                let char_code = c as u32;
                return format!("\x1b[{};{}u", char_code, modifier_code).into_bytes();
            }
            _ => {}
        }
    }

    // No modifiers or unsupported combination - just send the key
    key_to_bytes(key)
}

fn calculate_modifier_code(shift: bool, alt: bool, ctrl: bool, super_: bool) -> u8 {
    // xterm modifier encoding:
    // 1 = no modifiers (but we add 1 as base, so shift=2, etc.)
    let mut code = 1u8;

    if shift {
        code += 1;
    }
    if alt {
        code += 2;
    }
    if ctrl {
        code += 4;
    }
    if super_ {
        code += 8;
    }

    code
}
