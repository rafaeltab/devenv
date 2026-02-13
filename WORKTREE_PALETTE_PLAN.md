# Test-Driven Plan: Adding Worktree Commands to Command Palette

## Overview

Add two new commands to the command palette:

1. **Worktree Start** - Creates a new git worktree with tmux session
2. **Worktree Complete** - Removes/cleans up a worktree and its tmux session

These commands already exist as CLI commands (`worktree start` and `worktree complete`), and we need to expose them through the interactive command palette.

---

## Architecture Summary

### Current Command Palette Pattern

- Commands implement the `Command` trait with `name()`, `description()`, and `run(&self, ctx: &mut CommandCtx)` methods
- Commands are registered in `CommandRegistry` in `main.rs`
- `CommandCtx` provides TUI methods (`select`, `input`, `confirm`, `input_with_suggestions`) and access to `workspace_repo`
- The existing CLI commands use a different pattern (`RafaeltabCommand<T>` trait with `execute(&self, options: T)`)

### Key Design Decision

Create a `WorktreeService` that encapsulates the worktree start/complete logic, shared between CLI and palette commands. This avoids bloating `CommandCtx` with many repositories while keeping the palette commands simple.

---

## Implementation Plan

### Phase 1: Create Worktree Service

**New File: `src/domain/worktree/service.rs`**

Create a service that encapsulates the worktree logic:

```rust
pub struct WorktreeService {
    workspace_repository: Rc<dyn WorkspaceRepository>,
    worktree_storage: Rc<dyn WorktreeStorage>,
    session_repository: Rc<dyn TmuxSessionRepository>,
    client_repository: Rc<dyn TmuxClientRepository>,
    tmux_storage: Rc<dyn TmuxStorage>,
    popup_repository: Option<Rc<dyn TmuxPopupRepository>>,
    description_repository: Option<Rc<dyn SessionDescriptionRepository>>,
}

impl WorktreeService {
    pub fn new(...) -> Self;

    /// Start a new worktree (used by both CLI and palette)
    pub fn start_worktree(&self, branch_name: &str, force: bool) -> Result<WorktreeStartResult, WorktreeError>;

    /// Complete a worktree (used by both CLI and palette)
    pub fn complete_worktree(&self, branch_name: Option<&str>, force: bool) -> Result<WorktreeCompleteResult, WorktreeError>;

    /// List available worktrees for selection
    pub fn list_worktrees(&self) -> Vec<WorktreeInfo>;

    /// Get current worktree if in one
    pub fn get_current_worktree(&self) -> Option<WorktreeInfo>;
}
```

### Phase 2: Create Palette Commands

**New File: `src/commands/builtin/worktree_start.rs`**

```rust
#[derive(Debug)]
pub struct WorktreeStartCommand {
    service: Rc<WorktreeService>,
}

impl Command for WorktreeStartCommand {
    fn name(&self) -> &str { "Worktree Start" }
    fn description(&self) -> &str { "Create a new git worktree with tmux session" }

    fn run(&self, ctx: &mut CommandCtx) {
        // 1. Get branch name from user
        // 2. Validate and get confirmation with details
        // 3. Execute using service
        // 4. Show result (restore terminal first)
    }
}
```

**New File: `src/commands/builtin/worktree_complete.rs`**

```rust
#[derive(Debug)]
pub struct WorktreeCompleteCommand {
    service: Rc<WorktreeService>,
}

impl Command for WorktreeCompleteCommand {
    fn name(&self) -> &str { "Worktree Complete" }
    fn description(&self) -> &str { "Remove a git worktree and its tmux session" }

    fn run(&self, ctx: &mut CommandCtx) {
        // 1. Determine if we're in a worktree or need to select one
        // 2. Show confirmation with details
        // 3. Execute using service
        // 4. Show result (restore terminal first)
    }
}
```

### Phase 3: Update Module Exports

**File: `src/commands/builtin/mod.rs`**

```rust
pub mod add_workspace;
pub mod worktree_start;
pub mod worktree_complete;

pub use add_workspace::AddWorkspaceCommand;
pub use worktree_start::WorktreeStartCommand;
pub use worktree_complete::WorktreeCompleteCommand;
```

### Phase 4: Update Main.rs Registration

**File: `src/main.rs`**

