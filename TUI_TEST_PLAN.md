# TUI Test Package Implementation Plan

## Overview

Create a framework-agnostic TUI testing package at `packages/tui-test/` that enables testing any terminal user interface application through imperative interactions with a virtual PTY.

## Design Decisions

### Dependencies

- **`portable-pty` (v0.9)**: Cross-platform PTY spawning and management
- **`alacritty_terminal` (v0.25)**: Production-grade terminal emulator that handles all VT100/ANSI escape sequences, colors, cursor positioning, and terminal state. This eliminates the need to write our own terminal parser.
- **`thiserror` (v1)**: Error handling

### Key Design Principles

1. **Framework-agnostic**: Must work with any TUI application, not just Rust/Ratatui apps
2. **Imperative API**: Interactions are actions (not declarative descriptors)
3. **Owned snapshots**: `TextMatch` captures screen state at time of search
4. **Clean separation**: Package knows nothing about the CLI application being tested

### Configuration

- **Settle timeout**: Default 300ms, configurable via:
  1. Per-call: `wait_for_settle_ms(500)`
  2. Per-session: `.settle_timeout(500)`
  3. Environment: `TUI_TEST_SETTLE_MS=500`
- **Screen dump on failure**: Disabled by default, configurable via:

  1. Per-session: `.dump_on_fail(true)`
  2. Environment: `TUI_TEST_DUMP_ON_FAIL=1`

- **Max settle wait**: 1 second default, configurable per-call

### Terminal Emulation

- Use `alacritty_terminal` for full terminal emulation
- Terminal size: 40 rows × 120 cols (configurable per-session)
- Process cleanup: `TuiSession` implements `Drop` to ensure child processes are killed

## Public API

### Entry Point

```rust
pub fn spawn_tui(command: &str, args: &[&str]) -> TuiSessionBuilder
```

### Builder Pattern

```rust
pub struct TuiSessionBuilder {
    pub fn env(self, key: &str, value: &str) -> Self
    pub fn terminal_size(self, rows: u16, cols: u16) -> Self
    pub fn settle_timeout(self, ms: u64) -> Self
    pub fn dump_on_fail(self, enabled: bool) -> Self
    pub fn spawn(self) -> Result<TuiSession, TuiError>
}
```

### Session API

```rust
pub struct TuiSession {
    // Lifecycle
    pub fn wait_for_settle(&mut self)
    pub fn wait_for_settle_ms(&mut self, timeout_ms: u64, max_wait_ms: u64)
    pub fn wait_ms(&mut self, ms: u64)
    pub fn expect_completion(&mut self) -> i32
    pub fn expect_exit_code(&mut self, expected: i32)

    // Input
    pub fn type_text(&mut self, text: &str)
    pub fn press_key(&mut self, key: Key)
    pub fn send_keys(&mut self, keys: &[Key])

    // Queries
    pub fn find_text(&self, text: &str) -> TextMatch
    pub fn find_all_text(&self, text: &str) -> Vec<TextMatch>
    pub fn screen(&self) -> String

    // Debug
    pub fn dump_screen(&self)
}
```

### Text Matching

```rust
pub struct TextMatch {
    pub fg: ColorAssertion
    pub bg: ColorAssertion

    pub fn assert_visible(&self)
    pub fn assert_not_visible(&self)
    pub fn position(&self) -> Option<(u16, u16)>
}
```

### Color Assertions

```rust
pub struct ColorAssertion {
    pub fn assert(&self, matcher: ColorMatcher)
    pub fn exact(&self, r: u8, g: u8, b: u8)
}

pub enum ColorMatcher {
    Grayscale,
    YellowIsh,
    RedIsh,
    GreenIsh,
    BlueIsh,
    CyanIsh,
    MagentaIsh,
    Hue { min: f32, max: f32 },
    Saturation { min: f32, max: f32 },
    Lightness { min: f32, max: f32 },
}
```

### Key Input

```rust
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Tab,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Ctrl,
    Alt,
    Shift,
    Super,
}
```

**Key combinations**: Send using `send_keys(&[Key::Ctrl, Key::Char('c')])` which sends all modifiers as key_down, then the regular key, then all as key_up.

### Error Handling

```rust
pub enum TuiError {
    PtySpawn(String),
    PtyWrite(io::Error),
    PtyRead(io::Error),
    UnexpectedExit(i32),
    ProcessStillRunning,
}
```

## Package Structure

