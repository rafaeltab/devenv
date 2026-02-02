# Plan 1: Fix Worktree Start Window Configuration

## Problem Statement

**Current Behavior:** `worktree start` command hardcodes window configuration:

- Window 1: "neovim" with command "nvim ."
- Window 2: "shell" with no command

**Location:** `apps/cli/src/commands/worktree/start.rs` lines 392-401

**Expected Behavior:** Should use workspace's tmux session config if it exists, otherwise fall back to default windows (same behavior as regular workspace sessions created by `tmux start`).

---

## Current Code Analysis

### Where Windows Are Hardcoded

**File:** `apps/cli/src/commands/worktree/start.rs`

```rust
// Lines 392-401
windows: vec![
    WindowDescription {
        name: "neovim".to_string(),
        command: Some("nvim .".to_string()),
    },
    WindowDescription {
        name: "shell".to_string(),
        command: None,
    },
],
```

### How Normal Workspace Sessions Get Windows

**File:** `apps/cli/src/infrastructure/tmux_workspaces/repositories/tmux/description_repository.rs`

**Lines 44-53:** Reads default windows from config

```rust
let default_window_descriptions: Vec<WindowDescription> = self
    .tmux_storage
    .read()
    .default_windows
    .iter()
    .map(|x| WindowDescription {
        name: x.name.clone(),
        command: x.command.clone(),
    })
    .collect();
```

**Lines 76-94:** Overrides with workspace-specific windows if configured

```rust
Session::Workspace(workspace) => {
    let windows: Vec<WindowDescription> = workspace
        .windows
        .iter()
        .map(|x| WindowDescription {
            name: x.name.clone(),
            command: x.command.clone(),
        })
        .collect();
    // Replace the windows for this workspace
    res_workspace.windows = windows;
}
```

---

## Test Cases

### TC-WS-01: Worktree session uses default windows when no workspace config

**Given:**

- A workspace with no custom tmux session config
- Global default windows configured: ["editor", "shell", "build"]
- A worktree is created for branch "feat/test"

**When:** `worktree start feat/test` is executed

**Then:**

- Worktree session is created with 3 windows:
  - Window 1: "editor" (from default config)
  - Window 2: "shell" (from default config)
  - Window 3: "build" (from default config)
- Windows match the default windows configuration

**Status:** ❌ Not implemented

---

### TC-WS-02: Worktree session uses workspace-specific windows when configured

**Given:**

- A workspace "my-project" with custom tmux session config
- Workspace windows configured: ["nvim", "terminal", "server"]
- A worktree is created for branch "feat/api"

**When:** `worktree start feat/api` is executed

**Then:**

- Worktree session is created with 3 windows:
  - Window 1: "nvim" (from workspace config)
  - Window 2: "terminal" (from workspace config)
  - Window 3: "server" (from workspace config)
- Windows match the workspace-specific configuration

**Status:** ❌ Not implemented

---

### TC-WS-03: Worktree session uses window commands from config

**Given:**

- Default windows configured with commands:
  - "editor" → "nvim ."
  - "shell" → None
  - "logs" → "tail -f logs/app.log"
- A worktree is created

**When:** `worktree start feat/logs` is executed

**Then:**

- Window "editor" runs "nvim ." command
- Window "shell" has no command (just shell)
- Window "logs" runs "tail -f logs/app.log" command

**Status:** ❌ Not implemented

---

### TC-WS-04: Worktree session handles empty default windows

**Given:**

- No default windows configured (empty array)
- A worktree is created

**When:** `worktree start feat/test` is executed

**Then:**

- Session is created with 0 windows
- Or falls back to a single default window (implementation decision)

**Status:** ❌ Not implemented (need to decide fallback behavior)

---

## Implementation Plan

### Step 1: Create Shared Window Configuration Helper

**New file:** `apps/cli/src/commands/tmux/session_utils.rs`

