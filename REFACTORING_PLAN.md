# Test Descriptors Refactoring Plan - Hybrid Approach

## Progress Summary

| Stage                                 | Status         | Tests            |
| ------------------------------------- | -------------- | ---------------- |
| Stage 1: Basic Hierarchical Structure | ✅ Complete    | 29 tests         |
| Stage 2: Complete Git Features        | ✅ Complete    | 36 tests         |
| Stage 3: Tmux Integration             | ✅ Complete    | 20 tests         |
| Stage 4: Query API                    | ✅ Complete    | (included above) |
| Stage 5: Integration Tests            | ✅ Complete    | 9 tests          |
| Stage 6: Rafaeltab Integration        | ✅ Complete    | 7 tests          |
| Stage 7: Cleanup Old API              | ❌ Not Started | -                |

**Total: 121 tests passing (114 test-descriptors + 7 rafaeltab-descriptors)**

---

## What's Been Implemented

### Hierarchical Builder API ✅

```rust
let env = TestEnvironment::describe(|root| {
    root.test_dir(|td| {
        td.dir("workspace", |d| {
            d.git("my-repo", |g| {
                g.remote("origin");
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Test");
                        c.pushed("origin");
                    });
                });
            });
            d.tmux_session("dev-session", |s| {
                s.window("editor");
                s.window("terminal");
            });
        });
    });
}).create();
```

### Git Worktrees ✅

```rust
td.dir("workspace", |d| {
    d.git("my-repo", |g| { /* ... */ });
    d.git_worktree("my-repo", "main", "feature/test", |w| {
        w.commit("Add feature", |c| {
            c.file("feature.txt", "content");
        });
    });
});

// Query
let worktree = env.find_worktree("my-repo", "feature/test").unwrap();
assert!(worktree.exists());
```

### Staged/Unstaged Changes ✅

```rust
d.git("my-repo", |g| {
    g.branch("main", |b| { /* commits */ });
    g.staged(|s| {
        s.file("staged.txt", "content");
        s.delete("old-file.txt");
    });
    g.unstaged(|u| {
        u.modify("README.md", "modified content");
        u.untracked("new-file.txt", "untracked");
        u.delete("another.txt");
    });
});

// Query
assert!(repo.has_staged_changes());
assert!(repo.has_unstaged_changes());
assert!(repo.has_untracked_files());
```

### Query API ✅

```rust
// Find resources by name
let repo = env.find_git_repo("my-repo").unwrap();
let dir = env.find_dir("workspace").unwrap();
let session = env.find_tmux_session("dev-session").unwrap();
let worktree = env.find_worktree("my-repo", "feature/test").unwrap();

// GitRepoRef methods
repo.path();
repo.current_branch();
repo.branches();
repo.is_clean();
repo.has_staged_changes();
repo.has_unstaged_changes();
repo.has_untracked_files();
repo.has_unpushed_commits();
repo.git(&["status"]);

// TmuxSessionRef methods
session.name();
session.working_dir();
session.exists();
session.windows();
session.has_window("editor");
session.run_shell("echo hello");
session.run_shell_args("git", &["status"]);

// WorktreeRef methods
worktree.path();
worktree.branch();
worktree.current_branch();
worktree.is_clean();
worktree.has_staged_changes();
worktree.commit_count();
```

### Shell Command Execution ✅

```rust
let session = env.find_tmux_session("dev-session").unwrap();

// Run commands in session's working directory
let output = session.run_shell("pwd && ls -la");
output.assert_success()
      .assert_stdout_contains("workspace");

// With separate args
let output = session.run_shell_args("git", &["branch", "--list"]);
assert!(output.success());

// ShellOutput helpers
output.success();
output.exit_code();
output.assert_success();
output.assert_failure();
output.assert_stdout_contains("text");
output.assert_stderr_contains("error");
output.assert_stdout_eq("exact match");
```

### Rafaeltab Integration ✅

CLI-specific descriptors in `apps/cli/tests/common/rafaeltab_descriptors/`:

```rust
use common::rafaeltab_descriptors::{RafaeltabRootMixin, RafaeltabGitMixin, RafaeltabDirMixin};

let env = TestEnvironment::describe(|root| {
    // Create rafaeltab config with defaults
    root.rafaeltab_config(|c| {
        c.defaults();
        c.worktree_global(&["pnpm install"], &["**/.env"]);
    });

    root.test_dir(|td| {
        td.dir("projects", |d| {
            // Register workspace inside a git repo
            d.git("my-repo", |g| {
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Test");
                    });
                });

                g.rafaeltab_workspace("my_project", "My Project", |w| {
                    w.tag("rust");
                    w.worktree(&["cargo build"], &[]);
                });
            });

            // Or register workspace directly on a directory
            d.rafaeltab_workspace("my_workspace", "My Workspace", |w| {
                w.tag("docs");
            });
        });
    });
}).create();

// Config is written to config.json in root
let config_path = env.context().config_path().unwrap();
```

