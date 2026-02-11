# Command Palette TDD Test Specification

## Overview

This document contains comprehensive test cases for the command palette system, designed for Test-Driven Development (TDD).

**Test Mode**: All TUI tests run with `TEST_MODE=1` environment variable enabled, which exposes test commands and output formats for verification.

## Test Categories

- **TUI** - Screen-based integration tests using PTY harness
- **Unit** - Code-level tests against internal logic
- **Hybrid** - Mix of both approaches

---

## Core Picker Tests

### Select Picker Tests

#### SP-001: Display Items

**Type**: TUI  
**Given** a list of items implementing `PickerItem`  
**When** the select picker is shown  
**Then** all items should be visible in the middle panel

**Test Command**: `test picker` (reads items from `TEST_PICKER_ITEMS` env var, comma-separated)  
**Verification**: `asserter.find_text("<item>").assert_visible()` for each item

#### SP-002: Fuzzy Search Filtering

**Type**: TUI  
**Given** items: ["Apple", "Banana", "Cherry"] via `TEST_PICKER_ITEMS`  
**When** user types "ap"  
**Then** only "Apple" should be visible  
**And** it should be highlighted (yellow color)

**Verification**:

```rust
asserter.find_text("Banana").assert_not_visible();
asserter.find_text("Cherry").assert_not_visible();
let apple = asserter.find_text("Apple");
apple.assert_visible();
apple.fg.assert_not_grayscale(); // highlighted
```

#### SP-003: Fuzzy Search Scoring

**Type**: TUI  
**Given** items: ["application", "Apple", "pineapple"] via `TEST_PICKER_ITEMS`  
**When** user types "app"  
**Then** items should be ordered by match quality (Apple > application > pineapple)

**Verification**:

```rust
asserter.assert_vertical_order(&[
    asserter.find_text("Apple"),
    asserter.find_text("application"),
    asserter.find_text("pineapple"),
]);
```

#### SP-004: Empty Search Shows All

**Type**: TUI  
**Given** items: ["Item1", "Item2", "Item3"] via `TEST_PICKER_ITEMS`  
**When** user has typed nothing  
**Then** all items should be visible in original order

**Verification**: Same `assert_vertical_order` pattern as SP-003

#### SP-005: No Matches Display

**Type**: TUI  
**Given** items: ["Apple", "Banana"] via `TEST_PICKER_ITEMS`  
**When** user types "xyz"  
**Then** middle panel should show "No matches"

**Verification**: `asserter.find_text("No matches").assert_visible()`

#### SP-006: Navigation - Down Arrow

**Type**: TUI  
**Given** items: ["Item1", "Item2", "Item3"] with "Item1" selected  
**When** user presses Down arrow  
**Then** "Item2" should be highlighted (yellow)

**Verification**:

```rust
// Initially Item1 highlighted
let item1 = asserter.find_text("Item1");
item1.fg.assert_not_grayscale();
let item2 = asserter.find_text("Item2");
item2.fg.assert_grayscale();

// Press down
asserter.press_key(Key::Down);
asserter.wait_for_settle();

// Now Item2 highlighted, Item1 not
item2.fg.assert_not_grayscale();
item1.fg.assert_grayscale();
```

#### SP-007: Navigation - Up Arrow Wrap

**Type**: TUI  
**Given** items: ["Item1", "Item2"] with "Item1" selected  
**When** user presses Up arrow  
**Then** "Item2" should be highlighted (wraps around)

**Verification**: Color assertions as in SP-006

#### SP-008: Navigation - Down Arrow Wrap

**Type**: TUI  
**Given** items: ["Item1", "Item2"] with "Item2" selected  
**When** user presses Down arrow  
**Then** "Item1" should be highlighted (wraps around)

**Verification**: Color assertions as in SP-006

#### SP-009: Selection - Enter Key

**Type**: TUI  
**Given** items with "Item2" highlighted  
**When** user presses Enter  
**Then** picker should output `Some("Item2")` to stdout  
**And** TUI should exit

**Verification**:

```rust
asserter.press_key(Key::Enter);
let output = asserter.expect_completion_and_get_output();
assert_eq!(output.trim(), "Some(\"Item2\")");
```

#### SP-010: Cancel - Escape Key