```rust
use crate::{
    domain::tmux_workspaces::aggregates::tmux::description::window::WindowDescription,
    storage::tmux::{Session, TmuxStorage},
};

/// Get window configuration for a workspace session.
/// Returns workspace-specific windows if configured, otherwise returns default windows.
pub fn get_windows_for_workspace(
    workspace_id: &str,
    tmux_storage: &dyn TmuxStorage,
) -> Vec<WindowDescription> {
    let tmux_config = tmux_storage.read();

    // Check if workspace has custom session config
    if let Some(sessions) = &tmux_config.sessions {
        for session in sessions {
            if let Session::Workspace(ws_session) = session {
                if ws_session.workspace == workspace_id {
                    return ws_session.windows.iter().map(|w| WindowDescription {
                        name: w.name.clone(),
                        command: w.command.clone(),
                    }).collect();
                }
            }
        }
    }

    // Fall back to default windows
    tmux_config.default_windows.iter().map(|w| WindowDescription {
        name: w.name.clone(),
        command: w.command.clone(),
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{
        test::mocks::MockTmuxStorage,
        tmux::{Session, Tmux, Window, WorkspaceSession},
    };

    #[test]
    fn test_returns_default_windows_when_no_workspace_config() {
        let storage = MockTmuxStorage {
            data: Tmux {
                sessions: None,
                default_windows: vec![
                    Window {
                        name: "editor".to_string(),
                        command: Some("vim".to_string()),
                    },
                    Window {
                        name: "shell".to_string(),
                        command: None,
                    },
                ],
            },
        };

        let result = get_windows_for_workspace("test-workspace", &storage);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "editor");
        assert!(result[0].command.as_ref().map_or(false, |c| c.contains("vim")));
        assert_eq!(result[1].name, "shell");
        assert_eq!(result[1].command, None);
    }

    #[test]
    fn test_returns_workspace_config_when_exists() {
        let storage = MockTmuxStorage {
            data: Tmux {
                sessions: Some(vec![Session::Workspace(WorkspaceSession {
                    workspace: "my-workspace".to_string(),
                    name: None,
                    windows: vec![
                        Window {
                            name: "nvim".to_string(),
                            command: Some("nvim .".to_string()),
                        },
                        Window {
                            name: "build".to_string(),
                            command: Some("npm run dev".to_string()),
                        },
                    ],
                })]),
                default_windows: vec![Window {
                    name: "default".to_string(),
                    command: None,
                }],
            },
        };

        let result = get_windows_for_workspace("my-workspace", &storage);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "nvim");
        assert!(result[0].command.as_ref().map_or(false, |c| c.contains("nvim .")));
        assert_eq!(result[1].name, "build");
        assert!(result[1].command.as_ref().map_or(false, |c| c.contains("npm run dev")));
    }

    #[test]
    fn test_returns_default_for_different_workspace() {
        let storage = MockTmuxStorage {
            data: Tmux {
                sessions: Some(vec![Session::Workspace(WorkspaceSession {
                    workspace: "workspace-a".to_string(),
                    name: None,
                    windows: vec![Window {
                        name: "custom".to_string(),
                        command: None,
                    }],
                })]),
                default_windows: vec![Window {
                    name: "default".to_string(),
                    command: None,
                }],
            },
        };

        let result = get_windows_for_workspace("workspace-b", &storage);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "default");
    }

    #[test]
    fn test_handles_empty_default_windows() {
        let storage = MockTmuxStorage {
            data: Tmux {
                sessions: None,
                default_windows: vec![],
            },
        };

        let result = get_windows_for_workspace("test-workspace", &storage);

        assert_eq!(result.len(), 0);
    }
}
```

---

### Step 2: Export Module

**File:** `apps/cli/src/commands/tmux/mod.rs`

Add:

```rust
pub mod session_utils;
```

---

### Step 3: Update WorktreeStartOptions

**File:** `apps/cli/src/commands/worktree/start.rs`

**Add to struct** (around line 37):

```rust
pub struct WorktreeStartOptions<'a> {
    /// The branch name for the new worktree
    pub branch_name: String,
    /// Force creation even without worktree config
    pub force: bool,
    /// Skip confirmation prompt
    pub yes: bool,
    /// Repository for workspace operations
    pub workspace_repository: &'a dyn WorkspaceRepository,
    /// Storage for global worktree config
    pub worktree_storage: &'a dyn WorktreeStorage,
    /// Repository for tmux session operations
    pub session_repository: &'a dyn TmuxSessionRepository,
    /// Repository for tmux client operations
    pub client_repository: &'a dyn TmuxClientRepository,
    /// Storage for tmux configuration  // ADD THIS
    pub tmux_storage: &'a dyn TmuxStorage,  // ADD THIS
}
```

---

### Step 4: Update create_tmux_session Function

**File:** `apps/cli/src/commands/worktree/start.rs`

