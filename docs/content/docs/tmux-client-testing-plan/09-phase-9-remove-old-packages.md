---
title: Phase 9 Remove Old Packages and Helpers
---

# Phase 9: Remove Old Packages and Helpers

**Goal:** Remove the deprecated `tui-test` package and old helper functions from the CLI tests, creating compile errors that will be fixed in Phase 10.

**Prerequisite:** Phases 3-8 complete (all testers and CliCommandBuilder implemented and working)

## Overview

This phase is intentionally destructive. We remove:

1. The entire `packages/tui-test/` directory
2. `CliTestRunner` and `TuiCliTestRunner` from `apps/cli/tests/common/mod.rs`
3. Any imports or dependencies on the old code

After this phase, the CLI tests will fail to compile. Phase 10 will fix them by migrating to the new infrastructure.

## Steps

### 1. Remove tui-test Package

Delete the entire package directory:

```
packages/tui-test/
```

This includes:

- `src/lib.rs`
- `src/session.rs`
- `src/pty_manager.rs`
- `src/terminal.rs`
- `src/color.rs`
- `src/text_match.rs`
- `src/keys.rs`
- `tests/` directory with all test files
- `Cargo.toml`
- `build.rs`

### 2. Remove tui-test from Workspace

**File:** Root `Cargo.toml` (workspace)

Remove `packages/tui-test` from the workspace members list.

### 3. Remove tui-test Dependency from CLI

**File:** `apps/cli/Cargo.toml`

Remove the `tui-test` dependency from `[dev-dependencies]`:

```diff
[dev-dependencies]
test-descriptors = { path = "../../packages/test-descriptors" }
-tui_test = { path = "../../packages/tui-test" }
tempfile = "3"
```

### 4. Remove Old CliTestRunner

**File:** `apps/cli/tests/common/mod.rs`

Remove the following:

- `use tui_test::{spawn_tui, TuiSession};` import
- `CliTestRunner` struct and all its impl blocks
- `TuiCliTestRunner` struct and all its impl blocks

Keep only:

- `pub mod rafaeltab_descriptors;` (the config builder extensions)
- The new `pub mod cli_command_builder;` and `pub use cli_command_builder::CliCommandBuilder;`

The file should look like:

```rust
pub mod rafaeltab_descriptors;
pub mod cli_command_builder;

pub use cli_command_builder::CliCommandBuilder;
```

### 5. Update rafaeltab_descriptors Module

**File:** `apps/cli/tests/common/rafaeltab_descriptors/mod.rs`

Verify this module doesn't depend on CliTestRunner. It should only contain:

- `pub mod config;`
- `pub mod workspace;`
- Re-exports of traits

### 6. Verify Compile Errors

After these changes, running `cargo build --tests` in the `apps/cli` directory should produce compile errors in all test files that:

- Import `CliTestRunner` or `TuiCliTestRunner`
- Use `.with_tui()` method
- Import from `tui_test`

Expected error count: All test files except those already migrated (if any).

## Files to Delete

```
packages/tui-test/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs
│   ├── session.rs
│   ├── pty_manager.rs
│   ├── terminal.rs
│   ├── color.rs
│   ├── text_match.rs
│   └── keys.rs
└── tests/
    ├── integration_tests.rs
    ├── lifecycle_tests.rs
    ├── config_tests.rs
    ├── input_tests.rs
    ├── screen_tests.rs
    ├── color_tests.rs
    ├── terminal_sequences_tests.rs
    ├── wait_settle_tests.rs
    ├── text_finding_tests.rs
    ├── assertions_tests.rs
    └── test_programs/
        ├── mod.rs
        ├── echo_program.rs
        ├── menu_program.rs
        ├── colored_program.rs
        ├── multiscreen_program.rs
        ├── clear_screen_test.rs
        ├── cursor_position_test.rs
        ├── cursor_movement_test.rs
        ├── cursor_save_restore_test.rs
        ├── erase_line_test.rs
        ├── scrolling_region_test.rs
        ├── insert_chars_test.rs
        ├── insert_lines_test.rs
        ├── overwrite_test.rs
        ├── tab_stops_test.rs
        ├── reverse_video_test.rs
        ├── color_256_test.rs
        └── rgb_color_test.rs
```

## Files to Modify

### apps/cli/Cargo.toml

```diff
[dev-dependencies]
test-descriptors = { path = "../../packages/test-descriptors" }
-tui_test = { path = "../../packages/tui-test" }
tempfile = "3"
```

### apps/cli/tests/common/mod.rs

Remove all old code, keep only:

```rust
pub mod rafaeltab_descriptors;
pub mod cli_command_builder;

pub use cli_command_builder::CliCommandBuilder;
```

### Workspace Cargo.toml

Remove from members:

```diff
[workspace]
members = [
    "apps/cli",
    "packages/test-descriptors",
-   "packages/tui-test",
]
```

## Verification

After completing this phase:

1. `cargo build -p test-descriptors` should succeed (new infrastructure works)
2. `cargo test -p test-descriptors` should pass (new tester tests pass)
3. `cargo build -p rafaeltab --tests` should FAIL with compile errors
4. The errors should all be related to missing `CliTestRunner`, `TuiCliTestRunner`, or `tui_test` imports

## Expected Compile Errors

The following test files should have errors:

| File                              | Expected Error                                  |
| --------------------------------- | ----------------------------------------------- |
| `cli_integration_tests.rs`        | Cannot find `CliTestRunner`                     |
| `config_flag_tests.rs`            | Cannot find `CliTestRunner`                     |
| `descriptor_tests.rs`             | Cannot find `CliTestRunner`                     |
| `rafaeltab_descriptor_tests.rs`   | Cannot find `CliTestRunner`                     |
| `tmux_start_tests.rs`             | Cannot find `CliTestRunner`                     |
| `tui_command_palette_tests.rs`    | Cannot find `CliTestRunner`, `TuiCliTestRunner` |
| `tui_tmux_switch_tests.rs`        | Cannot find `CliTestRunner`, `TuiCliTestRunner` |
| `workspace_list_tests.rs`         | Cannot find `CliTestRunner`                     |
| `worktree_start_windows_tests.rs` | Cannot find `CliTestRunner`                     |

## Rollback Plan

If issues are discovered:

1. Revert the git changes with `git checkout -- .`
2. Or restore from a branch/commit before this phase

Keep a backup branch before starting this phase:

```bash
git checkout -b backup/before-phase-9
git checkout -
```

## Deliverables

1. `packages/tui-test/` directory deleted
2. `tui-test` removed from workspace Cargo.toml
3. `tui_test` dependency removed from CLI Cargo.toml
4. `CliTestRunner` and `TuiCliTestRunner` removed from common/mod.rs
5. Documented list of compile errors for Phase 10