**Type**: TUI  
**Given** picker is open  
**When** user presses Escape  
**Then** picker should output `None` to stdout  
**And** TUI should exit

**Verification**: `assert_eq!(output.trim(), "None")`

#### SP-011: Cancel - Ctrl+C

**Type**: TUI  
**Given** picker is open  
**When** user presses Ctrl+C  
**Then** picker should output `None` to stdout  
**And** TUI should exit

**Verification**: `assert_eq!(output.trim(), "None")`

#### SP-012: Custom Item Rendering

**Type**: TUI  
**Given** items with custom `WidgetRef` implementation  
**When** picker is shown  
**Then** items should render according to their custom implementation

**Verification**: Text assertions for custom-rendered content

#### SP-013: Item Constraints

**Type**: TUI  
**Given** items with different `constraint()` values  
**When** picker is shown  
**Then** items should be laid out according to their constraints

**Verification**: Size/position assertions on rendered items

---

### Text Picker Tests

#### TP-001: Basic Text Input

**Type**: TUI  
**Test Command**: `test text input` (prompt from `TEST_TEXT_PROMPT` env var)  
**Given** empty text picker with prompt "Enter name:"  
**When** user types "Hello"  
**Then** input panel should show "Enter name: Hello"

**Verification**: `asserter.find_text("Enter name: Hello").assert_visible()`

#### TP-002: Text Input - Backspace

**Type**: TUI  
**Given** text picker with input "Hello"  
**When** user presses Backspace  
**Then** input should show "Hell"

**Verification**:

```rust
asserter.find_text("Enter name: Hello").assert_visible();
asserter.press_key(Key::Backspace);
asserter.wait_for_settle();
asserter.find_text("Enter name: Hell").assert_visible();
asserter.find_text("Enter name: Hello").assert_not_visible();
```

#### TP-003: Text Input - Empty Backspace

**Type**: TUI  
**Given** text picker with empty input  
**When** user presses Backspace  
**Then** input should remain empty

**Verification**: Prompt still visible, no change after backspace

#### TP-004: Text Input - Confirm with Enter

**Type**: TUI  
**Given** text picker with input "Test"  
**When** user presses Enter  
**Then** picker should output `Some("Test")` to stdout  
**And** TUI should exit

**Verification**: `assert_eq!(output.trim(), "Some(\"Test\")")`

#### TP-005: Text Input - Cancel with Escape

**Type**: TUI  
**Given** text picker with input "Test"  
**When** user presses Escape  
**Then** picker should output `None` to stdout  
**And** TUI should exit

**Verification**: `assert_eq!(output.trim(), "None")`

#### TP-006: Text Input - Cancel with Ctrl+C

**Type**: TUI  
**Given** text picker is open  
**When** user presses Ctrl+C  
**Then** picker should output `None` to stdout  
**And** TUI should exit

**Verification**: `assert_eq!(output.trim(), "None")`

#### TP-007: Unicode Support

**Type**: TUI  
**Given** text picker  
**When** user types "日本語テスト"  
**Then** input should show "日本語テスト" correctly

**Verification**: `asserter.find_text("日本語テスト").assert_visible()`

---

### Text Picker with Suggestions Tests

#### TPS-001: Suggestions Display

**Type**: TUI  
**Test Command**: `test text input suggestions`  
**Given** text picker with suggestions provider returning ["apple", "application", "apply"]  
**When** user types "app"  
**Then** middle panel should show all three suggestions  
**And** "apple" should be highlighted (top match)

**Verification**:

```rust
// All suggestions visible
asserter.find_text("apple").assert_visible();
asserter.find_text("application").assert_visible();
asserter.find_text("apply").assert_visible();

// First one highlighted
let apple = asserter.find_text("apple");
apple.fg.assert_not_grayscale();
```

#### TPS-002: Top Suggestion Auto-Highlighted

**Type**: TUI  
**Given** text picker with suggestions  
**When** suggestions are displayed  
**Then** the first suggestion should be visually highlighted  
**And** should be the Tab completion target

**Verification**: Color assertion on first suggestion

#### TPS-003: Tab Completes Top Suggestion

**Type**: TUI  
**Given** text picker with suggestions, top suggestion "apple" highlighted  
**When** user presses Tab  
**Then** input should be completed to "apple"

