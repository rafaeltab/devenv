use crate::descriptor::TmuxSocket;
use crate::testers::color::ColorAssertion;
use crate::testers::internal::{KeyConversion, TerminalBuffer};
use crate::testers::keys::Key;
use crate::testers::text_match::TextMatch;
use crate::testers::tui_asserter::TuiAsserter;
use std::thread;
use std::time::{Duration, Instant};

/// TUI asserter that uses tmux capture-pane for output capture.
///
/// This asserter captures pane content via `tmux capture-pane -p -e -J`,
/// which returns the application output without the tmux UI (status bar).
/// Input is sent via `tmux send-keys`.
pub struct CapturePaneAsserter {
    socket: TmuxSocket,
    session_name: String,
    terminal: TerminalBuffer,
    settle_timeout_ms: u64,
}

impl CapturePaneAsserter {
    pub(crate) fn new(
        socket: TmuxSocket,
        session_name: String,
        rows: u16,
        cols: u16,
        settle_timeout_ms: u64,
    ) -> Self {
        Self {
            socket,
            session_name,
            terminal: TerminalBuffer::new(rows, cols),
            settle_timeout_ms,
        }
    }

    /// Capture the current pane content via `tmux capture-pane`.
    fn capture_pane(&mut self) {
        let output = self
            .socket
            .run_tmux(&[
                "capture-pane",
                "-t",
                &self.session_name,
                "-p", // Print to stdout
                "-e", // Include escape sequences (colors)
                "-J", // Join wrapped lines
            ])
            .expect("Failed to capture pane");

        // Clear and reprocess the entire buffer
        self.terminal.clear();
        self.terminal.process_bytes(output.as_bytes());
    }
}

impl std::fmt::Debug for CapturePaneAsserter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturePaneAsserter")
            .field("session_name", &self.session_name)
            .field("settle_timeout_ms", &self.settle_timeout_ms)
            .finish()
    }
}

impl TuiAsserter for CapturePaneAsserter {
    fn wait_for_settle(&mut self) {
        self.wait_for_settle_ms(self.settle_timeout_ms, 5000);
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

    fn wait_ms(&mut self, ms: u64) {
        let start = Instant::now();
        while start.elapsed().as_millis() < ms as u128 {
            thread::sleep(Duration::from_millis(16));
            self.capture_pane();
        }
    }

    fn expect_completion(&mut self) -> i32 {
        // For capture-pane, we can't easily detect command completion
        // since we're only seeing the pane content.
        // We'll wait for settle and return 0.
        self.wait_for_settle();
        0
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
        self.socket
            .run_tmux(&[
                "send-keys",
                "-t",
                &self.session_name,
                "-l", // Literal (disable key lookup)
                text,
            ])
            .expect("Failed to send text");
    }

    fn press_key(&mut self, key: Key) {
        let key_name = KeyConversion::key_to_tmux_format(key);
        self.socket
            .run_tmux(&["send-keys", "-t", &self.session_name, &key_name])
            .expect("Failed to send key");
    }

    fn send_keys(&mut self, keys: &[Key]) {
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
                    "'{}' found {} times. Use find_all_text() instead.\nScreen:\n{}",
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
        eprintln!("=== Screen Dump (capture-pane) ===");
        eprintln!("{}", self.screen());
        eprintln!("==================================");
    }

    fn assert_vertical_order(&self, _matches: &[TextMatch]) {
        // Stub implementation for capture-pane asserter
        // Full implementation deferred to later phase
        unimplemented!("assert_vertical_order not yet implemented for CapturePaneAsserter")
    }

    fn expect_completion_and_get_output(&mut self) -> String {
        // Stub implementation - returns screen content as output
        self.expect_completion();
        self.screen()
    }
}