**Change function signature** (around line 372):

```rust
fn create_tmux_session(
    session_repository: &dyn TmuxSessionRepository,
    session_name: &str,
    worktree_path: &Path,
    workspace_id: &str,        // ADD THIS
    tmux_storage: &dyn TmuxStorage,  // ADD THIS
) -> Option<TmuxSession> {
```

**Update function body** (around line 386):

```rust
use crate::commands::tmux::session_utils::get_windows_for_workspace;  // ADD THIS

let description = SessionDescription {
    id: id.to_string(),
    name: session_name.to_string(),
    kind: SessionKind::Path(PathSessionDescription {
        path: worktree_path.to_string_lossy().to_string(),
    }),
    windows: get_windows_for_workspace(workspace_id, tmux_storage),  // CHANGE THIS LINE
    session: None,
};
```

**Update function call** (around line 310):

```rust
let session = create_tmux_session(
    options.session_repository,
    &session_name,
    &worktree_path,
    &workspace.id,          // ADD THIS
    options.tmux_storage,   // ADD THIS
);
```

---

### Step 5: Update Call Site in main.rs

**File:** `apps/cli/src/main.rs`

Find the `Worktree(Start)` command handler and add:

```rust
Commands::Worktree(WorktreeArgs {
    command: WorktreeCommand::Start(start_args),
}) => {
    WorktreeStartCommand.execute(WorktreeStartOptions {
        branch_name: start_args.branch_name,
        force: start_args.force,
        yes: start_args.yes,
        workspace_repository: &workspace_repository,
        worktree_storage: &storage,
        session_repository: &session_repository,
        client_repository: &client_repository,
        tmux_storage: &storage,  // ADD THIS LINE
    });
}
```

---

### Step 6: Integration Tests

**New file:** `apps/cli/tests/worktree_start_windows_tests.rs`

```rust
mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabGitMixin, RafaeltabRootMixin},
    run_cli_with_tmux,
};
use std::process::Command;
use test_descriptors::TestEnvironment;

#[test]
fn test_worktree_start_uses_default_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // Set custom default windows
            c.default_windows(&[
                ("editor", Some("vim")),
                ("shell", None),
                ("build", Some("npm run dev")),
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
                    g.rafaeltab_workspace("proj", "MyProject", |_w| {});
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let repo_path = env.root_path().join("project/repo");

    // Run worktree start
    let (stdout, stderr, success) = run_cli_with_tmux(
        &["worktree", "start", "feat/test", "--yes"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(success, "Command should succeed.\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);

    // Verify session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/test"),
        "Worktree session should exist"
    );

    // Verify windows (this requires checking tmux window list)
    let windows = env.tmux().list_windows("MyProject-feat/test");
    assert_eq!(windows.len(), 3, "Should have 3 windows from default config");
    assert!(windows.iter().any(|w| w.contains("editor")), "Should have editor window");
    assert!(windows.iter().any(|w| w.contains("shell")), "Should have shell window");
    assert!(windows.iter().any(|w| w.contains("build")), "Should have build window");

    // Cleanup
    Command::new("git")
        .args(["worktree", "remove", "--force", "../feat-test"])
        .current_dir(&repo_path)
        .output()
        .ok();
}

#[test]
fn test_worktree_start_uses_workspace_specific_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // Set default windows
            c.default_windows(&[("default", None)]);

            // Set workspace-specific session config
            c.tmux_session("proj", Some("MyProject"), &[
                ("nvim", Some("nvim .")),
                ("terminal", None),
                ("server", Some("npm start")),
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
                    g.rafaeltab_workspace("proj", "MyProject", |_w| {});
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let repo_path = env.root_path().join("project/repo");

    // Run worktree start
    let (stdout, stderr, success) = run_cli_with_tmux(
        &["worktree", "start", "feat/api", "--yes"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(success, "Command should succeed.\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);

    // Verify session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/api"),
        "Worktree session should exist"
    );

    // Verify windows match workspace config (not default)
    let windows = env.tmux().list_windows("MyProject-feat/api");
    assert_eq!(windows.len(), 3, "Should have 3 windows from workspace config");
    assert!(windows.iter().any(|w| w.contains("nvim")), "Should have nvim window");
    assert!(windows.iter().any(|w| w.contains("terminal")), "Should have terminal window");
    assert!(windows.iter().any(|w| w.contains("server")), "Should have server window");

    // Should NOT have default window
    assert!(!windows.iter().any(|w| w.contains("default")), "Should not have default window");

    // Cleanup
    Command::new("git")
        .args(["worktree", "remove", "--force", "../feat-api"])
        .current_dir(&repo_path)
        .output()
        .ok();
}

#[test]
fn test_worktree_start_handles_empty_default_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // No default windows configured
            c.default_windows(&[]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |_w| {});
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let repo_path = env.root_path().join("project/repo");

    // Run worktree start
    let (stdout, stderr, success) = run_cli_with_tmux(
        &["worktree", "start", "feat/empty", "--yes"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(success, "Command should succeed even with empty windows.\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);

    // Verify session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/empty"),
        "Worktree session should exist"
    );

    // Verify no windows or one default window (implementation dependent)
    let windows = env.tmux().list_windows("MyProject-feat/empty");
    // Accept either 0 or 1 window depending on tmux behavior
    assert!(windows.len() <= 1, "Should have at most 1 window");

    // Cleanup
    Command::new("git")
        .args(["worktree", "remove", "--force", "../feat-empty"])
        .current_dir(&repo_path)
        .output()
        .ok();
}
```