**Verification**: `asserter.find_text("Query: apple").assert_visible()`

#### TPS-004: Arrow Navigation Changes Target

**Type**: TUI  
**Given** text picker with suggestions ["apple", "banana", "cherry"], "apple" selected  
**When** user presses Down arrow (now "banana" selected)  
**And** user presses Tab  
**Then** input should be completed to "banana"

**Verification**:

```rust
// Navigate down
asserter.press_key(Key::Down);
asserter.wait_for_settle();
// Tab to complete
asserter.press_key(Key::Tab);
asserter.wait_for_settle();
// Verify banana completed
asserter.find_text("Query: banana").assert_visible();
```

#### TPS-005: Enter Confirms Current Text

**Type**: TUI  
**Given** text picker with input "cust"  
**And** suggestions ["custom", "customer"]  
**When** user presses Enter (without Tab)  
**Then** picker should output `Some("cust")` to stdout

**Verification**: `assert_eq!(output.trim(), "Some(\"cust\")")`

#### TPS-006: No Suggestions - Shows Message

**Type**: Unit  
**Given** text picker where provider returns `None` for input "x"  
**When** user types "x"  
**Then** middle panel should show "No suggestions available"

**Note**: Unit test - tests suggestion provider logic, not rendering

#### TPS-007: Empty Suggestions - Shows Message

**Type**: Unit  
**Given** text picker where provider returns `Some([])` for input "xyz"  
**When** user types "xyz"  
**Then** middle panel should show "No suggestions available"

**Note**: Unit test - tests suggestion provider logic, not rendering

#### TPS-008: Suggestions Update on Input Change

**Type**: TUI  
**Given** text picker with dynamic suggestions  
**When** user types "a" (suggestions: ["apple", "apricot"])  
**And** user types "b" (suggestions: ["banana", "blueberry"])  
**Then** middle panel should update to show new suggestions

**Verification**:

```rust
// After typing "a"
asserter.find_text("apple").assert_visible();
asserter.find_text("banana").assert_not_visible();

// Type "b" (now "ab")
asserter.type_text("b");
asserter.wait_for_settle();

// Now banana visible, apple not
asserter.find_text("banana").assert_visible();
asserter.find_text("apple").assert_not_visible();
```

#### TPS-009: Ctrl+J/Ctrl+K Navigation

**Type**: TUI  
**Given** text picker with suggestions  
**When** user presses Ctrl+J  
**Then** selection should move down (highlight changes)  
**When** user presses Ctrl+K  
**Then** selection should move up (highlight changes)

**Verification**: Color assertions to verify highlight movement

---

### Confirm Picker Tests

#### CP-001: Confirm Display - Default Yes

**Type**: TUI  
**Test Command**: `test confirm` (reads prompt from `TEST_CONFIRM_PROMPT`, default from `TEST_CONFIRM_DEFAULT`)  
**Given** confirm picker with prompt "Continue?" and default `true`  
**Then** "Yes" should be highlighted  
**And** "No" should be visible but not highlighted

**Verification**:

```rust
let yes = asserter.find_text("Yes");
let no = asserter.find_text("No");
yes.fg.assert_not_grayscale(); // highlighted
no.fg.assert_grayscale();        // not highlighted
```

#### CP-002: Confirm Display - Default No

**Type**: TUI  
**Given** confirm picker with prompt "Delete?" and default `false`  
**Then** "No" should be highlighted  
**And** "Yes" should be visible but not highlighted

**Verification**: Inverse of CP-001 color assertions

#### CP-003: Confirm - Select Yes

**Type**: TUI  
**Given** confirm picker with "No" selected  
**When** user presses Left/Right arrow to change selection  
**And** user presses Enter  
**Then** picker should output `Some(true)` to stdout

**Verification**: `assert_eq!(output.trim(), "Some(true)")`

#### CP-004: Confirm - Select No

**Type**: TUI  
**Given** confirm picker  
**When** user selects "No"  
**And** user presses Enter  
**Then** picker should output `Some(false)` to stdout

**Verification**: `assert_eq!(output.trim(), "Some(false)")`

#### CP-005: Confirm - Cancel

**Type**: TUI  
**Given** confirm picker  
**When** user presses Escape  
**Then** picker should output `None` to stdout

