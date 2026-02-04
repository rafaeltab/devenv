---
title: Phase 10 Migrate Existing CLI Tests
---

# Phase 10: Migrate Existing CLI Tests

**Goal:** Fix all compile errors from Phase 9 by migrating existing CLI tests to use the new tester infrastructure.

**Prerequisite:** Phases 1-9 complete (all testers working, old code removed, compile errors present)

## Overview

After Phase 9, the following test files have compile errors due to removed `CliTestRunner`, `TuiCliTestRunner`, and `tui_test` imports. This phase migrates each file to use the new infrastructure.

## Files to Migrate

| File                              | Old Patterns Used                                                           | New Pattern                                                    |
| --------------------------------- | --------------------------------------------------------------------------- | -------------------------------------------------------------- |
| `cli_integration_tests.rs`        | `CliTestRunner.run()`                                                       | `testers().cmd().run()`                                        |
| `config_flag_tests.rs`            | `CliTestRunner.with_config().run()`                                         | `testers().cmd().run()` with `CliCommandBuilder.with_config()` |
| `descriptor_tests.rs`             | None (no CLI calls)                                                         | N/A - only uses descriptors                                    |
| `rafaeltab_descriptor_tests.rs`   | `CliTestRunner.run()`                                                       | `testers().cmd().run()`                                        |
| `tmux_start_tests.rs`             | `CliTestRunner.run()`                                                       | `testers().cmd().run()`                                        |
| `tui_command_palette_tests.rs`    | `CliTestRunner.with_tui().run()`, `tui_test::Key`                           | `testers().pty().run()`, `test_descriptors::testers::Key`      |
| `tui_tmux_switch_tests.rs`        | `CliTestRunner.with_tui().run()`, `tui_test::Key`, `tui_test::ColorMatcher` | `testers().pty().run()`, new imports                           |
| `workspace_list_tests.rs`         | `CliTestRunner.run()`                                                       | `testers().cmd().run()`                                        |
| `worktree_start_windows_tests.rs` | `CliTestRunner.run()`                                                       | `testers().cmd().run()`                                        |

## Migration Pattern

### Old Pattern (Command Execution)

```rust
use common::CliTestRunner;

let (stdout, stderr, success) = CliTestRunner::new()
    .with_env(&env)
    .with_cwd(&some_path)
    .run(&["workspace", "list"]);

assert!(success, "Failed: {}", stderr);
```

### New Pattern (Command Execution)

```rust
use common::CliCommandBuilder;

let cmd = CliCommandBuilder::new()
    .with_env(&env)
    .with_cwd(&some_path)
    .args(&["workspace", "list"])
    .build();

let result = env.testers().cmd().run(&cmd);

assert!(result.success, "Failed: {}", result.stderr);
```

### Old Pattern (TUI Execution)

```rust
use common::CliTestRunner;
use tui_test::Key;

let mut tui = CliTestRunner::new()
    .with_env(&env)
    .with_tui()
    .run(&["tmux", "switch"]);

tui.wait_for_settle();
tui.find_text("Hello").assert_visible();
tui.press_key(Key::Enter);
```

### New Pattern (TUI Execution)

```rust
use common::CliCommandBuilder;
use test_descriptors::testers::Key;

let cmd = CliCommandBuilder::new()
    .with_env(&env)
    .args(&["tmux", "switch"])
    .build();

let mut asserter = env.testers().pty().run(&cmd);

asserter.wait_for_settle();
asserter.find_text("Hello").assert_visible();
asserter.press_key(Key::Enter);
```

## Migration Instructions by File

### 1. cli_integration_tests.rs

**Changes Required:**

- Replace `use common::CliTestRunner;` with `use common::CliCommandBuilder;`
- Replace each `CliTestRunner::new().with_env(&env).run(&[...])` call

**Tests to Migrate (5):**

1. `test_workspace_list_command`
2. `test_workspace_with_git_repo`
3. `test_tmux_integration_with_workspace`
4. `test_workspace_with_worktree_config`
5. `test_complex_workspace_scenario`