In the `Commands::CommandPalette` match arm, add registration:

```rust
// Create shared service
let worktree_service = Rc::new(WorktreeService::new(
    workspace_repository.clone(),
    // ... other repositories
));

// Register commands
registry.register(AddWorkspaceCommand::new());
registry.register(WorktreeStartCommand::new(worktree_service.clone()));
registry.register(WorktreeCompleteCommand::new(worktree_service.clone()));
```

### Phase 5: Refactor Existing CLI Commands

Update `src/commands/worktree/start.rs` and `complete.rs` to use the new `WorktreeService`, removing duplicate logic.

---

## Integration Test Cases

### File: `tests/command_palette_worktree_tests.rs`

#### CPW-001: Worktree Start Appears in Palette

**Test:** Verify "Worktree Start" command is visible in the command palette list.

```rust
#[test]
fn test_worktree_start_appears_in_palette() {
    let env = TestEnvironment::describe(|root| {
        root.git_repo();
        root.rafaeltab_config(|c| {
            c.workspace("test-workspace", |w| {
                w.path(".");
                w.name("test");
            });
        });
    }).create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);
    asserter.wait_for_settle();

    asserter.find_text("Worktree Start").assert_visible();
}
```

**Expected Behavior:**

- Command palette opens showing all available commands
- "Worktree Start" appears in the list with its description

---

#### CPW-002: Worktree Complete Appears in Palette

**Test:** Verify "Worktree Complete" command is visible in the command palette list.

**Expected Behavior:**

- "Worktree Complete" appears in the command list with its description

---

#### CPW-003: Worktree Start Flow - Full Success

**Test:** Complete happy path for creating a worktree from the palette.

```rust
#[test]
fn test_worktree_start_full_success() {
    let env = TestEnvironment::describe(|root| {
        root.git_repo();
        root.rafaeltab_config(|c| {
            c.workspace("test-workspace", |w| {
                w.path(".");
                w.name("myproject");
            });
        });
    }).create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);
    asserter.wait_for_settle();

    // Select "Worktree Start"
    asserter.type_text("worktree start");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Enter branch name
    asserter.type_text("feature-branch");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Confirm creation
    asserter.press_key(Key::Enter); // Default Yes
    asserter.wait_for_settle();

    // Verify success message
    asserter.find_text("Created worktree").assert_visible();
    asserter.find_text("Started tmux session: myproject-feature-branch").assert_visible();
}
```

**Expected Behavior:**

1. User selects "Worktree Start" from palette
2. Text input appears asking for "Branch name"
3. User types "feature-branch" and confirms
4. Confirmation dialog shows creation details (path, session name, etc.)
5. User confirms (Yes)
6. Git worktree is created at `../feature-branch`
7. Tmux session "myproject-feature-branch" is created
8. Client switches to new session
9. Success message is displayed

---

#### CPW-004: Worktree Start Flow - Cancellation at Branch Input

**Test:** Verify graceful exit when user cancels at branch name input.

**Expected Behavior:**

1. User selects "Worktree Start"
2. At branch name input, user presses Escape
3. Command exits without creating anything
4. Returns to shell without error

---

#### CPW-005: Worktree Start Flow - Cancellation at Confirmation

**Test:** Verify no worktree created when user declines confirmation.

```rust
#[test]
fn test_worktree_start_cancel_at_confirmation() {
    // Setup similar to CPW-003

    // Enter branch name
    asserter.type_text("feature-branch");
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Select "No" at confirmation
    asserter.press_key(Key::Right); // Move to "No"
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Verify no worktree created
    assert!(!Path::new("../feature-branch").exists());
}
```

**Expected Behavior:**

1. User enters branch name
2. Confirmation dialog shows "Create worktree for branch 'feature-branch'?"
3. User selects "No"
4. Command exits without creating worktree or tmux session
5. Returns to shell without error

---

#### CPW-006: Worktree Complete Flow - From Within Worktree

**Test:** Complete a worktree when currently inside it.

