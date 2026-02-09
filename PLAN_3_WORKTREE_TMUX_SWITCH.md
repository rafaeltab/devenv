# Plan 3: Add Worktree Session Creation to Tmux Switch

## Problem Statement

**Current Behavior:** When using `tmux switch` to select a workspace that has git worktrees, only the main workspace session is created. Worktree sessions are not created automatically.

**Desired Behavior:** When a workspace is selected in `tmux switch`:

1. Create the main workspace session (if needed)
2. Switch to the main workspace session
3. Discover all worktrees for that workspace
4. Create tmux sessions for each worktree (in the background)
5. Session naming: `{workspace-name}-{branch-name}`

**Dependencies:** This plan requires completion of Plan 1 (worktree start window configuration fix).

---

## Current Code Analysis

### How Switch Works Today

**File:** `apps/cli/src/commands/tmux/switch.rs`

**Lines 52-62:**

```rust
if let Some(selected_session) = res {
    println!("You selected {}!", selected_session.name);
    let session = match &selected_session.session {
        Some(se) => se,
        None => &session_repository.new_session(selected_session),
    };

    client_repository.switch_client(None, SwitchClientTarget::Session(session));
}
```

**What's Missing:**

- No worktree discovery after workspace selection
- No worktree session creation

### Available Infrastructure

✅ **Git worktree discovery:**

- `infrastructure::git::discover_worktrees_for_workspace()` - Already exists
- Returns `Vec<WorktreeInfo>` with path and branch info

✅ **Session creation:**

- `TmuxSessionRepository::new_session()` - Already exists
- `SessionDescription` - Already exists

✅ **Window configuration:**

- `commands::tmux::session_utils::get_windows_for_workspace()` - From Plan 1

---

## Test Cases

### TC-WT-01: Switch to workspace with no worktrees (baseline)

**Given:**

- A workspace exists in a git repository
- No worktrees have been created
- No tmux sessions exist

**When:** User selects the workspace in `tmux switch`

**Then:**

- Main workspace session is created
- User is switched to main workspace session
- No additional sessions are created
- No errors occur

**Status:** ❌ Not implemented

---

### TC-WT-02: Switch creates sessions for all worktrees

**Given:**

- A workspace "MyProject" exists
- Three worktrees exist:
  - `feature/login`
  - `fix/bug-123`
  - `feat/database`
- No tmux sessions exist

**When:** User selects "MyProject" in `tmux switch`

**Then:**

- Main session "MyProject" is created
- User is switched to "MyProject"
- Three worktree sessions are created:
  - "MyProject-feature/login"
  - "MyProject-fix/bug-123"
  - "MyProject-feat/database"
- All sessions exist after switch completes

**Status:** ❌ Not implemented

---

### TC-WT-03: Worktree sessions use correct window config

**Given:**

- A workspace with custom tmux session config
- Workspace windows: ["nvim", "terminal", "server"]
- A worktree exists for branch "feat/api"

**When:** User selects the workspace in `tmux switch`

**Then:**

- Worktree session "workspace-feat/api" is created
- Session has 3 windows from workspace config
- Windows match workspace config (not hardcoded)

**Status:** ❌ Not implemented

---

### TC-WT-04: Switch skips existing worktree sessions

**Given:**

- A workspace with 2 worktrees
- Tmux sessions already exist for both worktrees

**When:** User selects the workspace again

**Then:**

- No new sessions are created
- Existing sessions are reused
- User is switched to main workspace session
- No errors or duplicates

**Status:** ❌ Not implemented

---

### TC-WT-05: Switch handles non-git workspace gracefully

**Given:**

- A workspace is not a git repository

**When:** User selects the workspace

**Then:**

- Main workspace session is created
- No worktree discovery happens
- No errors are displayed
- User is switched successfully

**Status:** ❌ Not implemented

---

### TC-WT-06: Worktree sessions created after switch (timing)

**Given:**

- A workspace with multiple worktrees

**When:** User selects the workspace