```
packages/tui-test/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── session.rs          # TuiSession and TuiSessionBuilder
│   ├── terminal.rs         # Terminal buffer wrapper
│   ├── keys.rs             # Key enum and input handling
│   ├── color.rs            # ColorMatcher and HSL utilities
│   ├── text_match.rs       # TextMatch and assertions
│   └── pty_manager.rs      # PTY spawning and I/O
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
└── README.md
```

## Test-Driven Development Plan

### Phase 0: Package Setup

- Create `packages/tui-test/` directory structure
- Create `Cargo.toml` with dependencies
- Create initial module files (`lib.rs`, `session.rs`, etc.)
- Add package to workspace
- Verify package builds

### Phase 1: Test Suite Design

#### Basic Lifecycle Tests

- **spawn_simple_command**: Spawn `echo "hello"` and verify it completes
- **spawn_with_custom_env**: Spawn with environment variable and verify it's set
- **spawn_with_custom_terminal_size**: Verify terminal reports correct size
- **process_cleanup_on_drop**: Verify child process is killed when session drops
- **expect_completion_returns_exit_code**: Verify exit code is returned correctly
- **expect_exit_code_matches**: Verify assertion succeeds for correct exit code
- **expect_exit_code_panics_on_mismatch**: Verify panic on wrong exit code

#### Wait and Settle Tests

- **wait_ms_delays_execution**: Verify `wait_ms()` actually waits
- **wait_for_settle_detects_stable_screen**: Verify settle when screen stops changing
- **wait_for_settle_with_custom_timeout**: Verify custom settle timeout works
- **wait_for_settle_max_wait_timeout**: Verify max wait prevents infinite loops
- **wait_for_settle_with_continuously_changing_output**: Verify timeout handling

#### Text Input Tests

- **type_text_sends_to_pty**: Verify text appears in application
- **press_key_enter**: Verify Enter key works
- **press_key_arrows**: Verify arrow keys work
- **press_key_esc**: Verify Escape key works
- **press_key_backspace**: Verify backspace works
- **send_keys_ctrl_c**: Verify Ctrl+C combination
- **send_keys_ctrl_shift_r**: Verify multiple modifiers
- **send_keys_requires_non_modifier**: Verify panic if only modifiers
- **send_keys_single_regular_key_only**: Verify panic if multiple regular keys

#### Text Finding Tests

- **find_text_returns_position**: Verify text is found and position returned
- **find_text_not_found_returns_none**: Verify None when text not present
- **find_text_panics_on_multiple_matches**: Verify panic on duplicate matches
- **find_all_text_returns_all_positions**: Verify all occurrences returned
- **find_all_text_empty_when_not_found**: Verify empty vec when not found
- **find_text_exact_match_only**: Verify no partial matches
- **find_text_case_sensitive**: Verify case matters

#### Text Match Assertions

- **assert_visible_succeeds**: Verify assertion passes when text found
- **assert_visible_fails_with_message**: Verify panic with helpful message
- **assert_visible_dumps_screen_when_enabled**: Verify screen dump appears
- **assert_not_visible_succeeds**: Verify assertion passes when not found
- **assert_not_visible_fails**: Verify panic when text is visible
- **text_match_position_returns_coords**: Verify position() returns correct coords
- **text_match_is_snapshot**: Verify TextMatch uses captured screen state

#### Color Matching Tests

- **color_matcher_grayscale**: Verify grayscale detection
- **color_matcher_yellowish**: Verify yellow hue range
- **color_matcher_redish**: Verify red hue with wrap-around
- **color_matcher_greenish**: Verify green hue range
- **color_matcher_blueish**: Verify blue hue range
- **color_matcher_custom_hue_range**: Verify custom HSL ranges
- **color_assertion_fg_exact**: Verify exact RGB match
- **color_assertion_bg_matcher**: Verify background color matching
- **color_assertion_no_color_set**: Verify behavior with default terminal colors

#### Screen Capture Tests

- **screen_returns_full_buffer**: Verify screen() captures complete output
- **screen_reflects_current_state**: Verify screen updates with changes
- **dump_screen_prints_to_stderr**: Verify dump_screen() outputs correctly

#### Configuration Tests

- **builder_default_values**: Verify builder defaults
- **builder_env_var_settle_timeout**: Verify TUI_TEST_SETTLE_MS works
- **builder_env_var_dump_on_fail**: Verify TUI_TEST_DUMP_ON_FAIL works
- **builder_precedence_explicit_over_env**: Verify explicit config overrides env

