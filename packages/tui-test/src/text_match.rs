use crate::terminal::TerminalBuffer;
use crate::ColorAssertion;

pub struct TextMatch {
    text: String,
    position: Option<(u16, u16)>,
    screen_snapshot: TerminalBuffer,
    dump_on_fail: bool,
}

impl TextMatch {
    pub fn new(text: &str, terminal: &TerminalBuffer, dump_on_fail: bool) -> Self {
        let position = terminal.find_text(text);
        Self {
            text: text.to_string(),
            position,
            screen_snapshot: terminal.clone(),
            dump_on_fail,
        }
    }

    pub fn new_with_position(
        text: &str,
        position: Option<(u16, u16)>,
        terminal: &TerminalBuffer,
        dump_on_fail: bool,
    ) -> Self {
        Self {
            text: text.to_string(),
            position,
            screen_snapshot: terminal.clone(),
            dump_on_fail,
        }
    }

    pub fn assert_visible(&self) {
        if self.position.is_none() {
            if self.dump_on_fail {
                eprintln!("\n=== Screen Dump ===");
                eprintln!("{}", self.screen_snapshot.render());
                eprintln!("===================\n");
            }
            panic!(
                "assertion failed: text {:?} should be visible on screen\n\
                [Set TUI_TEST_DUMP_ON_FAIL=1 to see full screen output]",
                self.text
            );
        }
    }

    pub fn assert_not_visible(&self) {
        if self.position.is_some() {
            panic!(
                "assertion failed: text {:?} should NOT be visible on screen (found at {:?})",
                self.text, self.position
            );
        }
    }

    pub fn position(&self) -> Option<(u16, u16)> {
        self.position
    }

    pub fn fg(&self) -> ColorAssertion {
        let color = self.position.and_then(|(row, col)| {
            self.screen_snapshot
                .get_cell_color(row, col)
                .and_then(|c| c.fg)
        });

        ColorAssertion::new(
            color,
            format!("foreground color of text {:?}", self.text),
            self.dump_on_fail,
            &self.screen_snapshot,
        )
    }

    pub fn bg(&self) -> ColorAssertion {
        let color = self.position.and_then(|(row, col)| {
            self.screen_snapshot
                .get_cell_color(row, col)
                .and_then(|c| c.bg)
        });

        ColorAssertion::new(
            color,
            format!("background color of text {:?}", self.text),
            self.dump_on_fail,
            &self.screen_snapshot,
        )
    }
}
