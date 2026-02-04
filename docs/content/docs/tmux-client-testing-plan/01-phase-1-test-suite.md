---
title: Phase 1 Test Suite Definition
---

# Phase 1: Test Suite Definition

**Goal:** Define comprehensive test cases for all tester types. Tests will not compile yet.

**Location:** `packages/test-descriptors/tests/testers/`

## Test Categories

### 1. TuiAsserter Core Tests

These tests verify the core TuiAsserter functionality that is shared across all TUI testers.

#### 1.1 Lifecycle Tests

| Test Name                             | Description                                                            |
| ------------------------------------- | ---------------------------------------------------------------------- |
| `spawn_simple_command`                | Spawn a simple command (echo) and verify it completes with exit code 0 |
| `spawn_with_custom_env`               | Spawn with custom environment variable, verify it's accessible         |
| `spawn_with_custom_terminal_size`     | Set custom terminal size, verify via `tput cols`                       |
| `spawn_with_custom_cwd`               | Set working directory, verify via `pwd`                                |
| `expect_completion_returns_exit_code` | Run `exit 42`, verify exit code is 42                                  |
| `expect_exit_code_matches`            | Verify `expect_exit_code(0)` passes for successful command             |
| `expect_exit_code_panics_on_mismatch` | Verify `expect_exit_code(0)` panics when exit code is 42               |

#### 1.2 Wait/Settle Tests

| Test Name                               | Description                                 |
| --------------------------------------- | ------------------------------------------- |
| `wait_ms_delays_execution`              | Verify `wait_ms(100)` delays at least 100ms |
| `wait_for_settle_detects_stable_screen` | Echo command should settle quickly          |
| `wait_for_settle_with_custom_timeout`   | Custom settle timeout works                 |
| `wait_for_settle_max_wait_timeout`      | Continuous output respects max_wait         |

#### 1.3 Text Finding Tests

| Test Name                              | Description                                      |
| -------------------------------------- | ------------------------------------------------ |
| `find_text_returns_position`           | Find "Hello" in "Hello World", position is Some  |
| `find_text_not_found_returns_none`     | Find "Goodbye" in "Hello", position is None      |
| `find_text_panics_on_multiple_matches` | Find "test" when it appears twice, should panic  |
| `find_all_text_returns_all_positions`  | Find all "test" when it appears twice            |
| `find_all_text_empty_when_not_found`   | Find all "Goodbye" in "Hello", returns empty vec |
| `find_text_exact_match_only`           | Finds exact substrings                           |
| `find_text_case_sensitive`             | "Hello" != "hello"                               |

#### 1.4 Assertion Tests

| Test Name                            | Description                                     |
| ------------------------------------ | ----------------------------------------------- |
| `assert_visible_succeeds`            | `assert_visible()` passes when text exists      |
| `assert_visible_fails_with_message`  | `assert_visible()` panics with helpful message  |
| `assert_not_visible_succeeds`        | `assert_not_visible()` passes when text absent  |
| `assert_not_visible_fails`           | `assert_not_visible()` panics when text present |
| `text_match_position_returns_coords` | Position returns (row, col) tuple               |

#### 1.5 Input Tests

| Test Name                           | Description                               |
| ----------------------------------- | ----------------------------------------- |
| `type_text_sends_to_pty`            | Type "hello" into cat, verify echoed back |
| `press_key_enter`                   | Press Enter, verify newline               |
| `press_key_arrows`                  | Press arrow keys, don't crash             |
| `press_key_esc`                     | Press Esc, don't crash                    |
| `press_key_backspace`               | Press Backspace, verify character deleted |
| `send_keys_ctrl_c`                  | Ctrl+C interrupts running process         |
| `send_keys_requires_non_modifier`   | Panics when only modifiers sent           |
| `send_keys_single_regular_key_only` | Panics when multiple regular keys sent    |
| `send_keys_alt_a`                   | Alt+A detected by key detector program    |
| `send_keys_shift_up`                | Shift+Up detected                         |
| `send_keys_ctrl_shift_r`            | Ctrl+Shift+R detected                     |

#### 1.6 Screen Tests

| Test Name                       | Description                       |
| ------------------------------- | --------------------------------- |
| `screen_returns_full_buffer`    | `screen()` contains output text   |
| `screen_reflects_current_state` | Screen changes after input        |
| `dump_screen_prints_to_stderr`  | `dump_screen()` outputs to stderr |

#### 1.7 Color Tests

| Test Name                        | Description                   |
| -------------------------------- | ----------------------------- |
| `color_matcher_grayscale`        | Grayscale detection works     |
| `color_matcher_yellowish`        | Yellow hue detection works    |
| `color_matcher_redish`           | Red hue detection works       |
| `color_matcher_greenish`         | Green hue detection works     |
| `color_matcher_blueish`          | Blue hue detection works      |
| `color_matcher_cyanish`          | Cyan hue detection works      |
| `color_matcher_magentaish`       | Magenta hue detection works   |
| `color_matcher_custom_hue_range` | Custom hue range works        |
| `color_assertion_fg_exact`       | Exact foreground RGB matching |
| `color_assertion_bg_matcher`     | Background color matcher      |

