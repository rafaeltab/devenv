# Command Palette Implementation Plan (TDD Approach)

## Overview

This plan follows strict Test-Driven Development:

1. Write tests first
2. Implement to make tests pass
3. Refactor after tests pass

## Phase 0: Cleanup

**Goal**: Remove old code to start fresh

### 0.1: Remove Existing Tests

- Delete `apps/cli/tests/tui_command_palette_tests.rs`
- Delete `apps/cli/tests/command_palette_additional_tests.rs`
- Remove any command palette test helpers from `apps/cli/tests/common/`

### 0.2: Remove Existing Command Palette Code

- Delete `apps/cli/src/command_palette/` directory entirely
- Remove command palette references from `apps/cli/src/main.rs`
- Remove command palette CLI args if any

### 0.3: Verify Clean Slate

- Run `cargo build` - should compile without command palette
- Run `cargo test` - should pass (palette tests removed)

---

## Phase 1: Testing Infrastructure (TDD)

**Goal**: Create the testing utilities needed for screen-based TDD

**Note**: This phase is itself TDD - write test-infra tests first, then implement

### 1.1: Testing Infrastructure Tests

Create `packages/test-descriptors/src/testers/infra_tests.rs`:

```rust
// These test that our testing infrastructure works

#[test]
fn test_color_assertion_grayscale() {
    let color = ColorAssertion::from_rgb(128, 128, 128);
    color.assert_grayscale(); // Should pass
}

#[test]
fn test_color_assertion_not_grayscale() {
    let color = ColorAssertion::from_rgb(255, 255, 0); // Yellow
    color.assert_not_grayscale(); // Should pass
}

#[test]
fn test_vertical_order_assertion() {
    // Test that assert_vertical_order works correctly
    // This will need a mock TuiAsserter
}

#[test]
fn test_output_capture() {
    // Test that expect_completion_and_get_output works
}
```

**Status**: ⏳ Write these tests first - they'll fail

### 1.2: Implement Color Assertions

Add to `packages/test-descriptors/src/testers/color.rs`:

- `assert_grayscale()`
- `assert_not_grayscale()`

**Status**: ⏳ Implement - tests should pass

### 1.3: Implement TUI Asserter Extensions

Add to `packages/test-descriptors/src/testers/tui_asserter.rs`:

- `assert_vertical_order(&[TextMatch])`
- `expect_completion_and_get_output() -> String`

**Status**: ⏳ Implement - tests should pass

### 1.4: Verify Testing Infrastructure

- Run `cargo test -p test-descriptors`
- All new test-infra tests pass

---

## Phase 2: Test Commands & TEST_MODE (TDD)

**Goal**: Create test harness in CLI for picker testing

**Note**: Tests written here will fail until Phase 7 when we implement test mode

### 2.1: Write Tests for Test Commands

Create `apps/cli/tests/picker_test_commands_tests.rs`:

This file contains tests that verify our test commands work correctly. These tests will fail initially.

```rust
// SP-001: Test that test picker displays items
#[test]
fn test_picker_displays_items() {
    // This test uses the test command to verify picker infrastructure
    // Will fail until test commands are implemented
}

// SP-002: Test that fuzzy search filtering works
#[test]
fn test_picker_fuzzy_filter() {
    // Will fail until picker is implemented
}

// ... all picker tests from command_palette_tests.md
// Each test uses TEST_MODE=1 and test commands
```

**Status**: ⏳ Write all picker tests (SP-001 to CP-005) - they'll fail

### 2.2: Write Tests for Add Workspace Flow

Create `apps/cli/tests/add_workspace_command_tests.rs`:

All AW-001 to AW-012 tests, using TEST_MODE to mock inputs.

**Status**: ⏳ Write all add workspace tests - they'll fail

---

## Phase 3: Core TUI Framework (TDD)

**Goal**: Build ratatui components that tests expect

### 3.1: Create Module Structure

```
apps/cli/src/tui/
  mod.rs
  frame.rs
  picker_item.rs
  picker_ctx.rs
  pickers/
    mod.rs
    select_picker.rs
    text_picker.rs
    text_picker_with_suggestions.rs
    confirm_picker.rs
```

### 3.2: Implement PickerItem Trait

In `tui/picker_item.rs`:

```rust
pub trait PickerItem: WidgetRef {
    fn constraint(&self) -> Constraint;
    fn search_text(&self) -> &str;
}
```

**Status**: ⏳ Implement - picker tests start passing

### 3.3: Implement Frame Component