---

## Implementation Checklist

### Phase 1: Create Helper Function

- [ ] Create `apps/cli/src/commands/tmux/session_utils.rs`
- [ ] Implement `get_windows_for_workspace()` function
- [ ] Add 4 unit tests for the helper function
- [ ] Add `pub mod session_utils;` to `apps/cli/src/commands/tmux/mod.rs`
- [ ] Run: `cargo test --lib session_utils`
- [ ] Verify all unit tests pass

### Phase 2: Update Worktree Start

- [ ] Update `WorktreeStartOptions` struct with `tmux_storage` field
- [ ] Update `create_tmux_session()` function signature
- [ ] Update `create_tmux_session()` to use `get_windows_for_workspace()`
- [ ] Update call to `create_tmux_session()` in execute method
- [ ] Update call site in `main.rs`
- [ ] Run: `cargo build`
- [ ] Verify compilation succeeds

### Phase 3: Integration Tests

- [ ] Create `apps/cli/tests/worktree_start_windows_tests.rs`
- [ ] Implement `test_worktree_start_uses_default_windows`
- [ ] Implement `test_worktree_start_uses_workspace_specific_windows`
- [ ] Implement `test_worktree_start_handles_empty_default_windows`
- [ ] Run: `cargo test --test worktree_start_windows_tests`
- [ ] Verify all tests pass

### Phase 4: Manual Testing

- [ ] Create a test workspace with custom tmux config
- [ ] Run `worktree start feat/test`
- [ ] Verify windows match config (not hardcoded)
- [ ] Test with default windows only
- [ ] Test with workspace-specific windows

### Phase 5: Regression Testing

- [ ] Run full test suite: `cargo test`
- [ ] Verify no existing tests broken
- [ ] Manual test all worktree commands still work

---

## Files Modified

1. ✨ **NEW:** `apps/cli/src/commands/tmux/session_utils.rs` - Helper function
2. 🔧 **MODIFIED:** `apps/cli/src/commands/tmux/mod.rs` - Export new module
3. 🔧 **MODIFIED:** `apps/cli/src/commands/worktree/start.rs` - Use helper function
4. 🔧 **MODIFIED:** `apps/cli/src/main.rs` - Pass tmux_storage
5. ✨ **NEW:** `apps/cli/tests/worktree_start_windows_tests.rs` - Integration tests

---

## Dependencies

### Required for Testing

- [ ] Verify `TestEnvironment::describe` supports `default_windows()` config
- [ ] Verify `env.tmux().list_windows(session_name)` is available
- [ ] If not available, need to add helper to get window list from tmux

### Required Traits/Types (Already Exist)

✅ `TmuxStorage::read()` - Read tmux config  
✅ `WindowDescription` - Window structure  
✅ `Session::Workspace` - Workspace session type  
✅ `storage::tmux::Window` - Config window type

---

## Success Criteria

- [ ] Unit tests pass for `get_windows_for_workspace()`
- [ ] Integration tests pass for worktree start
- [ ] Worktree sessions use config windows, not hardcoded
- [ ] Behavior matches regular workspace sessions
- [ ] No regressions in existing tests
- [ ] Code compiles without warnings