#### 1.8 Terminal Sequence Tests

| Test Name                       | Description                           |
| ------------------------------- | ------------------------------------- |
| `test_clear_screen_sequence`    | ESC[2J clears screen                  |
| `test_cursor_positioning`       | ESC[row;colH positions cursor         |
| `test_erase_in_line`            | ESC[K erases to end of line           |
| `test_cursor_save_restore`      | ESC[s/ESC[u save/restore cursor       |
| `test_cursor_movement`          | ESC[A/B/C/D move cursor               |
| `test_scrolling_region`         | ESC[top;bottomr sets scroll region    |
| `test_insert_delete_characters` | ESC[@n/ESC[Pn insert/delete chars     |
| `test_insert_delete_lines`      | ESC[Ln/ESC[Mn insert/delete lines     |
| `test_overwrite_mode`           | Characters overwrite existing content |
| `test_tab_stops`                | Tab stops at column 8, 16, 24...      |
| `test_reverse_video`            | ESC[7m swaps fg/bg                    |
| `test_256_color_support`        | ESC[38;5;nm 256-color mode            |
| `test_rgb_color_support`        | ESC[38;2;r;g;bm true color            |

#### 1.9 Integration Tests

| Test Name                     | Description                       |
| ----------------------------- | --------------------------------- |
| `interactive_menu_navigation` | Multi-step menu interaction       |
| `text_input_echo`             | Type name, verify greeting        |
| `colored_output_detection`    | Multiple colors in output         |
| `multi_screen_interaction`    | Navigate through multiple screens |

### 2. Command Tests

Tests for `CommandResult` based testers (cmd, tmux_client_cmd).

#### 2.1 Basic Execution Tests

| Test Name                        | Description                                   |
| -------------------------------- | --------------------------------------------- |
| `run_simple_command`             | Run `echo hello`, verify stdout               |
| `run_command_with_args`          | Run with multiple arguments                   |
| `run_command_captures_stderr`    | Verify stderr is captured separately          |
| `run_command_captures_exit_code` | Non-zero exit code captured                   |
| `run_command_with_env_var`       | Environment variable is accessible            |
| `run_command_with_cwd`           | Working directory is set correctly            |
| `run_command_success_flag`       | `success` is true when exit_code is 0         |
| `run_command_failure_flag`       | `success` is false when exit_code is non-zero |

### 3. Tester-Specific Tests

#### 3.1 CmdTester Tests

| Test Name                      | Description                         |
| ------------------------------ | ----------------------------------- |
| `cmd_tester_runs_outside_tmux` | $TMUX env var is not set            |
| `cmd_tester_inherits_env`      | Command env vars are passed through |

#### 3.2 TmuxClientCmdTester Tests

| Test Name                                 | Description                     |
| ----------------------------------------- | ------------------------------- |
| `tmux_client_cmd_runs_inside_client`      | $TMUX env var is set            |
| `tmux_client_cmd_panics_without_client`   | Panics if no client was created |
| `tmux_client_cmd_separates_stdout_stderr` | stdout and stderr are separate  |
| `tmux_client_cmd_captures_exit_code`      | Exit code is correct            |

#### 3.3 PtyTester Tests

| Test Name                         | Description              |
| --------------------------------- | ------------------------ |
| `pty_tester_runs_outside_tmux`    | $TMUX env var is not set |
| `pty_tester_full_tui_interaction` | Full TUI test works      |

#### 3.4 TmuxClientPtyTester Tests

| Test Name                                  | Description                       |
| ------------------------------------------ | --------------------------------- |
| `tmux_client_pty_captures_pane_output`     | Output captured via capture-pane  |
| `tmux_client_pty_sends_keys`               | Keys sent via send-keys           |
| `tmux_client_pty_panics_without_client`    | Panics if no client               |
| `tmux_client_pty_captures_colors`          | ANSI colors captured with -e flag |
| `tmux_client_pty_supports_cursor_position` | Cursor positioning works          |

#### 3.5 TmuxFullClientTester Tests

| Test Name                                | Description               |
| ---------------------------------------- | ------------------------- |
| `tmux_full_client_shows_tmux_ui`         | Tmux status bar visible   |
| `tmux_full_client_panics_without_client` | Panics if no client       |
| `tmux_full_client_full_pty_interaction`  | Full PTY keyboard support |

### 4. Client Management Tests

#### 4.1 Client Creation Tests

| Test Name                               | Description                        |
| --------------------------------------- | ---------------------------------- |
| `with_client_creates_tmux_client`       | Client is spawned                  |
| `with_client_attaches_to_session`       | Client attached to correct session |
| `with_client_errors_if_session_missing` | Error if session doesn't exist     |
| `with_client_only_one_allowed`          | Second `with_client()` errors      |
| `with_client_respects_pty_size`         | PTY size is set correctly          |

#### 4.2 Client Query Tests

| Test Name                                  | Description                       |
| ------------------------------------------ | --------------------------------- |
| `has_tmux_client_returns_true`             | Returns true when client exists   |
| `has_tmux_client_returns_false`            | Returns false when no client      |
| `tmux_client_returns_handle`               | Returns client handle when exists |
| `tmux_client_returns_none`                 | Returns None when no client       |
| `current_session_returns_attached_session` | Returns session name              |
| `current_session_updates_after_switch`     | Returns new session after switch  |

#### 4.3 Client Cleanup Tests

| Test Name                     | Description                            |
| ----------------------------- | -------------------------------------- |
| `client_killed_on_env_drop`   | Client process killed when env dropped |
| `client_killed_before_server` | Client killed before tmux server       |

### 5. Command Builder Tests

#### 5.1 Command Struct Tests

| Test Name                        | Description                |
| -------------------------------- | -------------------------- |
| `command_new_sets_program`       | Program is set correctly   |
| `command_args_adds_arguments`    | Arguments are added        |
| `command_env_adds_variable`      | Environment variable added |
| `command_cwd_sets_directory`     | Working directory set      |
| `command_build_args_returns_all` | All args returned in order |
| `command_build_env_returns_all`  | All env vars returned      |

## Test File Structure

```
packages/test-descriptors/tests/
  testers/
    mod.rs                      # Module declarations
    lifecycle_tests.rs          # 1.1 Lifecycle Tests
    wait_settle_tests.rs        # 1.2 Wait/Settle Tests
    text_finding_tests.rs       # 1.3 Text Finding Tests
    assertion_tests.rs          # 1.4 Assertion Tests
    input_tests.rs              # 1.5 Input Tests
    screen_tests.rs             # 1.6 Screen Tests
    color_tests.rs              # 1.7 Color Tests
    terminal_sequence_tests.rs  # 1.8 Terminal Sequence Tests
    integration_tests.rs        # 1.9 Integration Tests
    command_tests.rs            # 2.1 Basic Execution Tests
    cmd_tester_tests.rs         # 3.1 CmdTester Tests
    tmux_client_cmd_tests.rs    # 3.2 TmuxClientCmdTester Tests
    pty_tester_tests.rs         # 3.3 PtyTester Tests
    tmux_client_pty_tests.rs    # 3.4 TmuxClientPtyTester Tests
    tmux_full_client_tests.rs   # 3.5 TmuxFullClientTester Tests
    client_creation_tests.rs    # 4.1 Client Creation Tests
    client_query_tests.rs       # 4.2 Client Query Tests
    client_cleanup_tests.rs     # 4.3 Client Cleanup Tests
    command_builder_tests.rs    # 5.1 Command Struct Tests
  test_programs/                # Helper programs for testing
    mod.rs
    echo_program.rs
    menu_program.rs
    key_detector.rs
    colored_program.rs
    multiscreen_program.rs
    ... terminal sequence test programs
```

## Test Programs

The following test programs are needed (compile with rustc during tests):

1. **echo_program** - Prompts for input, echoes back with greeting
2. **menu_program** - Interactive menu with arrow key navigation
3. **key_detector** - Detects and prints key combinations (Alt+A, Shift+Up, etc.)
4. **colored_program** - Outputs text in various colors
5. **multiscreen_program** - Multi-screen navigation with clear screen between
6. **clear_screen_test** - Tests ESC[2J clear screen
7. **cursor_position_test** - Tests ESC[row;colH positioning
8. **erase_line_test** - Tests ESC[K erase to end of line
9. **cursor_save_restore_test** - Tests ESC[s/ESC[u
10. **cursor_movement_test** - Tests ESC[A/B/C/D
11. **scrolling_region_test** - Tests ESC[top;bottomr
12. **insert_chars_test** - Tests ESC[@n
13. **insert_lines_test** - Tests ESC[Ln
14. **overwrite_test** - Tests character overwrite
15. **tab_stops_test** - Tests tab character handling
16. **reverse_video_test** - Tests ESC[7m
17. **color_256_test** - Tests ESC[38;5;nm
18. **rgb_color_test** - Tests ESC[38;2;r;g;bm

## Notes

- Tests should be parameterized where possible to run against multiple tester types
- Terminal sequence tests should run against all TUI testers to verify consistency
- Test programs can be copied from existing `tui-test` package
- Some tests may need to be skipped for certain tester types (e.g., complex key combos for capture-pane)

## Deliverables

1. Test file stubs with all test functions defined
2. Test helper module for compiling test programs
3. Test program source files
4. Module declarations in `lib.rs`
