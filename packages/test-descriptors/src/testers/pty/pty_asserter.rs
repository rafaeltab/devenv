use crate::testers::color::ColorAssertion;
use crate::testers::internal::{KeyConversion, PtyBackend, TerminalBuffer};
use crate::testers::keys::Key;
use crate::testers::text_match::TextMatch;
use crate::testers::tui_asserter::TuiAsserter;
use portable_pty::Child;
use std::thread;
use std::time::{Duration, Instant};

/// TUI asserter for direct PTY-based testing.
///
/// This asserter runs commands in a pseudo-terminal without tmux involvement.
/// It provides the fastest TUI testing experience with full key support.
pub struct PtyAsserter {
    backend: PtyBackend,
    terminal: TerminalBuffer,
    child: Box<dyn Child + Send + Sync>,
    settle_timeout_ms: u64,
}

impl PtyAsserter {
    pub(crate) fn new(
        backend: PtyBackend,
        rows: u16,
        cols: u16,
        child: Box<dyn Child + Send + Sync>,
        settle_timeout_ms: u64,
    ) -> Self {
        Self {
            backend,
            terminal: TerminalBuffer::new(rows, cols),
            child,
            settle_timeout_ms,
        }
    }

    /// Read available PTY output and process it into the terminal buffer.
    fn read_pty_output(&mut self) {
        if let Some(bytes) = self.backend.read_available() {
            self.terminal.process_bytes(&bytes);
        }
    }
}

impl std::fmt::Debug for PtyAsserter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyAsserter")
            .field("terminal_rows", &self.terminal.render().lines().count())
            .field("settle_timeout_ms", &self.settle_timeout_ms)
            .field("backend", &self.backend)
            .finish()
    }
}

impl TuiAsserter for PtyAsserter {
    fn wait_for_settle(&mut self) {
        self.wait_for_settle_ms(self.settle_timeout_ms, 5000);
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
        let start = Instant::now();
        while start.elapsed().as_millis() < ms as u128 {
            thread::sleep(Duration::from_millis(16));
            self.read_pty_output();
        }
    }

    fn expect_completion(&mut self) -> i32 {
        // Wait for the child process to exit and return its exit code
        match self.child.wait() {
            Ok(status) => status.exit_code() as i32,
            Err(e) => {
                panic!("Failed to wait for child process: {}", e);
            }
        }
    }

    fn expect_exit_code(&mut self, expected: i32) {
        let actual = self.expect_completion();
        if actual != expected {
            panic!(
                "Expected exit code {}, got {}. Screen:\n{}",
                expected,
                actual,
                self.screen()
            );
        }
    }

    fn type_text(&mut self, text: &str) {
        if let Err(e) = self.backend.write_bytes(text.as_bytes()) {
            panic!("Failed to type text: {}", e);
        }
    }

    fn press_key(&mut self, key: Key) {
        let bytes = KeyConversion::key_to_bytes(key);
        if let Err(e) = self.backend.write_bytes(&bytes) {
            panic!("Failed to press key: {}", e);
        }
    }

    fn send_keys(&mut self, keys: &[Key]) {
        // send_keys is designed for sending a single key combination.
        // Multiple independent keys should be sent via multiple press_key calls.
        if keys.len() > 1 {
            panic!(
                "send_keys can only send a single key at a time. Use press_key for multiple keys."
            );
        }
        for key in keys {
            self.press_key(key.clone());
        }
    }

    fn find_text(&self, text: &str) -> TextMatch {
        let positions = self.terminal.find_all_text(text);

        match positions.len() {
            0 => TextMatch::not_found(text),
            1 => {
                let (row, col) = positions[0];
                let cell = self.terminal.get_cell(row, col);
                let (fg, bg) = if let Some(cell) = cell {
                    (
                        ColorAssertion::from_rgb(cell.fg_r, cell.fg_g, cell.fg_b),
                        ColorAssertion::from_rgb(cell.bg_r, cell.bg_g, cell.bg_b),
                    )
                } else {
                    (ColorAssertion::not_found(), ColorAssertion::not_found())
                };
                TextMatch::new(text.to_string(), Some(row), Some(col), true, fg, bg)
            }
            _ => {
                panic!(
                    "'{}' found multiple times ({} occurrences). Use find_all_text() instead.\nScreen:\n{}",
                    text,
                    positions.len(),
                    self.screen()
                );
            }
        }
    }

    fn find_all_text(&self, text: &str) -> Vec<TextMatch> {
        self.terminal
            .find_all_text(text)
            .into_iter()
            .map(|(row, col)| {
                let cell = self.terminal.get_cell(row, col);
                let (fg, bg) = if let Some(cell) = cell {
                    (
                        ColorAssertion::from_rgb(cell.fg_r, cell.fg_g, cell.fg_b),
                        ColorAssertion::from_rgb(cell.bg_r, cell.bg_g, cell.bg_b),
                    )
                } else {
                    (ColorAssertion::not_found(), ColorAssertion::not_found())
                };
                TextMatch::new(text.to_string(), Some(row), Some(col), true, fg, bg)
            })
            .collect()
    }

    fn screen(&self) -> String {
        self.terminal.render()
    }

    fn dump_screen(&self) {
        eprintln!("=== Screen Dump ===");
        eprintln!("{}", self.screen());
        eprintln!("===================");
    }
}

impl Drop for PtyAsserter {
    fn drop(&mut self) {
        // Kill the child process if still running
        let _ = self.child.kill();
        // Wait to clean up the process
        let _ = self.child.wait();
    }
}
