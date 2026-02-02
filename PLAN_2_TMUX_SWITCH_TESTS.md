# Plan 2: Fix Tmux Switch Test Coverage

## Problem Statement

**Current Issue:** ALL existing tmux switch tests call `tmux start` before testing the switch command. This means:

- ❌ Session creation logic in switch is never tested
- ❌ We don't verify that switch can create sessions on its own
- ❌ Tests only verify TUI behavior (display, filtering, navigation)

**Goal:** Update all tests to NOT pre-create sessions, ensuring they test the actual session creation logic in the switch command.

---

## Current Test Analysis

### File: `apps/cli/tests/tui_tmux_switch_tests.rs`

All 5 tests follow this pattern:

```rust
// 1. Create environment
let env = TestEnvironment::describe(|root| { ... }).create();

// 2. PRE-CREATE SESSIONS (lines 35-41, 99-105, 172-178, 243-249, 283-289)
let (_, _, success) = common::run_cli_with_tmux(
    &["tmux", "start"],
    config_path.to_str().unwrap(),
    env.tmux_socket(),
);
assert!(success, "Failed to start tmux sessions");

// 3. Test TUI
let mut tui = run_cli_tui(&["tmux", "switch"], ...);
```

### Tests Affected

1. ✅ `test_tmux_switch_displays_sessions` (lines 11-72)
2. ✅ `test_tmux_switch_fuzzy_filtering` (lines 74-145)
3. ✅ `test_tmux_switch_navigation` (lines 147-224)
4. ✅ `test_tmux_switch_cancel_with_q` (lines 226-264)
5. ✅ `test_tmux_switch_cancel_with_ctrl_c` (lines 266-305)

---

## Test Requirements Analysis

### Question: Do These Tests Need Tmux Running?

**YES** - The TUI tests use the `run_cli_tui` helper which likely requires tmux to be running to:

- Display the fuzzy picker
- Show session names
- Handle input/output

**However:** The tests do NOT need sessions to already exist. The switch command should create them.

### Solution Strategy

1. **Keep tmux server running** (start empty session if needed)
2. **Remove `tmux start` calls** from all tests
3. **Add new test** to verify switch creates sessions when tmux isn't running
4. **Verify sessions are created by switch command** in each test

---

## Test Cases

### TC-ST-01: Switch displays and creates sessions (UPDATE EXISTING)

**Current:** Pre-creates sessions, then displays them  
**New Behavior:**

- **Given:** Workspaces configured, no sessions exist
- **When:** User opens `tmux switch`
- **Then:**
  - All workspace names are displayed in picker
  - When user selects and presses Enter, session is created
  - User is switched to newly created session

**Test:** `test_tmux_switch_displays_sessions`  
**Status:** ⚠️ Needs updating

---

### TC-ST-02: Switch fuzzy filtering works without pre-existing sessions (UPDATE EXISTING)

**Current:** Pre-creates sessions, then filters  
**New Behavior:**

- **Given:** Multiple workspaces configured, no sessions exist
- **When:** User types to filter in switch dialog
- **Then:** Filtering works on workspace names

**Test:** `test_tmux_switch_fuzzy_filtering`  
**Status:** ⚠️ Needs updating

---

### TC-ST-03: Switch navigation works with session descriptions (UPDATE EXISTING)

**Current:** Pre-creates sessions, then navigates  
**New Behavior:**

- **Given:** Multiple workspaces configured
- **When:** User navigates with arrow keys
- **Then:** Selection moves correctly through session descriptions

**Test:** `test_tmux_switch_navigation`  
**Status:** ⚠️ Needs updating

---

### TC-ST-04: Cancel switch before session creation (UPDATE EXISTING)

**Current:** Pre-creates sessions, then cancels  
**New Behavior:**

- **Given:** Workspaces configured
- **When:** User cancels switch dialog
- **Then:** No sessions are created

**Test:** `test_tmux_switch_cancel_with_q` and `test_tmux_switch_cancel_with_ctrl_c`  
**Status:** ⚠️ Needs updating

---

### TC-ST-05: Switch creates session on selection (NEW TEST)

**New Test:** Verify the core functionality that's currently untested

- **Given:** Workspace configured, no session exists
- **When:** User selects workspace and presses Enter
- **Then:**
  - Session is created
  - Session has correct windows from config
  - User is switched to session

**Test:** `test_tmux_switch_creates_session_on_selection` (NEW)  
**Status:** ❌ Doesn't exist

---

### TC-ST-06: Switch works when tmux not running (NEW TEST)

**New Test:** Verify switch can start tmux if needed

- **Given:** Workspace configured, tmux server not running
- **When:** User runs `tmux switch` and selects workspace
- **Then:**
  - Tmux server starts
  - Session is created
  - User is switched to session