**Verification**: `assert_eq!(output.trim(), "None")`

---

## Command Palette Tests

### CPAL-001: Display All Commands

**Type**: TUI  
**Given** command registry with commands: [OpenWorkspace, AddWorkspace, TmuxSwitch]  
**When** command palette is shown  
**Then** all three commands should be visible  
**And** each should show its title and description

**Verification**: Text visibility assertions for titles and descriptions

### CPAL-002: Command Fuzzy Search

**Type**: TUI  
**Given** commands: ["Open Workspace", "Add Workspace", "Tmux Switch"]  
**When** user types "add"  
**Then** only "Add Workspace" should be visible

**Verification**:

```rust
asserter.find_text("Add Workspace").assert_visible();
asserter.find_text("Open Workspace").assert_not_visible();
```

### CPAL-003: Search Across Description

**Type**: TUI  
**Given** command "Git Status" with description "Show repository status"  
**When** user types "repository"  
**Then** "Git Status" should appear in results

**Verification**: `asserter.find_text("Git Status").assert_visible()`

### CPAL-004: Command Selection Runs Command

**Type**: Unit  
**Given** command palette with commands  
**When** user selects "Add Workspace"  
**Then** `AddWorkspaceCommand::run()` should be called  
**And** command palette should transfer control to the command

**Note**: Unit test - mocks command execution to verify control transfer

### CPAL-005: Cancel Returns None

**Type**: TUI  
**Given** command palette is open  
**When** user presses Escape  
**Then** command palette should exit  
**And** no command should be executed

**Verification**: Process exits with no output / command not run

### CPAL-006: Empty Registry

**Type**: TUI  
**Given** empty command registry  
**When** command palette is shown  
**Then** middle panel should show "No commands available"

**Verification**: `asserter.find_text("No commands available").assert_visible()`

---

## Add Workspace Command Tests

### AW-001: Full Flow - Happy Path

**Type**: TUI  
**Setup:**

- Workspace storage is empty
- Existing tags: ["rust", "typescript", "go"] in config

**Flow:**

1. **Command Palette Display**

   - Command palette shows with "Add Workspace" command visible
   - Description shows: "Create a workspace in the current directory"

2. **Select Add Workspace**

   - User selects "Add Workspace"
   - Control transfers to `AddWorkspaceCommand`

3. **Name Input (No Suggestions)**

   - Text picker shown with prompt "Workspace name:"
   - Middle panel is empty (no suggestions for this input)
   - User types "my-project"
   - User presses Enter

4. **Tags Input (With Suggestions)**

   - Text picker shown with prompt "Tags (comma-separated):"
   - User types "rus"
   - Middle panel shows suggestions: ["rust", ...]
   - User presses Tab to complete "rust"
   - User types ", ty"
   - Middle panel shows suggestions based on existing tags: ["typescript"]
   - User presses Tab to complete "typescript"
   - Input shows: "rust, typescript"
   - User presses Enter

5. **Confirmation Display**

   - Confirm picker shown with summary:
     ```
     Name: my-project
     ID: my-project (slugified from name)
     Tags:
       1. rust
       2. typescript
     ```
   - Prompt: "Create this workspace?"
   - Default: Yes

6. **Confirm Creation**

   - User confirms with Enter
   - Command outputs success message

7. **Command Completes**
   - TUI exits (no return to palette)

**Verification:**

- Screen text assertions at each step
- Verify config file contains workspace with correct name, id, and tags

### AW-002: Cancel at Name Input

**Type**: TUI  
**Given** at name input step  
**When** user presses Escape  
**Then** command should exit  
**And** no workspace should be added (verify config unchanged)

### AW-003: Cancel at Tags Input

**Type**: TUI  
**Given** at tags input step  
**When** user presses Escape  
**Then** command should exit  
**And** no workspace should be added

### AW-004: Cancel at Confirmation

**Type**: TUI  
**Given** at confirmation step  
**When** user selects "No"  
**Then** command should exit  
**And** no workspace should be added

### AW-005: Empty Name Validation

**Type**: TUI  
**Given** at name input  
**When** user tries to enter empty string and press Enter  
**Then** input should be rejected (or handled gracefully)  
**And** user should remain at name input

**Verification**: Still at name input prompt after pressing Enter with empty input