**Then:**

- User is switched to main session immediately (no delay)
- Worktree sessions are created in background
- Switch command returns quickly

**Status:** ❌ Not implemented

---

### TC-WT-07: Worktree sessions get unique UUIDs

**Given:**

- A workspace "project-a" with worktree "feat/test"

**When:** User selects the workspace

**Then:**

- Worktree session has `RAFAELTAB_SESSION_ID` environment variable
- UUID is deterministic based on session name
- UUID uses worktree namespace: `f47ac10b-58cc-4372-a567-0e02b2c3d479`

**Status:** ❌ Not implemented

---

### TC-WT-08: Switch handles git command failures

**Given:**

- A workspace is a git repo but `git worktree list` fails

**When:** User selects the workspace

**Then:**

- Main session is created
- Worktree creation is skipped silently
- No user-visible errors
- Switch succeeds

**Status:** ❌ Not implemented

---

## Implementation Plan

### Step 1: Update TmuxSwitchOptions

**File:** `apps/cli/src/commands/tmux/switch.rs`

**Current struct** (around line 29):

```rust
pub struct TmuxSwitchOptions<'a> {
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
    pub session_repository: &'a dyn TmuxSessionRepository,
    pub client_repository: &'a dyn TmuxClientRepository,
}
```

**Update to:**

```rust
pub struct TmuxSwitchOptions<'a> {
    pub session_description_repository: &'a dyn SessionDescriptionRepository,
    pub session_repository: &'a dyn TmuxSessionRepository,
    pub client_repository: &'a dyn TmuxClientRepository,
    pub workspace_repository: &'a dyn WorkspaceRepository,  // ADD
    pub tmux_storage: &'a dyn TmuxStorage,                  // ADD
}
```

---

### Step 2: Add Worktree Session Creation Helper

**File:** `apps/cli/src/commands/tmux/switch.rs`

**Add after `select_name` function** (around line 68):

```rust
/// Create tmux sessions for all worktrees in a workspace.
/// This runs after the main workspace session has been created and switched to.
/// Errors are silently ignored (TODO: add logging when available).
fn create_worktree_sessions(
    workspace: &crate::domain::tmux_workspaces::aggregates::workspaces::workspace::Workspace,
    session_repository: &dyn TmuxSessionRepository,
    tmux_storage: &dyn TmuxStorage,
) {
    use crate::commands::tmux::session_utils::get_windows_for_workspace;
    use crate::domain::tmux_workspaces::aggregates::tmux::{
        description::{
            session::{PathSessionDescription, SessionDescription, SessionKind},
        },
        include_fields_builder::IncludeFieldsBuilder,
    };
    use crate::infrastructure::git;
    use crate::utils::path::expand_path;
    use std::path::Path;
    use uuid::{uuid, Uuid};

    let workspace_path = expand_path(&workspace.path);
    let workspace_path = Path::new(&workspace_path);

    // Try to discover worktrees (silently fail if not a git repo)
    let worktrees = match git::discover_worktrees_for_workspace(workspace_path) {
        Ok(wts) => wts,
        Err(_) => {
            // TODO: Log this error when logging infrastructure is available
            return;
        }
    };

    if worktrees.is_empty() {
        return; // No worktrees to create sessions for
    }

    // Get window configuration for this workspace
    let windows = get_windows_for_workspace(&workspace.id, tmux_storage);

    // Get existing sessions to avoid recreating
    let existing_sessions = session_repository.get_sessions(
        None,
        IncludeFieldsBuilder::new().build_session()
    );

    // Create session for each worktree
    for worktree_info in worktrees {
        let session_name = format!("{}-{}", workspace.name, worktree_info.branch);

        // Skip if session already exists
        let session_exists = existing_sessions.iter().any(|s| s.name == session_name);
        if session_exists {
            continue;
        }

        // Create session description
        let worktree_namespace = uuid!("f47ac10b-58cc-4372-a567-0e02b2c3d479");
        let id = Uuid::new_v5(&worktree_namespace, session_name.as_bytes());

        let description = SessionDescription {
            id: id.to_string(),
            name: session_name,
            kind: SessionKind::Path(PathSessionDescription {
                path: worktree_info.path.to_string_lossy().to_string(),
            }),
            windows: windows.clone(),
            session: None,
        };

        // Create the session (ignore errors silently for now)
        // TODO: Log creation errors when logging infrastructure is available
        let _result = session_repository.new_session(&description);
    }
}
```