```rust
#[test]
fn test_worktree_complete_from_within_worktree() {
    let env = TestEnvironment::describe(|root| {
        root.git_repo();
        root.worktree("feature-branch"); // Pre-create worktree
        root.rafaeltab_config(|c| {
            c.workspace("test-workspace", |w| {
                w.path(".");
                w.name("myproject");
            });
        });
    }).create();

    // Change to worktree directory
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .current_dir("../feature-branch") // Execute from within worktree
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);
    asserter.wait_for_settle();

    // Select "Worktree Complete"
    asserter.type_text("worktree complete");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Should show confirmation for current worktree
    asserter.find_text("Complete worktree for branch 'feature-branch'?").assert_visible();

    // Confirm
    asserter.press_key(Key::Enter); // Yes
    asserter.wait_for_settle();

    // Verify worktree removed
    assert!(!Path::new("../feature-branch").exists());
}
```

**Expected Behavior:**

1. User is in worktree directory
2. User selects "Worktree Complete" from palette
3. Since in a worktree, skips selection picker
4. Confirmation shows "Complete worktree for branch 'feature-branch'?"
5. User confirms
6. Client switches to main workspace session
7. Tmux session "myproject-feature-branch" is killed
8. Git worktree is removed
9. Success message displayed

---

#### CPW-007: Worktree Complete Flow - Selection Required

**Test:** Complete a worktree when not currently in one (shows selection picker).

```rust
#[test]
fn test_worktree_complete_with_selection() {
    let env = TestEnvironment::describe(|root| {
        root.git_repo();
        root.worktree("feature-branch");
        root.worktree("bugfix-branch");
        root.rafaeltab_config(|c| {
            c.workspace("test-workspace", |w| {
                w.path(".");
                w.name("myproject");
            });
        });
    }).create();

    // Execute from main repo (not in a worktree)
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);
    asserter.wait_for_settle();

    // Select "Worktree Complete"
    asserter.type_text("worktree complete");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Should show picker with available worktrees
    asserter.find_text("feature-branch").assert_visible();
    asserter.find_text("bugfix-branch").assert_visible();

    // Select first worktree
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Should show confirmation
    asserter.find_text("Complete worktree").assert_visible();
}
```

**Expected Behavior:**

1. User is in main repo (not a worktree)
2. User selects "Worktree Complete"
3. Picker appears showing available worktrees: "feature-branch", "bugfix-branch"
4. User selects one
5. Confirmation dialog appears
6. After confirmation, selected worktree is removed

---

#### CPW-008: Worktree Start - Not in Workspace Error

**Test:** Error handling when executed outside a configured workspace.

**Expected Behavior:**

1. User selects "Worktree Start"
2. User enters branch name
3. Error message displayed: "Not in a known workspace"
4. Command exits gracefully
5. No worktree created

---

#### CPW-009: Worktree Start - Not in Git Repository Error

**Test:** Error handling when in a workspace but not in a git repo.

**Expected Behavior:**

1. User selects "Worktree Start"
2. User enters branch name
3. Error message displayed: "Not in a git repository"
4. Command exits gracefully

---

#### CPW-010: Worktree Start - Path Conflict

**Test:** Handling when target worktree path already exists.

**Expected Behavior:**

1. User enters branch name "feature-branch"
2. System detects directory `../feature-branch` already exists
3. Error message displayed: "Path conflict: {path} already exists"
4. Command exits without creating worktree

---

#### CPW-011: Worktree Start - Missing Config (No Force)

**Test:** Behavior when no worktree config exists and force flag not set.

**Expected Behavior:**

1. User enters branch name
2. System checks for worktree config (global or workspace-specific)
3. No config found
4. Error displayed: "Worktree config missing for workspace '{name}'"
5. Suggests using `--force` or configuring worktree settings

---

#### CPW-012: Worktree Complete - Safety Check - Uncommitted Changes

**Test:** Prevent completion when worktree has uncommitted changes.

```rust
#[test]
fn test_worktree_complete_prevents_with_uncommitted_changes() {
    // Setup worktree with uncommitted changes
    let env = TestEnvironment::describe(|root| {
        root.git_repo();
        root.worktree_with_uncommitted_changes("feature-branch");
        // ... config
    }).create();

    // Execute from within worktree
    // ...

    // Confirm completion
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Error displayed
    asserter.find_text("has uncommitted changes").assert_visible();

    // Worktree still exists
    assert!(Path::new("../feature-branch").exists());
}
```

**Expected Behavior:**