### AW-006: Special Characters in Name

**Type**: TUI  
**Given** at name input  
**When** user types "My Project!@#"  
**And** completes the flow  
**Then** ID in config should be "my-project" (slugified)

**Verification**: Read config file and verify ID field

### AW-007: Duplicate Tag Handling

**Type**: TUI  
**Given** user types "rust, rust, typescript" in tags input  
**When** at confirmation step  
**Then** tags in config should be deduplicated to ["rust", "typescript"]

**Verification**: Read config file and verify tags array

### AW-008: Empty Tags

**Type**: TUI  
**Given** at tags input  
**When** user presses Enter with empty input  
**Then** workspace should be created with empty tags list in config

**Verification**: Read config file, verify tags is empty array

### AW-009: Tag Suggestion Based on Existing Workspaces

**Type**: TUI  
**Setup:**

- Existing workspace with tags: ["python", "django"] in config

**Flow:**

- User types "dj" in tags input
- Suggestions should include "django" from existing tags

**Verification**: `asserter.find_text("django").assert_visible()`

### AW-010: Tag Suggestion Partial Match

**Type**: TUI  
**Setup:**

- Existing tags: ["rust", "ruby", "react"] in config

**Flow:**

- User types "ru" in tags input
- Suggestions should include "rust", "ruby"
- User types "rus"
- Suggestions should only include "rust"

**Verification**: Visibility assertions for suggestions at each step

### AW-011: Case Insensitive Tag Matching

**Type**: TUI  
**Setup:**

- Existing tags: ["Rust", "TypeScript"] in config

**Flow:**

- User types "rust" in tags input
- Suggestions should include "Rust" (case-insensitive match)

**Verification**: `asserter.find_text("Rust").assert_visible()`

### AW-012: Multi-word Tag Input

**Type**: TUI  
**Given** at tags input  
**When** user types "web framework, rust, cli"  
**Then** after Enter, tags in config should be parsed as: ["web framework", "rust", "cli"]

**Verification**: Read config file and verify tags array

---

## Integration Tests

### INT-001: Terminal Restoration

**Type**: TUI  
**Given** command palette is running  
**When** user cancels with Escape  
**Then** terminal should be fully restored  
**And** no alternate screen artifacts should remain

**Verification**:

```rust
asserter.press_key(Key::Esc);
let exit_code = asserter.expect_completion();
assert_eq!(exit_code, 0);
// Verify terminal is back to normal (no escape sequences in output)
```

### INT-002: External Command Execution

**Type**: TUI  
**Given** command that calls `ctx.execute("echo test")`  
**When** command runs  
**Then** terminal should exit TUI mode  
**And** "test" should be printed to stdout  
**And** terminal should return to shell (not TUI)

**Verification**: Output contains "test" after command execution

### INT-003: Chained Inputs

**Type**: TUI  
**Given** command with multiple picker calls  
**When** first picker completes  
**Then** second picker should show immediately  
**And** terminal should remain in TUI mode

**Verification**: Sequential screen assertions showing picker transitions

### INT-004: Unicode in Workspaces

**Type**: TUI  
**Given** workspace name with unicode characters  
**When** creating workspace  
**Then** name should be preserved correctly in config

**Verification**: Read config file and verify name field

---

## Edge Cases

### EDGE-001: Very Long Command List

**Type**: TUI  
**Given** 1000+ commands in registry  
**When** command palette shown  
**Then** fuzzy search should remain responsive  
**And** scrolling should work smoothly

**Verification**: Time-based assertions (completes within X ms)

### EDGE-002: Rapid Input

**Type**: TUI  
**Given** text picker with suggestions  
**When** user types very quickly  
**Then** suggestions should not lag significantly  
**And** no duplicate/out-of-order suggestions

**Verification**: Time-based assertions

### EDGE-003: Resize During Picker

**Type**: TUI  
**Given** picker is open  
**When** terminal is resized  
**Then** picker should redraw correctly  
**And** selected item should remain selected (still highlighted)

**Verification**: Color assertions before/after resize

### EDGE-004: Panic Recovery

**Type**: Unit  
**Given** picker is open  
**When** a panic occurs in rendering  
**Then** terminal should be restored via `catch_unwind`  
**And** panic should propagate after cleanup