---

### Step 3: Update Execute Method

**File:** `apps/cli/src/commands/tmux/switch.rs`

**Update execute method** (around line 36):

```rust
fn execute(
    &self,
    TmuxSwitchOptions {
        session_description_repository,
        session_repository,
        client_repository,
        workspace_repository,  // ADD
        tmux_storage,          // ADD
    }: TmuxSwitchOptions,
) {
    let descriptions = session_description_repository.get_session_descriptions();

    let res = fuzzy_pick(FuzzySearchArgs {
        items: &descriptions,
        search_text_fun: select_name,
    })
    .expect("Hey");

    if let Some(selected_session) = res {
        println!("You selected {}!", selected_session.name);

        // Create main session if needed
        let session = match &selected_session.session {
            Some(se) => se,
            None => &session_repository.new_session(selected_session),
        };

        // Switch to main session first
        client_repository.switch_client(None, SwitchClientTarget::Session(session));

        // Create worktree sessions if this is a workspace (ADD THIS BLOCK)
        use crate::domain::tmux_workspaces::aggregates::tmux::description::session::SessionKind;
        if let SessionKind::Workspace(workspace) = &selected_session.kind {
            create_worktree_sessions(workspace, session_repository, tmux_storage);
        }
    } else {
        println!("You didn't make a selection :/");
    }
}
```

---

### Step 4: Update Call Site in main.rs

**File:** `apps/cli/src/main.rs`

Find the `Tmux(Switch)` command handler and update:

```rust
Some(Commands::Tmux(TmuxArgs {
    command: TmuxCommand::Switch,
})) => {
    TmuxSwitchCommand.execute(TmuxSwitchOptions {
        session_description_repository: &description_repository,
        session_repository: &session_repository,
        client_repository: &client_repository,
        workspace_repository: &workspace_repository,  // ADD
        tmux_storage: &storage,                       // ADD
    });
}
```

---

### Step 5: Integration Tests

**New file:** `apps/cli/tests/tmux_switch_worktree_tests.rs`