In `tui/frame.rs` - Three-panel layout component

**Status**: ⏳ Implement - UI structure tests pass

### 3.4: Implement PickerCtx

In `tui/picker_ctx.rs` - Context for running pickers

**Status**: ⏳ Implement - picker integration works

---

## Phase 4: Picker Implementations (TDD)

**Goal**: Implement each picker to satisfy tests

### 4.1: Select Picker

In `tui/pickers/select_picker.rs`:

- Fuzzy search with sublime_fuzzy
- Navigation (arrows, Ctrl+J/K)
- Selection (Enter)
- Cancel (Escape, Ctrl+C)
- Highlighting

**Tests Passing**: SP-001 to SP-008, SP-012, SP-013

### 4.2: Text Picker

In `tui/pickers/text_picker.rs`:

- Basic text input
- Backspace handling
- Unicode support
- Confirm/Cancel

**Tests Passing**: TP-001 to TP-007

### 4.3: Text Picker with Suggestions

In `tui/pickers/text_picker_with_suggestions.rs`:

- Display suggestions
- Tab completion
- Arrow navigation for suggestions
- Ctrl+J/K navigation

**Tests Passing**: TPS-001 to TPS-005, TPS-008, TPS-009

### 4.4: Confirm Picker

In `tui/pickers/confirm_picker.rs`:

- Yes/No display
- Default highlighting
- Arrow selection
- Confirm/Cancel

**Tests Passing**: CP-001 to CP-005

### 4.5: Unit Tests for Suggestion Logic

Create unit tests for:

- TPS-006: No suggestions handling
- TPS-007: Empty suggestions handling

**Status**: ⏳ Write unit tests - implement - tests pass

---

## Phase 5: Command Abstraction (TDD)

**Goal**: Define how commands work

### 5.1: Command Trait

In `commands/command.rs`:

```rust
pub trait Command: PickerItem {
    fn run(&self, ctx: &mut CommandCtx);
}
```

### 5.2: CommandCtx

In `commands/command.rs`:

```rust
pub struct CommandCtx {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl CommandCtx {
    pub fn select<T: PickerItem>(&mut self, items: &[T], prompt: &str) -> Option<&T>;
    pub fn input(&mut self, prompt: &str) -> Option<String>;
    pub fn input_with_suggestions(&mut self, prompt: &str, provider: &dyn SuggestionProvider) -> Option<String>;
    pub fn confirm(&mut self, prompt: &str, default: bool) -> bool;
    pub fn execute(&self, command: &str) -> io::Result<()>;
}
```

### 5.3: CommandRegistry

In `commands/registry.rs`:

```rust
pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
}
```

**Status**: ⏳ Implement - command palette structure ready

---

## Phase 6: Command Palette (TDD)

**Goal**: The palette that lists and runs commands

### 6.1: Implement CommandPalette Command

In `commands/command_palette.rs`:

```rust
pub struct CommandPalette {
    registry: CommandRegistry,
}

impl Command for CommandPalette {
    fn run(&self, ctx: &mut CommandCtx) {
        // Show picker with all commands
        // Run selected command
    }
}
```

**Tests Passing**: CPAL-001 to CPAL-003, CPAL-005, CPAL-006

### 6.2: Unit Test for Command Execution

Unit test for:

- CPAL-004: Command selection runs command

**Status**: ⏳ Write test - implement - test passes

---

## Phase 7: Test Mode & Test Commands (TDD)

**Goal**: Enable TEST_MODE for running tests

### 7.1: Test Mode Detection

In CLI argument parsing or early init:

```rust
if env::var("TEST_MODE").is_ok() {
    // Enable test commands
}
```

### 7.2: Test Commands Implementation

#### test picker

```rust
fn test_picker_command() {
    let items = env::var("TEST_PICKER_ITEMS")
        .unwrap_or_default()
        .split(',')
        .collect::<Vec<_>>();

    // Show select picker with these items
    // Output: Some("<selection>") or None
}
```

#### test text input

```rust
fn test_text_input_command() {
    let prompt = env::var("TEST_TEXT_PROMPT").unwrap_or("Input:".to_string());
    // Show text picker
    // Output: Some("<input>") or None
}
```

#### test text input suggestions

```rust
fn test_text_input_suggestions_command() {
    // Show text picker with suggestions
    // Output: Some("<input>") or None
}
```

#### test confirm

