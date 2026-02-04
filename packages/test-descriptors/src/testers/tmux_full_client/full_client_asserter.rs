use crate::testers::keys::Key;
use crate::testers::text_match::TextMatch;
use crate::testers::tui_asserter::TuiAsserter;

/// TUI asserter that captures the full tmux client output including tmux UI.
#[derive(Debug)]
pub struct FullClientAsserter {
    // Internal: PTY reader/writer, terminal buffer, etc.
    _private: (),
}

impl FullClientAsserter {
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl TuiAsserter for FullClientAsserter {
    fn wait_for_settle(&mut self) {
        todo!("Phase 3: Implement FullClientAsserter::wait_for_settle")
    }

    fn wait_for_settle_ms(&mut self, _timeout_ms: u64, _max_wait_ms: u64) {
        todo!("Phase 3: Implement FullClientAsserter::wait_for_settle_ms")
    }

    fn wait_ms(&mut self, _ms: u64) {
        todo!("Phase 3: Implement FullClientAsserter::wait_ms")
    }

    fn expect_completion(&mut self) -> i32 {
        todo!("Phase 3: Implement FullClientAsserter::expect_completion")
    }

    fn expect_exit_code(&mut self, _expected: i32) {
        todo!("Phase 3: Implement FullClientAsserter::expect_exit_code")
    }

    fn type_text(&mut self, _text: &str) {
        todo!("Phase 3: Implement FullClientAsserter::type_text")
    }

    fn press_key(&mut self, _key: Key) {
        todo!("Phase 3: Implement FullClientAsserter::press_key")
    }

    fn send_keys(&mut self, _keys: &[Key]) {
        todo!("Phase 3: Implement FullClientAsserter::send_keys")
    }

    fn find_text(&self, _text: &str) -> TextMatch {
        todo!("Phase 3: Implement FullClientAsserter::find_text")
    }

    fn find_all_text(&self, _text: &str) -> Vec<TextMatch> {
        todo!("Phase 3: Implement FullClientAsserter::find_all_text")
    }

    fn screen(&self) -> String {
        todo!("Phase 3: Implement FullClientAsserter::screen")
    }

    fn dump_screen(&self) {
        todo!("Phase 3: Implement FullClientAsserter::dump_screen")
    }
}