**Example Migration:**

```rust
// Before:
let (stdout, stderr, success) = CliTestRunner::new()
    .with_env(&env)
    .run(&["workspace", "list"]);

// After:
let cmd = CliCommandBuilder::new()
    .with_env(&env)
    .args(&["workspace", "list"])
    .build();
let result = env.testers().cmd().run(&cmd);
let (stdout, stderr, success) = (result.stdout, result.stderr, result.success);
```

### 2. config_flag_tests.rs

**Changes Required:**

- Replace `use common::CliTestRunner;` with `use common::CliCommandBuilder;`
- Replace `CliTestRunner::new().with_config(&path).run()` calls

**Tests to Migrate (3):**

1. `test_config_flag_uses_specified_file`
2. `test_config_flag_isolates_from_home_config`
3. `test_multiple_configs_no_crosstalk`

**Special Case:** These tests use `.with_config()` directly without `.with_env()`:

```rust
// Before:
let (stdout, _stderr, _success) = CliTestRunner::new()
    .with_config(&config_path)
    .run(&["workspace", "list"]);

// After:
let cmd = CliCommandBuilder::new()
    .with_config(&config_path)
    .args(&["workspace", "list"])
    .build();
let result = env.testers().cmd().run(&cmd);
```

### 3. descriptor_tests.rs

**Changes Required:**

- None - this file doesn't use CliTestRunner

**Verification:** Ensure it still compiles and passes.

### 4. rafaeltab_descriptor_tests.rs

**Changes Required:**

- Replace `use ... CliTestRunner;` with `use common::CliCommandBuilder;`

**Tests to Migrate (1):**

1. `test_cli_integration`

### 5. tmux_start_tests.rs

**Changes Required:**

- Replace `use ... CliTestRunner;` with `use common::CliCommandBuilder;`

**Tests to Migrate (5):**

1. `test_start_creates_sessions_from_workspace_config`
2. `test_start_is_idempotent`
3. `test_start_with_empty_config`
4. `test_start_creates_path_based_session`
5. `test_start_creates_multiple_sessions`

### 6. tui_command_palette_tests.rs

**Changes Required:**

- Replace `use crate::common::CliTestRunner;` with `use common::CliCommandBuilder;`
- Replace `use tui_test::Key;` with `use test_descriptors::testers::Key;`
- Replace all `.with_tui().run()` patterns with `.testers().pty().run()`

**Tests to Migrate (5):**

1. `test_command_palette_displays_commands`
2. `test_command_palette_filters_commands`
3. `test_command_palette_text_input`
4. `test_command_palette_enter_completes`
5. `test_command_palette_ctrl_c_exits`

**Example Migration:**

```rust
// Before:
use crate::common::CliTestRunner;
use tui_test::Key;

let mut tui = CliTestRunner::new()
    .with_env(&env)
    .with_tui()
    .run(&["command-palette", "show"]);

tui.wait_for_settle();
tui.find_text("Enter your command:").assert_visible();
tui.send_keys(&[Key::Ctrl, Key::Char('c')]);
let exit_code = tui.expect_completion();

// After:
use common::CliCommandBuilder;
use test_descriptors::testers::Key;

let cmd = CliCommandBuilder::new()
    .with_env(&env)
    .args(&["command-palette", "show"])
    .build();

let mut asserter = env.testers().pty().run(&cmd);

asserter.wait_for_settle();
asserter.find_text("Enter your command:").assert_visible();
asserter.send_keys(&[Key::Ctrl, Key::Char('c')]);
let exit_code = asserter.expect_completion();
```

### 7. tui_tmux_switch_tests.rs

**Changes Required:**

- Replace `use ... CliTestRunner;` with `use common::CliCommandBuilder;`
- Replace `use tui_test::Key;` with `use test_descriptors::testers::Key;`
- Replace `tui_test::ColorMatcher` with `test_descriptors::testers::ColorMatcher`
- Replace all TUI patterns

**Tests to Migrate (5):**