```rust
fn test_confirm_command() {
    let prompt = env::var("TEST_CONFIRM_PROMPT").unwrap_or("Confirm?".to_string());
    let default = env::var("TEST_CONFIRM_DEFAULT").map(|v| v == "true").unwrap_or(true);
    // Show confirm picker
    // Output: Some(true), Some(false), or None
}
```

### 7.3: Run All Picker Tests

```bash
cargo test picker_test_commands_tests
```

**Status**: All picker tests (SP-_, TP-_, TPS-_, CP-_) should pass

---

## Phase 8: Add Workspace Command (TDD)

**Goal**: Full add workspace flow

### 8.1: Implement AddWorkspace Command

In `commands/builtin/add_workspace.rs`:

```rust
pub struct AddWorkspaceCommand;

impl Command for AddWorkspaceCommand {
    fn run(&self, ctx: &mut CommandCtx) {
        // 1. Input workspace name (no suggestions)
        let name = ctx.input("Workspace name:");

        // 2. Input tags (with suggestions from existing tags)
        let tags_provider = ExistingTagsSuggestionProvider::new();
        let tags = ctx.input_with_suggestions("Tags (comma-separated):", &tags_provider);

        // 3. Confirm creation
        let confirmed = ctx.confirm(&format!("Create workspace '{}' with tags {:?}?", name, tags), true);

        // 4. Add to storage if confirmed
    }
}
```

### 8.2: ExistingTagsSuggestionProvider

```rust
pub struct ExistingTagsSuggestionProvider {
    existing_tags: Vec<String>,
}

impl SuggestionProvider for ExistingTagsSuggestionProvider {
    fn suggestions(&self, input: &str) -> Option<Vec<String>> {
        // Return fuzzy matches from existing_tags
    }
}
```

### 8.3: Run All Add Workspace Tests

```bash
cargo test add_workspace_command_tests
```

**Status**: All AW-\* tests pass

---

## Phase 9: Integration & Verification

**Goal**: Everything works together

### 9.1: Wire Up to CLI

In `main.rs`:

```rust
fn main() {
    // Register commands
    let registry = CommandRegistry::new()
        .register(OpenWorkspaceCommand)
        .register(AddWorkspaceCommand)
        .register(TmuxSwitchCommand);

    // Handle CLI args
    // If TEST_MODE, enable test commands
    // Otherwise run normal command palette
}
```

### 9.2: Run Full Test Suite

```bash
cargo test
```

**Status**: All tests pass (including old tests for other features)

### 9.3: Manual Verification

Test in terminal:

```bash
TEST_MODE=1 cargo run -- test picker  # Should show picker
```

---

## Phase 10: Refactoring

**Goal**: Clean up code without breaking tests

### 10.1: Code Quality Review

- [ ] Check for duplication between pickers
- [ ] Review error handling
- [ ] Review public API surface
- [ ] Add documentation

### 10.2: Refactoring Changes

- [ ] Extract common picker logic
- [ ] Simplify frame rendering
- [ ] Optimize fuzzy search caching
- [ ] Clean up imports

### 10.3: Verify Tests Still Pass

```bash
cargo test
```

**Status**: All tests still pass after refactoring

---

## Summary

| Phase | Activity               | Tests                                   |
| ----- | ---------------------- | --------------------------------------- |
| 0     | Cleanup                | Delete old tests/code                   |
| 1     | Testing Infra          | Write infra tests → implement → pass    |
| 2     | Write All Tests        | Write picker tests (will fail)          |
| 3     | Core Framework         | Implement → picker structure tests pass |
| 4     | Picker Implementations | Implement → all picker tests pass       |
| 5     | Command Abstraction    | Implement → structure ready             |
| 6     | Command Palette        | Implement → CPAL tests pass             |
| 7     | Test Mode              | Implement → Phase 2 tests pass          |
| 8     | Add Workspace          | Implement → AW tests pass               |
| 9     | Integration            | Full test suite passes                  |
| 10    | Refactoring            | Tests still pass                        |

**Total Test Files Created**:

1. `packages/test-descriptors/src/testers/infra_tests.rs` - Testing infrastructure tests
2. `apps/cli/tests/picker_test_commands_tests.rs` - All picker TUI tests (SP-_, TP-_, TPS-_, CP-_)
3. `apps/cli/tests/add_workspace_command_tests.rs` - All add workspace tests (AW-\*)
4. Unit tests embedded in source files (TPS-006, TPS-007, CPAL-004)

**Key TDD Principle**: At every phase, tests are written BEFORE implementation, and we don't move to the next phase until all current tests pass.
