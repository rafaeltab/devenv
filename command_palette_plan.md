# Command Palette Implementation Plan (TDD Approach)

## Overview

This plan follows strict Test-Driven Development:

1. Write tests first
2. Implement to make tests pass
3. Refactor after tests pass

**Critical Correction**: Test commands are NOT CLI subcommands. They are command-palette commands accessed by:

1. Running `rafaeltab command-palette show`
2. Typing the test command name (e.g., "test picker")
3. Pressing Enter to select it

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

**Goal**: Create test harness commands that appear in the command palette when TEST_MODE is enabled

**Note**: These are **command-palette commands**, not CLI subcommands. Tests access them by:

1. Opening the palette: `rafaeltab command-palette show`
2. Typing: `test picker`
3. Pressing Enter

### 2.1: Write Tests for Test Commands

Create `apps/cli/tests/picker_test_commands_tests.rs`:

This file contains tests that verify our test commands work correctly. These tests will fail initially.

**Example Test Pattern**:

```rust
// SP-001: Test that test picker displays items
#[test]
fn test_picker_displays_items() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    }).create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .env("TEST_MODE", "1")
        .env("TEST_PICKER_ITEMS", "Item1,Item2,Item3")
        .args(&["command-palette", "show"])  // Open palette, not CLI subcommand
        .build();

    let mut asserter = env.testers().pty()
        .terminal_size(40, 120)
        .run(&cmd);

    asserter.wait_for_settle();

    // Find and select "test picker" command from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Now we're in the test picker - verify items displayed
    asserter.find_text("Item1").assert_visible();
    asserter.find_text("Item2").assert_visible();
    asserter.find_text("Item3").assert_visible();
}
```

**Status**: ⏳ Write all picker tests (SP-001 to CP-005) using this pattern - they'll fail

### 2.2: Write Tests for Add Workspace Flow

Create `apps/cli/tests/add_workspace_command_tests.rs`:

All AW-001 to AW-012 tests, using the same palette-first approach:

1. Run `rafaeltab command-palette show`
2. Type "add workspace"
3. Press Enter
4. Continue with the multi-step flow

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

In `commands/command_ctx.rs`:

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

impl CommandRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, command: impl Command) -> &mut Self;
    pub fn commands(&self) -> &[Box<dyn Command>];
    pub fn find_by_name(&self, name: &str) -> Option<&dyn Command>;
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

**Goal**: Enable TEST_MODE to register test commands in the palette

**Note**: Test commands are **palette commands**, not CLI subcommands

### 7.1: Test Mode Detection

In command registry initialization:

```rust
let mut registry = CommandRegistry::new();

// Register normal commands
registry.register(OpenWorkspaceCommand);
registry.register(AddWorkspaceCommand);
registry.register(TmuxSwitchCommand);

// Register test commands only in TEST_MODE
if env::var("TEST_MODE").is_ok() {
    registry.register(TestPickerCommand);
    registry.register(TestTextInputCommand);
    registry.register(TestTextInputSuggestionsCommand);
    registry.register(TestConfirmCommand);
}
```

### 7.2: Test Commands Implementation

Test commands implement the `Command` trait and appear in the palette when TEST_MODE is enabled.

#### TestPickerCommand

```rust
pub struct TestPickerCommand;

impl Command for TestPickerCommand {
    fn name(&self) -> &str { "test picker" }
    fn description(&self) -> &str { "Test the select picker" }

    fn run(&self, ctx: &mut CommandCtx) {
        let items = env::var("TEST_PICKER_ITEMS")
            .unwrap_or_default()
            .split(',')
            .map(|s| SimpleItem::new(s.to_string()))
            .collect::<Vec<_>>();

        let selection = ctx.select(&items, "Select an item:");

        // Output result for test verification
        match selection {
            Some(item) => println!("Some({:?})", item.search_text()),
            None => println!("None"),
        }
    }
}
```

#### TestTextInputCommand

```rust
pub struct TestTextInputCommand;

impl Command for TestTextInputCommand {
    fn name(&self) -> &str { "test text input" }
    fn description(&self) -> &str { "Test the text input picker" }

    fn run(&self, ctx: &mut CommandCtx) {
        let prompt = env::var("TEST_TEXT_PROMPT").unwrap_or_else(|_| "Input:".to_string());
        let input = ctx.input(&prompt);

        match input {
            Some(text) => println!("Some({:?})", text),
            None => println!("None"),
        }
    }
}
```

#### TestTextInputSuggestionsCommand

```rust
pub struct TestTextInputSuggestionsCommand;

impl Command for TestTextInputSuggestionsCommand {
    fn name(&self) -> &str { "test text input suggestions" }
    fn description(&self) -> &str { "Test text input with suggestions" }

    fn run(&self, ctx: &mut CommandCtx) {
        // Get suggestions from env or use defaults
        let suggestions = env::var("TEST_SUGGESTIONS")
            .map(|s| s.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_else(|| vec!["apple".to_string(), "application".to_string(), "apply".to_string()]);

        let provider = StaticSuggestionProvider::new(suggestions);
        let input = ctx.input_with_suggestions("Query:", &provider);

        match input {
            Some(text) => println!("Some({:?})", text),
            None => println!("None"),
        }
    }
}
```

#### TestConfirmCommand