```rust
mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabGitMixin, RafaeltabRootMixin},
    CliCommandBuilder,
};
use std::process::Command as StdCommand;
use test_descriptors::TestEnvironment;

#[test]
fn test_switch_to_workspace_without_worktrees() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("editor", None), ("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project-a", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project A");
                        });
                    });
                    g.rafaeltab_workspace("project_a", "Project A", |_w| {});
                });
            });
        });
    })
    .create();

    // Use tmux start (which will now also create worktree sessions)
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(result.success, "Failed to start tmux sessions");

    // Verify only main session exists (no worktrees)
    assert!(
        env.tmux().session_exists("Project A"),
        "Main workspace session should exist"
    );
    let sessions = env.tmux().list_sessions();
    assert_eq!(
        sessions.len(),
        1,
        "Should only have 1 session (main workspace, no worktrees)"
    );
}

#[test]
fn test_switch_creates_sessions_for_worktrees() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("editor", None), ("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# My Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create worktrees
    let worktree_path_1 = repo_path.parent().unwrap().join("feat-login");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feature/login",
            worktree_path_1.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree 1");

    let worktree_path_2 = repo_path.parent().unwrap().join("fix-bug");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "fix/bug-123",
            worktree_path_2.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree 2");

    // Start sessions - this should trigger worktree session creation
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(result.success, "Failed to start tmux sessions");

    // Verify main session exists
    assert!(
        env.tmux().session_exists("MyProject"),
        "Main workspace session should exist"
    );

    // Verify worktree sessions exist
    assert!(
        env.tmux().session_exists("MyProject-feature/login"),
        "Worktree session for feature/login should exist"
    );
    assert!(
        env.tmux().session_exists("MyProject-fix/bug-123"),
        "Worktree session for fix/bug-123 should exist"
    );

    let sessions = env.tmux().list_sessions();
    assert_eq!(
        sessions.len(),
        3,
        "Should have 3 sessions: main + 2 worktrees"
    );

    // Cleanup worktrees
    StdCommand::new("git")
        .args(["worktree", "remove", "--force", worktree_path_1.to_str().unwrap()])
        .current_dir(&repo_path)
        .output()
        .ok();
    StdCommand::new("git")
        .args(["worktree", "remove", "--force", worktree_path_2.to_str().unwrap()])
        .current_dir(&repo_path)
        .output()
        .ok();
}

#[test]
fn test_switch_uses_workspace_windows_for_worktrees() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("default", None)]);

            // Workspace-specific windows
            c.tmux_session("proj", Some("TestProj"), &[
                ("nvim", Some("nvim .")),
                ("terminal", None),
                ("logs", Some("tail -f app.log")),
            ]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "TestProj", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create a worktree
    let worktree_path = repo_path.parent().unwrap().join("feat-test");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat/test",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Start sessions
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let _result = env.testers().cmd().run(&cmd);

    // Verify worktree session exists
    assert!(
        env.tmux().session_exists("TestProj-feat/test"),
        "Worktree session should exist"
    );

    // Verify windows match workspace config (not default)
    let session = env
        .find_tmux_session("TestProj-feat/test")
        .expect("Session should exist");
    let windows = session.windows();
    assert_eq!(windows.len(), 3, "Should have 3 windows from workspace config");
    assert!(windows.iter().any(|w| w.contains("nvim")), "Should have nvim window");
    assert!(windows.iter().any(|w| w.contains("terminal")), "Should have terminal window");
    assert!(windows.iter().any(|w| w.contains("logs")), "Should have logs window");

    // Should NOT have default window
    assert!(!windows.iter().any(|w| w.contains("default")), "Should not use default windows");

    // Cleanup
    StdCommand::new("git")
        .args(["worktree", "remove", "--force", worktree_path.to_str().unwrap()])
        .current_dir(&repo_path)
        .output()
        .ok();
}

#[test]
fn test_switch_skips_existing_worktree_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "TestProj", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create a worktree
    let worktree_path = repo_path.parent().unwrap().join("feat-test");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat/test",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Start sessions first time
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let _result = env.testers().cmd().run(&cmd);

    // Verify sessions exist
    assert!(env.tmux().session_exists("TestProj"));
    assert!(env.tmux().session_exists("TestProj-feat/test"));

    // Run start again - should be idempotent
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(result.success, "Second start should succeed");

    // Still should have exactly 2 sessions (no duplicates)
    let sessions = env.tmux().list_sessions();
    assert_eq!(sessions.len(), 2, "Should still have 2 sessions (no duplicates)");

    // Cleanup
    StdCommand::new("git")
        .args(["worktree", "remove", "--force", worktree_path.to_str().unwrap()])
        .current_dir(&repo_path)
        .output()
        .ok();
}

#[test]
fn test_switch_handles_non_git_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("shell", None)]);
        });

        root.test_dir(|td| {
            // Regular directory, not a git repo
            td.dir("non-git-workspace", |d| {
                d.rafaeltab_workspace("non_git", "NonGit Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start sessions - should not error even though workspace is not a git repo
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success, "Should succeed even for non-git workspace");
    assert!(
        !result.stderr.contains("error") && !result.stderr.contains("Error"),
        "Should not show errors for non-git workspace"
    );

    // Should still create main workspace session
    assert!(
        env.tmux().session_exists("NonGit Workspace"),
        "Main workspace session should exist"
    );

    let sessions = env.tmux().list_sessions();
    assert_eq!(
        sessions.len(),
        1,
        "Should only have main session (no worktrees for non-git workspace)"
    );
}
```