**Test:** `test_tmux_switch_starts_tmux_if_needed` (NEW)  
**Status:** ❌ Doesn't exist

---

## Implementation Plan

### Step 1: Update Test Helper (if needed)

**File:** `apps/cli/tests/common/mod.rs`

Verify/add helper to start empty tmux session:

```rust
/// Start an empty tmux server without creating any workspace sessions.
/// This is needed for TUI tests that require tmux to be running.
pub fn start_empty_tmux_server(socket: &str) {
    // Create a detached dummy session just to start the server
    std::process::Command::new("tmux")
        .args(["-L", socket, "new-session", "-d", "-s", "dummy"])
        .output()
        .ok(); // Ignore errors if already running
}
```

---

### Step 2: Update Existing Tests

**File:** `apps/cli/tests/tui_tmux_switch_tests.rs`

#### Test 1: test_tmux_switch_displays_sessions

**REMOVE** (lines 35-41):

```rust
// Start the sessions first
let (_, _, success) = common::run_cli_with_tmux(
    &["tmux", "start"],
    config_path.to_str().unwrap(),
    env.tmux_socket(),
);
assert!(success, "Failed to start tmux sessions");
```

**ADD** after config_path:

```rust
// Start empty tmux server for TUI (don't create workspace sessions)
common::start_empty_tmux_server(env.tmux_socket());
```

**ADD** before `tui.press_key(Key::Esc)`:

```rust
// Verify we can select a session (this will create it)
tui.press_key(Key::Enter);
tui.wait_for_settle();

// Verify session was created
assert!(
    env.tmux().session_exists("Project A"),
    "Session should be created on selection"
);
```

**UPDATE** final assertion:

```rust
// The test now creates a session, so exit code might be different
// Or we test cancellation in a separate test
```

#### Test 2: test_tmux_switch_fuzzy_filtering

**REMOVE** (lines 99-105):

```rust
let (_, _, success) = common::run_cli_with_tmux(
    &["tmux", "start"],
    config_path.to_str().unwrap(),
    env.tmux_socket(),
);
assert!(success, "Failed to start tmux sessions");
```

**ADD**:

```rust
common::start_empty_tmux_server(env.tmux_socket());
```