1. `test_tmux_switch_displays_sessions`
2. `test_tmux_switch_fuzzy_filtering`
3. `test_tmux_switch_navigation`
4. `test_tmux_switch_cancel_with_q`
5. `test_tmux_switch_cancel_with_ctrl_c`

**Special Case:** Uses ColorMatcher for assertions:

```rust
// Before:
first_match.fg.assert(tui_test::ColorMatcher::YellowIsh);

// After:
use test_descriptors::testers::ColorMatcher;
first_match.fg.assert(ColorMatcher::YellowIsh);
```

### 8. workspace_list_tests.rs

**Changes Required:**

- Review file contents - if it uses CliTestRunner, migrate accordingly
- If file doesn't exist or is empty, skip

### 9. worktree_start_windows_tests.rs

**Changes Required:**

- Replace `use ... CliTestRunner;` with `use common::CliCommandBuilder;`

**Tests to Migrate (3):**

1. `test_worktree_start_uses_default_windows`
2. `test_worktree_start_uses_workspace_specific_windows`
3. `test_worktree_start_handles_empty_default_windows`

## Updated common/mod.rs

After migration, the file should look like:

```rust
pub mod rafaeltab_descriptors;
pub mod cli_command_builder;

pub use cli_command_builder::CliCommandBuilder;
```

## Import Changes Summary

### Remove These Imports

```rust
// From all test files:
use common::CliTestRunner;
use tui_test::Key;
use tui_test::ColorMatcher;
```

### Add These Imports

```rust
// For command execution:
use common::CliCommandBuilder;

// For TUI testing:
use test_descriptors::testers::Key;
use test_descriptors::testers::ColorMatcher;  // Only where used
```

## Verification Checklist

After migrating each file:

1. [ ] `cargo build -p rafaeltab --tests` compiles without errors
2. [ ] `cargo test -p rafaeltab --test <test_file>` runs successfully
3. [ ] All test assertions still work correctly
4. [ ] No `tui_test` or `CliTestRunner` references remain

## Final Verification

After all migrations:

```bash
# Verify all tests compile
cargo build -p rafaeltab --tests

# Run all tests
cargo test -p rafaeltab

# Verify no old references remain
grep -r "CliTestRunner" apps/cli/tests/ && echo "ERROR: Found old CliTestRunner references" || echo "OK: No old references"
grep -r "tui_test::" apps/cli/tests/ && echo "ERROR: Found old tui_test references" || echo "OK: No old references"
grep -r "TuiCliTestRunner" apps/cli/tests/ && echo "ERROR: Found old TuiCliTestRunner references" || echo "OK: No old references"
```

## Common Migration Errors

### Error: `env.testers()` not found

**Cause:** TestEnvironment doesn't have `testers()` method imported.

**Fix:** Ensure test-descriptors re-exports the testers module properly.

### Error: Key not found

**Cause:** Using `tui_test::Key` instead of new import.

**Fix:** Change to `use test_descriptors::testers::Key;`

### Error: ColorMatcher not found

**Cause:** Using `tui_test::ColorMatcher` instead of new import.

**Fix:** Change to `use test_descriptors::testers::ColorMatcher;`

### Error: Method `run` returns different type

**Cause:** Old `run()` returned tuple, new returns `CommandResult`.

**Fix:** Destructure properly:

```rust
let result = env.testers().cmd().run(&cmd);
let (stdout, stderr, success) = (result.stdout, result.stderr, result.success);
```

Or use directly:

```rust
assert!(result.success, "Failed: {}", result.stderr);
```

## Test Count

| Category                       | Count   |
| ------------------------------ | ------- |
| Command execution tests        | ~20     |
| TUI tests                      | ~10     |
| Descriptor-only tests (no CLI) | ~10     |
| **Total tests to migrate**     | **~30** |

## Deliverables

1. All test files migrated to new API
2. All tests compile and pass
3. No references to old `CliTestRunner`, `TuiCliTestRunner`, or `tui_test`
4. Updated `common/mod.rs` with only new exports
5. Verification that all tests still test the same behavior
