use crate::{Key, TextMatch, TuiError};
use std::collections::HashMap;

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
        todo!("Implement spawning")
    }
}

pub struct TuiSession {
    // Will be implemented
}

impl TuiSession {
    // Lifecycle
    pub fn wait_for_settle(&mut self) {
        todo!()
    }

    pub fn wait_for_settle_ms(&mut self, _timeout_ms: u64, _max_wait_ms: u64) {
        todo!()
    }

    pub fn wait_ms(&mut self, _ms: u64) {
        todo!()
    }

    pub fn expect_completion(&mut self) -> i32 {
        todo!()
    }

    pub fn expect_exit_code(&mut self, _expected: i32) {
        todo!()
    }

    // Input
    pub fn type_text(&mut self, _text: &str) {
        todo!()
    }

    pub fn press_key(&mut self, _key: Key) {
        todo!()
    }

    pub fn send_keys(&mut self, _keys: &[Key]) {
        todo!()
    }

    // Queries
    pub fn find_text(&self, _text: &str) -> TextMatch {
        todo!()
    }

    pub fn find_all_text(&self, _text: &str) -> Vec<TextMatch> {
        todo!()
    }

    pub fn screen(&self) -> String {
        todo!()
    }

    // Debug
    pub fn dump_screen(&self) {
        todo!()
    }
}