1. User in worktree with uncommitted changes
2. User selects "Worktree Complete" and confirms
3. Error: "Worktree has uncommitted changes"
4. Suggests using `--force` or committing changes first
5. Worktree remains intact

---

#### CPW-013: Worktree Complete - Safety Check - Unpushed Commits

**Test:** Prevent completion when worktree has unpushed commits.

**Expected Behavior:**

1. User in worktree with unpushed commits
2. User attempts to complete
3. Error: "Worktree has unpushed commits"
4. Suggests using `--force` or pushing changes

---

#### CPW-014: Worktree Complete - Cannot Complete Main Repo

**Test:** Prevent completion when in main repository (not a worktree).

**Expected Behavior:**

1. User in main repository
2. User selects "Worktree Complete"
3. Error: "Cannot complete main repository"
4. Suggests being in a worktree directory

---

#### CPW-015: Worktree Complete - Cancellation

**Test:** Verify no worktree removed when user cancels.

**Expected Behavior:**

1. User selects "Worktree Complete"
2. User selects a worktree from picker
3. User cancels at confirmation (No or Escape)
4. Worktree remains intact

---

#### CPW-016: Branch Name Input with Suggestions

**Test:** Branch name input shows existing branches as suggestions.

**Expected Behavior:**

1. User selects "Worktree Start"
2. Text input appears with suggestions
3. As user types, suggestions show matching branches from:
   - Local branches
   - Remote branches
4. User can Tab-complete or select from list

---

#### CPW-017: Worktree Selection Shows Details

**Test:** Worktree picker shows useful information about each worktree.

**Expected Behavior:**

1. User selects "Worktree Complete" from main repo
2. Picker shows worktrees with:
   - Branch name
   - Last commit message (truncated)
   - Uncommitted changes indicator
   - Unpushed commits indicator
3. Worktrees with issues are highlighted

---

## Error Handling Strategy

All errors in palette commands should:

1. Restore terminal before displaying error (call `ctx.restore()`)
2. Show clear, actionable error messages
3. Exit gracefully without panicking
4. For safety errors (uncommitted changes, etc.), suggest next steps (--force, commit, etc.)

Example error display pattern:

```rust
match result {
    Ok(success) => {
        ctx.restore()?;
        println!("✓ {}", success.message);
    }
    Err(e) => {
        ctx.restore()?;
        eprintln!("Error: {}", e);
        if let Some(suggestion) = e.suggestion() {
            eprintln!("{}", suggestion);
        }
    }
}
```

---

## Test Environment Setup Helpers

Add these helpers to `tests/common/rafaeltab_descriptors.rs`:

```rust
impl RafaeltabRootDescriptor {
    /// Setup a git repository
    pub fn git_repo(&mut self) -> &mut Self;

    /// Create a worktree with the given branch name
    pub fn worktree(&mut self, branch_name: &str) -> &mut Self;

    /// Create a worktree with uncommitted changes
    pub fn worktree_with_uncommitted_changes(&mut self, branch_name: &str) -> &mut Self;

    /// Create a worktree with unpushed commits
    pub fn worktree_with_unpushed_commits(&mut self, branch_name: &str) -> &mut Self;
}
```

---

## Success Criteria

1. ✅ Both commands appear in the command palette
2. ✅ Worktree Start successfully creates worktree + tmux session
3. ✅ Worktree Complete successfully removes worktree + kills session
4. ✅ Cancellation at any point leaves system unchanged
5. ✅ All safety checks work (uncommitted changes, unpushed commits)
6. ✅ Error messages are clear and actionable
7. ✅ Integration tests pass in CI
8. ✅ Existing CLI commands still work (backwards compatibility)

---

## Implementation Order

1. Create `WorktreeService` with extracted logic from CLI commands
2. Refactor existing CLI commands to use `WorktreeService`
3. Create `WorktreeStartCommand` for palette
4. Create `WorktreeCompleteCommand` for palette
5. Update module exports
6. Update `main.rs` registration
7. Write integration tests
8. Run full test suite
9. Manual testing

---

## Notes

- Keep palette commands focused on UI flow; delegate business logic to `WorktreeService`
- Use existing patterns from `AddWorkspaceCommand` as reference
- Ensure terminal is always restored before printing results
- Consider adding "force" option in confirmation dialog for expert users
- Worktree selection picker should prioritize worktrees from current workspace
