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

1. **Add a single tmux session via `tmux_session`** in the test environment descriptor to ensure tmux server is running
2. **Remove `tmux start` calls** from all tests
3. **Verify sessions are created by switch command** in each test

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

## Implementation Plan

### Step 1: Update Existing Tests

**File:** `apps/cli/tests/tui_tmux_switch_tests.rs`

#### Test 1: test_tmux_switch_displays_sessions

**UPDATE** the environment descriptor to add a single tmux session:

```rust
let env = TestEnvironment::describe(|root| {
    root.rafaeltab_config(|c| {
        c.tmux_session("project-a", Some("Project A"), &[("shell", None)]);
        c.tmux_session("project-b", Some("Project B"), &[("shell", None)]);
        c.tmux_session("project-c", Some("Project C"), &[("shell", None)]);
        // Add a dummy session to ensure tmux server is running
        c.tmux_session("_dummy", Some("_dummy"), &[("shell", None)]);
    });
    // ... rest of descriptor
})
```

**REMOVE** the `tmux start` command block (lines 33-39).

**REMOVE** the cancellation at the end and **ADD** session creation verification:

```rust
// Instead of canceling, select and create a session
asserter.press_key(Key::Enter);
let exit_code = asserter.expect_completion();
assert_eq!(exit_code, 0);

// Verify the session was created
assert!(
    env.tmux().session_exists("Project A"),
    "Session should be created on selection"
);
```

#### Test 2: test_tmux_switch_fuzzy_filtering

**UPDATE** the environment descriptor to add a single tmux session:

```rust
root.rafaeltab_config(|c| {
    c.tmux_session("frontend", Some("Frontend Dev"), &[("shell", None)]);
    c.tmux_session("backend", Some("Backend API"), &[("shell", None)]);
    c.tmux_session("database", Some("Database Work"), &[("shell", None)]);
    // Add a dummy session to ensure tmux server is running
    c.tmux_session("_dummy", Some("_dummy"), &[("shell", None)]);
});
```

**REMOVE** the `tmux start` command block (lines 100-106).

**KEEP** the rest (filtering doesn't require workspace sessions to exist, only the dummy session)

#### Test 3: test_tmux_switch_navigation

**UPDATE** the environment descriptor to add a single tmux session:

```rust
root.rafaeltab_config(|c| {
    c.tmux_session("first", Some("First Session"), &[("shell", None)]);
    c.tmux_session("second", Some("Second Session"), &[("shell", None)]);
    c.tmux_session("third", Some("Third Session"), &[("shell", None)]);
    // Add a dummy session to ensure tmux server is running
    c.tmux_session("_dummy", Some("_dummy"), &[("shell", None)]);
});
```

**REMOVE** the `tmux start` command block (lines 176-182).

#### Test 4: test_tmux_switch_cancel_with_q

**UPDATE** the environment descriptor to add a single tmux session:

```rust
root.rafaeltab_config(|c| {
    c.tmux_session("test", Some("Test Session"), &[("shell", None)]);
    // Add a dummy session to ensure tmux server is running
    c.tmux_session("_dummy", Some("_dummy"), &[("shell", None)]);
});
```

**REMOVE** the `tmux start` command block (lines 242-248).

**ADD** at end (after cancellation):

```rust
// Verify the configured session was NOT created (it only exists in config, not tmux)
// The dummy session should exist
assert!(
    env.tmux().session_exists("_dummy"),
    "Dummy session should exist to keep tmux running"
);
```

#### Test 5: test_tmux_switch_cancel_with_ctrl_c

**UPDATE** the environment descriptor to add a single tmux session:

```rust
root.rafaeltab_config(|c| {
    c.tmux_session("test", Some("Test Session"), &[("shell", None)]);
    // Add a dummy session to ensure tmux server is running
    c.tmux_session("_dummy", Some("_dummy"), &[("shell", None)]);
});
```

**REMOVE** the `tmux start` command block (lines 285-291).

**ADD** at end (after cancellation):

```rust
// Verify the dummy session exists (tmux is running)
assert!(
    env.tmux().session_exists("_dummy"),
    "Dummy session should exist to keep tmux running"
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
            // Add a dummy session to ensure tmux server is running
            c.tmux_session("_dummy", Some("_dummy"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.rafaeltab_workspace("myworkspace", "My Workspace", |_w| {});
            });
        });
    })
    .create();

    // Verify no workspace session exists yet (only dummy exists)
    assert!(
        !env.tmux().session_exists("My Workspace"),
        "Workspace session should not exist before switch"
    );
    assert!(
        env.tmux().session_exists("_dummy"),
        "Dummy session should exist to keep tmux running"
    );

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "switch"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // Workspace should be displayed
    asserter.find_text("My Workspace").assert_visible();

    // Select and confirm
    asserter.press_key(Key::Enter);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0);

    // Verify session was created
    assert!(
        env.tmux().session_exists("My Workspace"),
        "Session should be created after selection"
    );

    // Verify windows were created correctly
    let session = env
        .find_tmux_session("My Workspace")
        .expect("Session should exist");
    let windows = session.windows();
    assert_eq!(windows.len(), 1, "Should have 1 window from config");
    assert!(windows.iter().any(|w| w.contains("editor")), "Should have editor window");
}
```

---

## Implementation Checklist

### Phase 1: Verify Test Infrastructure

- [ ] Verify `env.tmux().session_exists()` works
- [ ] Verify `env.find_tmux_session().windows()` works
- [ ] Test that TUI tests can run with a dummy tmux session

### Phase 2: Update Existing Tests

- [ ] Update `test_tmux_switch_displays_sessions` - remove tmux start, add assertions
- [ ] Update `test_tmux_switch_fuzzy_filtering` - remove tmux start
- [ ] Update `test_tmux_switch_navigation` - remove tmux start
- [ ] Update `test_tmux_switch_cancel_with_q` - remove tmux start, verify no session
- [ ] Update `test_tmux_switch_cancel_with_ctrl_c` - remove tmux start, verify no session

### Phase 3: Add New Tests

- [ ] Add `test_tmux_switch_creates_session_on_selection`

### Phase 4: Validation

- [ ] Run: `cargo test --test tui_tmux_switch_tests`
- [ ] Verify all existing tests still pass
- [ ] Verify new tests pass

### Phase 5: Cleanup

- [ ] Review all test output for warnings
- [ ] Ensure no sessions leak between tests
- [ ] Document any limitations (e.g., TUI requires tmux running)

---

## Files Modified

1. 🔧 **MODIFIED:** `apps/cli/tests/tui_tmux_switch_tests.rs` - Update all 5 tests + add 1 new

---

## Known Limitations

1. **TUI tests require tmux running:** The `run_cli_tui` helper needs a tmux server to be active for I/O. We use a dummy session in the config to ensure tmux is running.

2. **Testing session creation in TUI:** Some tests cancel before selection, so they verify the workspace session was NOT created (only the dummy session exists). This is correct behavior.

3. **Timing issues:** Session creation might take a moment. Tests might need small sleeps after Enter key press.

---

## Success Criteria

- [ ] All 5 existing tests updated to NOT pre-create sessions via `tmux start`
- [ ] All updated tests pass
- [ ] New test `test_tmux_switch_creates_session_on_selection` passes
- [ ] Tests now verify session creation behavior
- [ ] No regressions in other test suites
