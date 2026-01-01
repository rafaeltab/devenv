# tui-test

Framework-agnostic TUI testing library for terminal applications.

## Overview

`tui-test` enables testing any terminal user interface application through imperative interactions with a virtual PTY. It works with any TUI framework (Ratatui, Cursive, etc.) and even non-Rust applications.

## Status

🚧 **Under Development** - See [TUI_TEST_PLAN.md](../../TUI_TEST_PLAN.md) for implementation plan.

## Usage

```rust
use tui_test::{spawn_tui, Key, ColorMatcher};

let mut tui = spawn_tui("my-app", &["--flag"])
    .terminal_size(40, 120)
    .spawn()
    .expect("Failed to spawn");

tui.wait_for_settle();
tui.find_text("Welcome").assert_visible();
tui.type_text("hello");
tui.press_key(Key::Enter);

let exit_code = tui.expect_completion();
assert_eq!(exit_code, 0);
```

## License

Part of the rafaeltab CLI project.