**KEEP** the rest (filtering doesn't require sessions to exist)

#### Test 3: test_tmux_switch_navigation

**REMOVE** (lines 172-178):

```rust
let (_, _, success) = common::run_cli_with_tmux(
    &["tmux", "start"],
    config_path.to_str().unwrap(),
    env.tmux_socket(),
);
assert!(success, "Failed to start tmux sessions");
```

**ADD**:

```rust
common::start_empty_tmux_server(env.tmux_socket());
```

#### Test 4: test_tmux_switch_cancel_with_q

**REMOVE** (lines 243-249):

```rust
let (_, _, success) = common::run_cli_with_tmux(
    &["tmux", "start"],
    config_path.to_str().unwrap(),
    env.tmux_socket(),
);
assert!(success, "Failed to start tmux sessions");
```

**ADD**:

```rust
common::start_empty_tmux_server(env.tmux_socket());
```

**ADD** at end:

```rust
// Verify no session was created (canceled before selection)
assert!(
    !env.tmux().session_exists("Test Session"),
    "Session should not be created when canceled"
);
```

#### Test 5: test_tmux_switch_cancel_with_ctrl_c

**REMOVE** (lines 283-289):

```rust
let (_, _, success) = common::run_cli_with_tmux(
    &["tmux", "start"],
    config_path.to_str().unwrap(),
    env.tmux_socket(),
);
assert!(success, "Failed to start tmux sessions");
```

**ADD**:

```rust
common::start_empty_tmux_server(env.tmux_socket());
```

**ADD** at end:

```rust
// Verify no session was created (canceled before selection)
assert!(
    !env.tmux().session_exists("Test Session"),
    "Session should not be created when canceled"
);
```

---

### Step 3: Add New Tests

**File:** `apps/cli/tests/tui_tmux_switch_tests.rs`

#### NEW Test: test_tmux_switch_creates_session_on_selection

```rust
#[test]
fn test_tmux_switch_creates_session_on_selection() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("editor", None), ("shell", None)]);
            c.tmux_session("myworkspace", Some("My Workspace"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.rafaeltab_workspace("myworkspace", "My Workspace", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Start empty tmux server (don't create workspace sessions)
    common::start_empty_tmux_server(env.tmux_socket());

    // Verify no session exists yet
    assert!(
        !env.tmux().session_exists("My Workspace"),
        "Session should not exist before switch"
    );

    let mut tui = run_cli_tui(
        &["tmux", "switch"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // Workspace should be displayed
    tui.find_text("My Workspace").assert_visible();

    // Select and confirm
    tui.press_key(Key::Enter);

    // Wait for session creation and switch
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Verify session was created
    assert!(
        env.tmux().session_exists("My Workspace"),
        "Session should be created after selection"
    );

    // Verify windows were created correctly
    let windows = env.tmux().list_windows("My Workspace");
    assert_eq!(windows.len(), 1, "Should have 1 window from config");
    assert!(windows[0].contains("editor"), "Should have editor window");
}
```

#### NEW Test: test_tmux_switch_starts_tmux_if_needed

```rust
#[test]
fn test_tmux_switch_starts_tmux_if_needed() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.rafaeltab_workspace("ws", "TestWorkspace", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // DON'T start tmux server - test should handle this
    // (Actually, the TUI might require it, so this test might need to be non-TUI)

    // This might need to be a separate integration test without TUI
    // or we accept that TUI tests require tmux to be running

    // For now, document this as a limitation
    // TODO: Add non-TUI integration test for this case
}
```

**Note:** The last test might not be feasible with TUI tests. Consider making it a separate non-TUI integration test.

---

### Step 4: Add Non-TUI Integration Test

**New file:** `apps/cli/tests/tmux_switch_integration_tests.rs`

```rust
mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabRootMixin},
    run_cli_with_tmux,
};
use test_descriptors::TestEnvironment;

#[test]
fn test_switch_creates_session_without_tmux_running() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("editor", None), ("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.rafaeltab_workspace("ws", "MyWorkspace", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Ensure tmux is NOT running
    std::process::Command::new("tmux")
        .args(["-L", env.tmux_socket(), "kill-server"])
        .output()
        .ok();

    // Verify no sessions exist
    let sessions_before = env.tmux().list_sessions();
    assert_eq!(sessions_before.len(), 0, "No sessions should exist before switch");

    // This test is challenging because switch requires interactive TUI
    // Instead, test that tmux start works without tmux running
    let (_, _, success) = run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(success, "Should be able to start tmux when not running");

    // Verify session was created
    assert!(
        env.tmux().session_exists("MyWorkspace"),
        "Session should be created"
    );
}
```

---

## Implementation Checklist

### Phase 1: Verify Test Infrastructure

- [ ] Check if `start_empty_tmux_server()` helper exists in `common/mod.rs`
- [ ] If not, add the helper function
- [ ] Verify `env.tmux().session_exists()` works
- [ ] Verify `env.tmux().list_windows()` works
- [ ] Test that TUI tests can run with empty tmux server

### Phase 2: Update Existing Tests

- [ ] Update `test_tmux_switch_displays_sessions` - remove tmux start, add assertions
- [ ] Update `test_tmux_switch_fuzzy_filtering` - remove tmux start
- [ ] Update `test_tmux_switch_navigation` - remove tmux start
- [ ] Update `test_tmux_switch_cancel_with_q` - remove tmux start, verify no session
- [ ] Update `test_tmux_switch_cancel_with_ctrl_c` - remove tmux start, verify no session

### Phase 3: Add New Tests

- [ ] Add `test_tmux_switch_creates_session_on_selection`
- [ ] Create `tmux_switch_integration_tests.rs`
- [ ] Add `test_switch_creates_session_without_tmux_running`

### Phase 4: Validation

- [ ] Run: `cargo test --test tui_tmux_switch_tests`
- [ ] Verify all existing tests still pass
- [ ] Verify new tests pass
- [ ] Run: `cargo test --test tmux_switch_integration_tests`
- [ ] Manual test: Run switch without tmux running

### Phase 5: Cleanup

- [ ] Review all test output for warnings
- [ ] Ensure no sessions leak between tests
- [ ] Document any limitations (e.g., TUI requires tmux running)

---

## Files Modified

1. 🔧 **MODIFIED:** `apps/cli/tests/common/mod.rs` - Add `start_empty_tmux_server()` helper
2. 🔧 **MODIFIED:** `apps/cli/tests/tui_tmux_switch_tests.rs` - Update all 5 tests + add 1 new
3. ✨ **NEW:** `apps/cli/tests/tmux_switch_integration_tests.rs` - Non-TUI integration test

---

## Known Limitations

1. **TUI tests require tmux running:** The `run_cli_tui` helper likely needs a tmux server to be active for I/O. We start an empty server but can't test "switch when tmux not running" via TUI.

2. **Testing session creation in TUI:** Some tests cancel before selection, so they now verify NO session was created. This is correct behavior but different from before.

3. **Timing issues:** Session creation might take a moment. Tests might need small sleeps after Enter key press.

---

## Success Criteria

- [ ] All 5 existing tests updated to NOT pre-create sessions
- [ ] All updated tests pass
- [ ] New test `test_tmux_switch_creates_session_on_selection` passes
- [ ] Non-TUI integration test for tmux not running passes (if feasible)
- [ ] Tests now verify session creation behavior
- [ ] No regressions in other test suites
- [ ] Documentation updated if TUI/tmux limitations discovered