#### Integration Tests (with real commands)

- **interactive_menu_navigation**: Test with a simple menu program
- **text_input_echo**: Test with a text input that echoes back
- **colored_output_detection**: Test with a program that outputs colors
- **multi_screen_interaction**: Test program with multiple screens/states

### Phase 2: Core Implementation

- Implement `PtyManager` using `portable-pty`
- Integrate `alacritty_terminal` for terminal emulation
- Implement `TuiSessionBuilder` with configuration precedence
- Implement basic `TuiSession` lifecycle

### Phase 3: Input & Interaction

- Implement `type_text()` and `press_key()`
- Implement `send_keys()` with modifier handling
- Implement key-to-bytes conversion for PTY
- Implement `wait_for_settle()` with timeout logic

### Phase 4: Text Finding & Assertions

- Implement terminal buffer wrapper around `alacritty_terminal`
- Implement `find_text()` with duplicate detection
- Implement `find_all_text()`
- Implement `TextMatch` with owned snapshot
- Implement visibility assertions

### Phase 5: Color Support

- Implement RGB to HSL conversion
- Implement `ColorMatcher` with all variants
- Implement `ColorAssertion` with fg/bg support
- Extract colors from terminal buffer cells

### Phase 6: Error Handling & Debug

- Implement screen dump on assertion failure
- Implement `dump_screen()` method
- Proper error messages with context
- Implement `Drop` for process cleanup

### Phase 7: CLI Integration

- Add `run_cli_tui()` helper to `apps/cli/tests/common/`
- Write real tests for workspace switcher
- Write real tests for command palette
- Verify end-to-end functionality

### Phase 8: Documentation

- Comprehensive README with examples
- API documentation with doc comments
- Usage examples for common patterns

## Example Usage Pattern

```rust
// In apps/cli/tests/common/mod.rs
use tui_test::{spawn_tui, TuiSession};

pub fn run_cli_tui(args: &[&str], config_path: &str, tmux_socket: &str) -> TuiSession {
    let mut full_args = vec!["--config", config_path];
    full_args.extend_from_slice(args);

    spawn_tui(env!("CARGO_BIN_EXE_rafaeltab"), &full_args)
        .env("RAFAELTAB_TMUX_SOCKET", tmux_socket)
        .terminal_size(40, 120)
        .settle_timeout(300)
        .spawn()
        .expect("Failed to spawn TUI")
}

// In a test
#[test]
fn test_workspace_switcher() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("project-a", Some("Project A"), &[("shell", None)]);
            c.tmux_session("project-b", Some("Project B"), &[("shell", None)]);
        });
    }).create();

    let mut tui = run_cli_tui(&["tmux", "switch"], config_path, env.tmux_socket());

    tui.wait_for_settle();
    tui.find_text("Project A").assert_visible();
    tui.find_text("Project B").assert_visible();

    tui.type_text("B");
    tui.wait_for_settle();

    tui.find_text("Project B").fg.assert(ColorMatcher::YellowIsh);

    tui.press_key(Key::Enter);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}
```

## Implementation Notes

### Wait for Settle Algorithm

1. Check screen every 16ms (~60fps)
2. If screen unchanged for settle_timeout_ms → settled
3. If screen changes → reset counter
4. If max_wait_ms exceeded → timeout (default 1 second)

### Text Finding Behavior

- Exact string matching only (no regex, no partial matches)
- Case-sensitive
- `find_text()` panics if multiple occurrences found (use `find_all_text()`)
- Returns position as (row, col) 0-indexed

### Key Combination Handling

- Modifiers: `Ctrl`, `Alt`, `Shift`, `Super`
- Must include exactly one non-modifier key
- Sends as: key_down(all modifiers) → key_down(regular) → key_up(regular) → key_up(all modifiers)
- Generates proper xterm escape sequences

### Color Matching Strategy

- Use HSL color space for "ish" matching (more perceptually accurate)
- Grayscale: saturation < 0.1
- Color "ish" variants: hue range ± 30° with saturation > 0.3
- Allow custom HSL range matching for fine-grained control

### Error Messages

- Assertion failures show clear message
- Screen dump only when explicitly enabled
- Include context (what was being searched, expected vs actual)

## Future Enhancements (Out of Scope)

- Snapshot comparison with golden files
- Mouse input simulation
- Paste events
- Window resize handling
- Multiple terminal sessions
- Async/await API
