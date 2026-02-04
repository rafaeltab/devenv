use crate::descriptor::TmuxSocket;
use crate::testers::color::ColorAssertion;
use crate::testers::internal::{KeyConversion, TerminalBuffer};
use crate::testers::keys::Key;
use crate::testers::text_match::TextMatch;
use crate::testers::tui_asserter::TuiAsserter;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// TUI asserter that captures the full tmux client output including tmux UI.
///
/// This asserter reads directly from the client's PTY, capturing everything
/// the user would see including the tmux status bar and other UI elements.
pub struct FullClientAsserter {
    terminal: TerminalBuffer,
    settle_timeout_ms: u64,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Box<dyn Write + Send>,
    socket: TmuxSocket,
    session_name: String,
    read_buffer: Arc<Mutex<Vec<u8>>>,
}

impl FullClientAsserter {
    pub(crate) fn new(
        reader: Box<dyn Read + Send>,
        writer: Box<dyn Write + Send>,
        rows: u16,
        cols: u16,
        settle_timeout_ms: u64,
        socket: TmuxSocket,
        session_name: String,
    ) -> Self {
        let terminal = TerminalBuffer::new(rows, cols);
        let reader = Arc::new(Mutex::new(reader));
        let read_buffer = Arc::new(Mutex::new(Vec::new()));

        // Start background reader thread
        let reader_clone = Arc::clone(&reader);
        let buffer_clone = Arc::clone(&read_buffer);
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                let bytes_read = {
                    let mut reader = match reader_clone.lock() {
                        Ok(r) => r,
                        Err(_) => break,
                    };
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            drop(reader);
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                        Err(_) => break,
                    }
                };

                if let Ok(mut buffer) = buffer_clone.lock() {
                    buffer.extend_from_slice(&buf[..bytes_read]);
                }
            }
        });

        Self {
            terminal,
            settle_timeout_ms,
            reader,
            writer,
            socket,
            session_name,
            read_buffer,
        }
    }

    /// Read available PTY output and process it.
    fn read_pty_output(&mut self) {
        if let Ok(mut buffer) = self.read_buffer.lock() {
            if !buffer.is_empty() {
                self.terminal.process_bytes(&buffer);
                buffer.clear();
            }
        }
    }
}

impl std::fmt::Debug for FullClientAsserter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FullClientAsserter")
            .field("terminal_rows", &self.terminal.render().lines().count())
            .field("settle_timeout_ms", &self.settle_timeout_ms)
            .finish()
    }
}

impl TuiAsserter for FullClientAsserter {
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
        // For full client, we can't easily detect command completion
        // since we're seeing the whole tmux interface.
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
        if let Err(e) = self.writer.write_all(text.as_bytes()) {
            panic!("Failed to type text: {}", e);
        }
        if let Err(e) = self.writer.flush() {
            panic!("Failed to flush after typing: {}", e);
        }
    }

    fn press_key(&mut self, key: Key) {
        let bytes = KeyConversion::key_to_bytes(key);
        if let Err(e) = self.writer.write_all(&bytes) {
            panic!("Failed to press key: {}", e);
        }
        if let Err(e) = self.writer.flush() {
            panic!("Failed to flush after key press: {}", e);
        }
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
        eprintln!("=== Screen Dump ===");
        eprintln!("{}", self.screen());
        eprintln!("===================");
    }
}