```rust
pub struct TestConfirmCommand;

impl Command for TestConfirmCommand {
    fn name(&self) -> &str { "test confirm" }
    fn description(&self) -> &str { "Test the confirm picker" }

    fn run(&self, ctx: &mut CommandCtx) {
        let prompt = env::var("TEST_CONFIRM_PROMPT").unwrap_or_else(|_| "Confirm?".to_string());
        let default = env::var("TEST_CONFIRM_DEFAULT")
            .map(|v| v == "true")
            .unwrap_or(true);

        let confirmed = ctx.confirm(&prompt, default);

        match confirmed {
            true => println!("Some(true)"),
            false => println!("Some(false)"),
        }
    }
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

**Design Note**: Tag suggestions are sourced from existing workspaces, not from a global tags list. The `ExistingTagsSuggestionProvider` queries the workspace repository to collect all unique tags across workspaces.

### 8.1: Implement AddWorkspace Command

In `commands/builtin/add_workspace.rs`:

```rust
pub struct AddWorkspaceCommand;

impl Command for AddWorkspaceCommand {
    fn name(&self) -> &str { "add workspace" }
    fn description(&self) -> &str { "Create a workspace in the current directory" }

    fn run(&self, ctx: &mut CommandCtx) {
        // 1. Input workspace name (no suggestions)
        let name = ctx.input("Workspace name:");
        if name.is_none() { return; }
        let name = name.unwrap();

        // 2. Input tags (with suggestions from existing workspace tags)
        let tags_provider = ExistingTagsSuggestionProvider::new(ctx.workspace_repo.clone());
        let tags_input = ctx.input_with_suggestions("Tags (comma-separated):", &tags_provider);
        if tags_input.is_none() { return; }

        // Parse tags
        let tags: Vec<String> = tags_input
            .unwrap()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 3. Confirm creation
        let confirmed = ctx.confirm(&format!("Create workspace '{}' with tags {:?}?", name, tags), true);

        // 4. Add to storage if confirmed
        if confirmed {
            // Persist workspace
        }
    }
}
```

### 8.2: ExistingTagsSuggestionProvider

This provider sources tags from existing workspaces, merging and deduplicating them.

```rust
pub struct ExistingTagsSuggestionProvider {
    workspace_repo: Arc<dyn WorkspaceRepository>,
}

impl ExistingTagsSuggestionProvider {
    pub fn new(workspace_repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { workspace_repo }
    }

    /// Collect all unique tags from existing workspaces
    fn collect_all_tags(&self) -> Vec<String> {
        let workspaces = self.workspace_repo.list_all();

        // Collect tags from all workspaces, then deduplicate
        let mut all_tags: Vec<String> = workspaces
            .iter()
            .flat_map(|w| w.tags.clone())
            .collect();

        // Sort and deduplicate (case-sensitive dedup, but could be case-insensitive if needed)
        all_tags.sort();
        all_tags.dedup();

        all_tags
    }
}

impl SuggestionProvider for ExistingTagsSuggestionProvider {
    fn suggestions(&self, input: &str) -> Option<Vec<String>> {
        if input.is_empty() {
            return None;
        }

        let all_tags = self.collect_all_tags();

        // Return fuzzy matches from all workspace tags
        let matches: Vec<String> = all_tags
            .iter()
            .filter(|tag| tag.to_lowercase().contains(&input.to_lowercase()))
            .cloned()
            .collect();

        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }
}
```

**Key Points:**

- Tags are collected from all existing workspaces via `workspace_repo.list_all()`
- Tags are merged and deduplicated across workspaces
- If a tag appears in multiple workspaces, it only appears once in suggestions
- When no workspaces exist, no suggestions are provided (empty input will show "No suggestions available")

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
    // Create command registry
    let mut registry = CommandRegistry::new();

    // Register normal commands
    registry.register(OpenWorkspaceCommand);
    registry.register(AddWorkspaceCommand);
    registry.register(TmuxSwitchCommand);
    registry.register(CommandPalette);  // The palette itself is a command

    // Register test commands in TEST_MODE
    if env::var("TEST_MODE").is_ok() {
        registry.register(TestPickerCommand);
        registry.register(TestTextInputCommand);
        registry.register(TestTextInputSuggestionsCommand);
        registry.register(TestConfirmCommand);
    }

    // Handle CLI args
    // If "command-palette show" args given, run the palette
    // Otherwise run normal command
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
# Open command palette (normal mode)
cargo run -- command-palette show

# Open command palette with test commands available
TEST_MODE=1 cargo run -- command-palette show
# Then type "test picker" and select it
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

| Phase | Activity               | Tests                                          |
| ----- | ---------------------- | ---------------------------------------------- |
| 0     | Cleanup                | Delete old tests/code                          |
| 1     | Testing Infra          | Write infra tests → implement → pass           |
| 2     | Write All Tests        | Write picker tests using palette-first pattern |
| 3     | Core Framework         | Implement → picker structure tests pass        |
| 4     | Picker Implementations | Implement → all picker tests pass              |
| 5     | Command Abstraction    | Implement → structure ready                    |
| 6     | Command Palette        | Implement → CPAL tests pass                    |
| 7     | Test Mode              | Implement → Phase 2 tests pass                 |
| 8     | Add Workspace          | Implement → AW tests pass                      |
| 9     | Integration            | Full test suite passes                         |
| 10    | Refactoring            | Tests still pass                               |

**Total Test Files Created**:

1. `packages/test-descriptors/src/testers/infra_tests.rs` - Testing infrastructure tests
2. `apps/cli/tests/picker_test_commands_tests.rs` - All picker TUI tests (SP-_, TP-_, TPS-_, CP-_)
3. `apps/cli/tests/add_workspace_command_tests.rs` - All add workspace tests (AW-\*)
4. Unit tests embedded in source files (TPS-006, TPS-007, CPAL-004)

**Key TDD Principle**: At every phase, tests are written BEFORE implementation, and we don't move to the next phase until all current tests pass.

**Critical Correction**: Test commands are palette commands accessed via `rafaeltab command-palette show` → type "test picker" → Enter. They are NOT CLI subcommands like `rafaeltab test picker`.