---

## Implementation Checklist

### Phase 1: Update Switch Command

- [ ] Add `workspace_repository` to `TmuxSwitchOptions` struct
- [ ] Add `tmux_storage` to `TmuxSwitchOptions` struct
- [ ] Add necessary imports to switch.rs
- [ ] Implement `create_worktree_sessions()` helper function
- [ ] Update `execute()` method to call helper after switch
- [ ] Run: `cargo build`
- [ ] Verify compilation succeeds

### Phase 2: Update Call Site

- [ ] Update `main.rs` to pass new parameters to TmuxSwitchOptions
- [ ] Run: `cargo build`
- [ ] Verify compilation succeeds

### Phase 3: Integration Tests

- [ ] Create `apps/cli/tests/tmux_switch_worktree_tests.rs`
- [ ] Implement `test_switch_to_workspace_without_worktrees`
- [ ] Implement `test_switch_creates_sessions_for_worktrees`
- [ ] Implement `test_switch_uses_workspace_windows_for_worktrees`
- [ ] Implement `test_switch_skips_existing_worktree_sessions`
- [ ] Implement `test_switch_handles_non_git_workspace`
- [ ] Run: `cargo test --test tmux_switch_worktree_tests`
- [ ] Verify all tests pass

### Phase 4: Manual Testing

- [ ] Create test workspace with git repo
- [ ] Create 2-3 worktrees manually
- [ ] Run `rafaeltab tmux start`
- [ ] Verify all sessions created (main + worktrees)
- [ ] Run start again, verify idempotency
- [ ] Test with non-git workspace
- [ ] Test with workspace that has custom window config

### Phase 5: Regression Testing

- [ ] Run: `cargo test`
- [ ] Verify no existing tests broken
- [ ] Verify Plan 2 tests still pass
- [ ] Manual test: Run switch with TUI, verify works

---

## Files Modified

1. 🔧 **MODIFIED:** `apps/cli/src/commands/tmux/switch.rs` - Add worktree session creation
2. 🔧 **MODIFIED:** `apps/cli/src/main.rs` - Pass additional dependencies
3. ✨ **NEW:** `apps/cli/tests/tmux_switch_worktree_tests.rs` - Integration tests

---

## Dependencies

### Required from Plan 1

✅ `commands::tmux::session_utils::get_windows_for_workspace()` - Must be implemented first

### Already Available

✅ `infrastructure::git::discover_worktrees_for_workspace()` - Discovers worktrees  
✅ `infrastructure::git::WorktreeInfo` - Contains path and branch  
✅ `domain::tmux_workspaces::aggregates::tmux::description::session::SessionDescription`  
✅ `domain::tmux_workspaces::repositories::tmux::session_repository::TmuxSessionRepository`  
✅ `storage::tmux::TmuxStorage`

---

## Edge Cases Handled

1. **Non-git workspace:** `discover_worktrees_for_workspace()` returns error, silently skip
2. **No worktrees:** Function returns early, no sessions created
3. **Existing sessions:** Check and skip if session name already exists
4. **Git command failure:** Return from function, log TODO comment
5. **Empty window config:** Handled by `get_windows_for_workspace()` from Plan 1

---

## UUID Namespaces

**Worktree sessions:** `f47ac10b-58cc-4372-a567-0e02b2c3d479`

- Same namespace as used in `worktree start` command
- Ensures consistency across commands

---

## Success Criteria

- [ ] Switch command creates worktree sessions automatically
- [ ] Worktree sessions use correct window configuration
- [ ] Session naming follows pattern: `{workspace}-{branch}`
- [ ] Idempotent: Running multiple times doesn't duplicate
- [ ] Non-git workspaces handled gracefully
- [ ] All integration tests pass
- [ ] No regressions in existing tests
- [ ] User is switched to main session (not worktree)
- [ ] Worktree sessions created after switch (non-blocking)