---

## Files Created/Modified

### New Files (packages/test-descriptors)

- `src/builders/root.rs` - RootBuilder
- `src/builders/test_dir.rs` - TestDirBuilder
- `src/builders/dir.rs` - DirBuilder, DirDescriptor
- `src/builders/git.rs` - GitBuilder, BranchBuilder, CommitBuilder, HierarchicalGitRepoDescriptor
- `src/builders/tmux.rs` - SessionBuilder, HierarchicalTmuxSessionDescriptor
- `src/builders/worktree.rs` - WorktreeBuilder, HierarchicalWorktreeDescriptor
- `src/builders/changes.rs` - StagedBuilder, UnstagedBuilder, StagedChanges, UnstagedChanges
- `src/queries/dir_ref.rs` - DirRef
- `src/queries/git_repo_ref.rs` - GitRepoRef
- `src/queries/tmux_session_ref.rs` - TmuxSessionRef
- `src/queries/worktree_ref.rs` - WorktreeRef
- `src/queries/shell.rs` - ShellOutput
- `tests/worktree_tests.rs` - 13 tests
- `tests/changes_tests.rs` - 11 tests

### New Files (apps/cli/tests/common/rafaeltab_descriptors)

- `mod.rs` - Module exports
- `config.rs` - ConfigBuilder, ConfigDescriptor, RafaeltabRootMixin
- `workspace.rs` - WorkspaceBuilder, RafaeltabDirMixin, RafaeltabGitMixin

### Modified Files

- `src/environment.rs` - Added `describe()`, `find_*()` methods
- `src/descriptor/branch.rs` - Fixed branch creation when branch already exists
- `src/lib.rs` - Updated exports
- All test files migrated to new API

---

## What Still Needs To Be Done

### Stage 7: Cleanup Old API (1-2 hours)

Remove deprecated API:

- [ ] Remove `.with_*()` builder methods from descriptors
- [ ] Remove `env.add_descriptor()` method
- [ ] Remove old descriptor exports from `lib.rs` (keep only new builders)
- [ ] Update any remaining documentation

---

## Example: Full CLI Integration Test

Once Stage 6 is complete, a CLI integration test would look like:

```rust
use test_descriptors::TestEnvironment;
use rafaeltab_descriptors::{RafaeltabRootMixin, RafaeltabGitMixin};

#[test]
fn test_worktree_start_command() {
    let env = TestEnvironment::describe(|root| {
        // Create rafaeltab config
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("projects", |d| {
                // Create git repo with remote and register workspace inside it
                d.git("my-repo", |g| {
                    g.remote("origin");
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("Cargo.toml", "[package]\nname = \"test\"");
                            c.pushed("origin");
                        });
                    });

                    // Register workspace at the git repo's path
                    g.rafaeltab_workspace("my_project", "My Project", |w| {
                        w.tag("rust");
                        w.worktree(&["cargo build"], &[]);
                    });
                });

                // Create tmux session
                d.tmux_session("dev", |s| {
                    s.window("shell");
                });
            });
        });
    }).create();

    // Run CLI command
    let session = env.find_tmux_session("dev").unwrap();
    session
        .run_shell("cd my-repo && rafaeltab worktree start feature/test --yes")
        .assert_success();

    // Verify worktree was created
    let worktree = env.find_worktree("my-repo", "feature/test");
    assert!(worktree.is_some());
    assert!(worktree.unwrap().exists());
}
```

---

## Success Criteria

- [x] All existing tests pass with new API (114 tests)
- [x] Hierarchical nesting works correctly
- [x] Automatic path resolution from parent
- [x] Worktree descriptor implemented
- [x] Staged/unstaged changes support
- [x] Query API with typed refs
- [x] Shell command execution
- [x] Rafaeltab descriptors work in CLI package (7 tests)
- [ ] Old API completely removed
- [ ] Clean, intuitive API matching original design

---

## Commits Made

1. `feat(test-descriptors): add hierarchical builder API for directories`
2. `feat(test-descriptors): add Git support to hierarchical builder API`
3. `feat(test-descriptors): add tmux support to hierarchical builder API`
4. `feat(test-descriptors): add git worktree support to hierarchical API`
5. `feat(test-descriptors): add staged/unstaged changes support to builder API`
6. `feat(test-descriptors): add shell command execution to tmux sessions`
7. `feat(cli): add rafaeltab descriptor mixin traits for config and workspace`