**Note**: Unit test - tests panic handling logic

---

## Test Infrastructure

### Required Test Utilities

1. **TUI Asserter Extensions**:

   - `assert_vertical_order(&[TextMatch])` - Assert items appear in order from top to bottom
   - `expect_completion_and_get_output() -> String` - Get stdout after process completes
   - Color assertions: `fg.assert_not_grayscale()`, `fg.assert_grayscale()`

2. **Test Commands** (enabled by `TEST_MODE=1`):

   - `test picker` - Test select picker with items from `TEST_PICKER_ITEMS` env var
   - `test text input` - Test text picker with prompt from `TEST_TEXT_PROMPT`
   - `test text input suggestions` - Test text picker with suggestions
   - `test confirm` - Test confirm picker with prompt from `TEST_CONFIRM_PROMPT` and default from `TEST_CONFIRM_DEFAULT`

3. **Test Output Format**:

   - Select picker: `Some("<selection>")` or `None`
   - Text picker: `Some("<input>")` or `None`
   - Confirm picker: `Some(true)`, `Some(false)`, or `None`

4. **Environment Variables**:
   - `TEST_MODE=1` - Enable test commands and output
   - `TEST_PICKER_ITEMS=item1,item2,item3` - Items for select picker
   - `TEST_TEXT_PROMPT=Enter name:` - Prompt for text picker
   - `TEST_CONFIRM_PROMPT=Continue?` - Prompt for confirm picker
   - `TEST_CONFIRM_DEFAULT=true|false` - Default selection for confirm

### Test Patterns

```rust
// Example: SP-003 Fuzzy Search Scoring
#[test]
fn test_select_picker_fuzzy_search_scoring() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    }).create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .env("TEST_MODE", "1")
        .env("TEST_PICKER_ITEMS", "application,Apple,pineapple")
        .args(&["test", "picker"])
        .build();

    let mut asserter = env.testers().pty()
        .terminal_size(40, 120)
        .run(&cmd);

    asserter.wait_for_settle();

    // Type search query
    asserter.type_text("app");
    asserter.wait_for_settle();

    // Assert vertical order
    asserter.assert_vertical_order(&[
        asserter.find_text("Apple"),
        asserter.find_text("application"),
        asserter.find_text("pineapple"),
    ]);
}

// Example: TP-004 Confirm with Enter
#[test]
fn test_text_input_confirm() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    }).create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .env("TEST_MODE", "1")
        .env("TEST_TEXT_PROMPT", "Enter name:")
        .args(&["test", "text", "input"])
        .build();

    let mut asserter = env.testers().pty()
        .terminal_size(40, 120)
        .run(&cmd);

    asserter.wait_for_settle();
    asserter.type_text("Test");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "Some(\"Test\")");
}
```

---

## Implementation Priority

### Phase 1: Test Infrastructure

1. Add `TEST_MODE` environment variable support
2. Implement test commands (test picker, test text input, test text input suggestions, test confirm)
3. Add TUI asserter extensions (assert_vertical_order, expect_completion_and_get_output, color assertions)

### Phase 2: Core Picker TUI Tests

1. SP-001 to SP-008, SP-012, SP-013 - Select picker display and navigation
2. TP-001 to TP-003, TP-007 - Text input and unicode
3. TPS-001 to TPS-005, TPS-008, TPS-009 - Suggestions and navigation
4. CP-001 to CP-005 - Confirm picker

### Phase 3: Core Picker Unit Tests

1. SP-003 - Fuzzy search scoring (already working via sublime_fuzzy)
2. TPS-006, TPS-007 - Suggestion provider edge cases

### Phase 4: Command Palette TUI Tests

1. CPAL-001 to CPAL-003, CPAL-005, CPAL-006 - Display, search, cancel
2. CPAL-004 - Unit test for command execution

### Phase 5: Add Workspace Command

1. AW-001 - Full happy path
2. AW-002 to AW-004 - Cancel at various steps
3. AW-005 to AW-008 - Input validation and persistence
4. AW-009 to AW-012 - Tag suggestions

### Phase 6: Integration & Edge Cases

1. INT-001 to INT-004 - Terminal behavior
2. EDGE-001 to EDGE-003 - Performance and resizing
3. EDGE-004 - Panic recovery (unit test)
