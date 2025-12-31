# Test Descriptors Refactoring Plan - Hybrid Approach

## Current State Analysis

The test-descriptors package was implemented with **83 passing tests** but deviates significantly from the original design.

### What Was Implemented vs. What We Want

## Current Implementation (What Was Built)

```rust
use test_descriptors::*;

#[test]
fn test_worktree_complete() {
    let mut env = TestEnvironment::new();

    // Create git repo
    let repo = GitRepoDescriptor::new("repo-0")
        .with_branch(
            BranchDescriptor::new("main")
                .with_commit(
                    CommitDescriptor::new("Initial")
                        .with_file("README.md", "# Test")
                )
        )
        .with_remote(RemoteDescriptor::new("origin"));

    // Create tmux session
    let session = TmuxSessionDescriptor::new("session-0")
        .with_window(WindowDescriptor::new("zsh"));

    // Create directory
    let dir = DirectoryDescriptor::new("main");

    env.add_descriptor(dir);
    env.add_descriptor(repo);
    env.add_descriptor(session);
    env.create().unwrap();

    // No query API - must use paths directly
    assert!(env.root_path().join("main").exists());
    assert!(env.root_path().join("repo-0/.git").exists());
    assert!(env.tmux().session_exists("session-0"));
}
```

**Issues:**

- ❌ Flat structure - git repo not "inside" directory
- ❌ No automatic path resolution
- ❌ Manual descriptor addition to env
- ❌ No query API
- ❌ Session working dir not tied to directory

## Desired Design (What We Want)

```rust
use test_descriptors::TestEnvironment;

#[test]
fn test_worktree_complete() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("main", |d| {
                // Git repo IS INSIDE the "main" directory
                // Automatically gets path: root/main/repo-0
                d.git("repo-0", |g| {
                    g.remote("origin");
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Test");
                            c.pushed("origin");
                        });
                    });
                });

                // Tmux session IS INSIDE the "main" directory
                // Automatically uses root/main as working_dir
                d.tmux_session("session-0", |s| {
                    s.window("zsh");
                });
            });
        });
    }).create();

    // Query API - find by name
    assert!(env.find_dir("main").is_some());
    assert!(env.find_git_repo("repo-0").unwrap().path().ends_with("main/repo-0"));
    assert!(env.find_tmux_session("session-0").unwrap().working_dir().ends_with("main"));
}
```

**Benefits:**

- ✅ Hierarchical structure (git inside dir)
- ✅ Automatic path resolution from parent
- ✅ Closure-based clean syntax
- ✅ Query API for assertions
- ✅ Parent-child relationships

## With Rafaeltab Integration (CLI Package)

```rust
use test_descriptors::TestEnvironment;
use rafaeltab_descriptors::RafaeltabDescriptorMixin; // Trait in CLI package

#[test]
fn test_worktree_start() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("main", |d| {
                // Workspace auto-registers in config
                // Uses parent dir as root path
                d.rafaeltab_workspace("test_ws", "Test Workspace", |w| {
                    w.tag("test");
                    w.worktree(&["pnpm install"], &["**/.env"]);
                });

                d.git("repo-0", |g| {
                    g.remote("origin");
                    g.branch("main", |b| {
                        b.commit("Init", |c| {
                            c.file("README.md", "# Test");
                            c.pushed("origin");
                        });
                    });
                });

                d.tmux_session("session-0", |s| {
                    s.window("zsh");
                });
            });
        });
    }).create();

    // Run CLI command in session context
    env.find_tmux_session("session-0")
        .unwrap()
        .run_cli(&["worktree", "start", "feature/test", "--yes"])
        .assert_success();

    // Verify worktree was created
    assert!(env.find_worktree("repo-0", "feature/test").is_some());
}
```

## Key Differences Summary

| Aspect                    | Current Implementation                  | Desired Design                                       |
| ------------------------- | --------------------------------------- | ---------------------------------------------------- |
| **Structure**             | Flat - descriptors added to environment | Hierarchical - descriptors nested in parent contexts |
| **Builder API**           | Method chaining (`.with_x()`)           | Closure-based (`\|builder\| { builder.x(); }`)       |
| **Path Resolution**       | Manual - path is the name               | Automatic - path from parent directory               |
| **Extensibility**         | Direct implementation                   | Mixin traits allow extension                         |
| **Parent-Child**          | No relationship                         | Strong parent-child relationships                    |
| **Context**               | Single global context                   | Scoped builders with parent context                  |
| **Rafaeltab Integration** | Not present                             | Workspace/config descriptors in CLI package          |
| **Query API**             | None - direct path access               | `find_*()` methods with typed refs                   |

---

## Strategy: Hybrid Approach with TDD Migration

**Keep the working descriptor core** (GitRepoDescriptor, TmuxSessionDescriptor, etc.) but add a hierarchical builder layer on top.

### Implementation Approach

1. **Start with TDD Migration**: Rewrite existing tests using the new API first (they'll fail)
2. **Implement Builders**: Add hierarchical builders to make tests pass
3. **Add Missing Features**: Worktree descriptor, staged/unstaged changes, query API
4. **Remove Old API**: Once all tests pass with new API, remove the old methods

### Benefits

✅ **Low Risk**: Core descriptor logic is already tested and works  
✅ **Incremental**: Can implement feature by feature  
✅ **TDD Driven**: Tests guide implementation  
✅ **Clean Result**: Only new API remains at the end

---

## Implementation Plan

### Phase 1: Hierarchical Builders (Core API)

#### Step 1.1: Create Builder Module Structure

```
src/
  builders/
    mod.rs
    root.rs          - RootBuilder (entry point)
    test_dir.rs      - TestDirBuilder (container for dirs)
    dir.rs           - DirBuilder (hierarchical container)
    git.rs           - GitBuilder (wraps GitRepoDescriptor)
    branch.rs        - BranchBuilder (wraps BranchDescriptor)
    commit.rs        - CommitBuilder (wraps CommitDescriptor)
    tmux.rs          - SessionBuilder (wraps TmuxSessionDescriptor)
    mixins.rs        - Mixin traits
```

#### Step 1.2: Define Core Builders

```rust
// builders/root.rs
pub struct RootBuilder<'a> {
    env: &'a mut TestEnvironment,
}

impl<'a> RootBuilder<'a> {
    pub fn test_dir<F>(&mut self, f: F) where F: FnOnce(&mut TestDirBuilder);
}

// builders/test_dir.rs
pub struct TestDirBuilder {
    parent_path: PathBuf,
    descriptors: Vec<Box<dyn Descriptor>>,
}

impl TestDirBuilder {
    pub fn dir<F>(&mut self, name: &str, f: F) where F: FnOnce(&mut DirBuilder);
}

// builders/dir.rs
pub struct DirBuilder {
    name: String,
    parent_path: PathBuf,
    children: Vec<Box<dyn Descriptor>>,
}

impl DirBuilder {
    pub fn git<F>(&mut self, name: &str, f: F) where F: FnOnce(&mut GitBuilder);
    pub fn tmux_session<F>(&mut self, name: &str, f: F) where F: FnOnce(&mut SessionBuilder);
    pub fn dir<F>(&mut self, name: &str, f: F) where F: FnOnce(&mut DirBuilder);
}
```

#### Step 1.3: Implement Mixin Traits

```rust
// builders/mixins.rs
pub trait GitDescriptorMixin {
    fn git<F>(&mut self, name: &str, f: F) where F: FnOnce(&mut GitBuilder);
}

pub trait TmuxDescriptorMixin {
    fn tmux_session<F>(&mut self, name: &str, f: F) where F: FnOnce(&mut SessionBuilder);
}

pub trait DirDescriptorMixin {
    fn dir<F>(&mut self, name: &str, f: F) where F: FnOnce(&mut DirBuilder);
}

// DirBuilder implements all mixins
impl GitDescriptorMixin for DirBuilder { ... }
impl TmuxDescriptorMixin for DirBuilder { ... }
impl DirDescriptorMixin for DirBuilder { ... }
```

#### Step 1.4: Add TestEnvironment::describe()

```rust
// environment.rs
impl TestEnvironment {
    pub fn describe<F>(f: F) -> Self
    where F: FnOnce(&mut RootBuilder)
    {
        let mut env = TestEnvironment::new();
        {
            let mut root = RootBuilder::new(&mut env);
            f(&mut root);
        }
        env
    }
}
```

### Phase 2: Git Worktree Descriptor

#### Step 2.1: Create Worktree Descriptor

```rust
// descriptor/worktree.rs
pub struct GitWorktreeDescriptor {
    name: String,
    repo_name: String,
    base_branch: String,
    branch: String,
    commits: Vec<CommitDescriptor>,
    staged: Option<Vec<FileChange>>,
    unstaged: Option<Vec<FileChange>>,
}
```

#### Step 2.2: Add Worktree Builder

```rust
// builders/worktree.rs
pub struct WorktreeBuilder {
    worktree: GitWorktreeDescriptor,
}

impl WorktreeBuilder {
    pub fn commit<F>(&mut self, message: &str, f: F) where F: FnOnce(&mut CommitBuilder);
    pub fn staged<F>(&mut self, f: F) where F: FnOnce(&mut StagedBuilder);
    pub fn unstaged<F>(&mut self, f: F) where F: FnOnce(&mut UnstagedBuilder);
}
```

#### Step 2.3: Add to DirBuilder

```rust
impl DirBuilder {
    pub fn git_worktree<F>(&mut self, repo_name: &str, base_branch: &str, branch: &str, f: F)
    where F: FnOnce(&mut WorktreeBuilder);
}
```

### Phase 3: Staged/Unstaged Changes

#### Step 3.1: Create Change Descriptors

```rust
// descriptor/changes.rs
pub struct StagedChanges {
    changes: Vec<FileChange>,
}

pub struct UnstagedChanges {
    changes: Vec<FileChange>,
    untracked: Vec<FileChange>,
}
```

#### Step 3.2: Add Change Builders

```rust
// builders/changes.rs
pub struct StagedBuilder {
    changes: Vec<FileChange>,
}

impl StagedBuilder {
    pub fn file(&mut self, path: &str, content: &str);
    pub fn delete(&mut self, path: &str);
}

pub struct UnstagedBuilder {
    changes: Vec<FileChange>,
}

impl UnstagedBuilder {
    pub fn modify(&mut self, path: &str, content: &str);
    pub fn untracked(&mut self, path: &str, content: &str);
    pub fn delete(&mut self, path: &str);
}
```

#### Step 3.3: Integrate with GitBuilder

```rust
impl GitBuilder {
    pub fn staged<F>(&mut self, f: F) where F: FnOnce(&mut StagedBuilder);
    pub fn unstaged<F>(&mut self, f: F) where F: FnOnce(&mut UnstagedBuilder);
}
```

### Phase 4: Query API

#### Step 4.1: Create Query Module Structure

```
src/
  queries/
    mod.rs
    git_repo_ref.rs
    worktree_ref.rs
    tmux_session_ref.rs
    dir_ref.rs
    cli.rs
```

#### Step 4.2: Implement Reference Types

```rust
// queries/git_repo_ref.rs
pub struct GitRepoRef<'a> {
    name: String,
    path: PathBuf,
    env: &'a TestEnvironment,
}

impl<'a> GitRepoRef<'a> {
    pub fn path(&self) -> &Path;
    pub fn current_branch(&self) -> String;
    pub fn branches(&self) -> Vec<String>;
    pub fn is_clean(&self) -> bool;
    pub fn has_unpushed_commits(&self) -> bool;
    pub fn has_staged_changes(&self) -> bool;
    pub fn has_unstaged_changes(&self) -> bool;
    pub fn git(&self, args: &[&str]) -> String;
}

// queries/tmux_session_ref.rs
pub struct TmuxSessionRef<'a> {
    name: String,
    working_dir: PathBuf,
    env: &'a TestEnvironment,
}

impl<'a> TmuxSessionRef<'a> {
    pub fn name(&self) -> &str;
    pub fn exists(&self) -> bool;
    pub fn working_dir(&self) -> &Path;
    pub fn windows(&self) -> Vec<String>;
    pub fn run_cli(&self, args: &[&str]) -> CliOutput;
}

// queries/cli.rs
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

impl CliOutput {
    pub fn success(&self) -> bool;
    pub fn assert_success(&self) -> &Self;
    pub fn assert_failure(&self) -> &Self;
    pub fn assert_stdout_contains(&self, expected: &str) -> &Self;
}
```

#### Step 4.3: Add Find Methods to TestEnvironment

```rust
impl TestEnvironment {
    pub fn find_git_repo(&self, name: &str) -> Option<GitRepoRef>;
    pub fn find_worktree(&self, repo_name: &str, branch: &str) -> Option<WorktreeRef>;
    pub fn find_tmux_session(&self, name: &str) -> Option<TmuxSessionRef>;
    pub fn find_dir(&self, name: &str) -> Option<DirRef>;
}
```

### Phase 5: Rafaeltab Descriptors (CLI Package)

#### Step 5.1: Create Rafaeltab Module in CLI

```
apps/cli/tests/common/
  rafaeltab_descriptors/
    mod.rs
    config.rs
    workspace.rs
```

#### Step 5.2: Implement Descriptors

```rust
// apps/cli/tests/common/rafaeltab_descriptors/config.rs
pub struct ConfigDescriptor {
    workspaces: Vec<WorkspaceConfig>,
    global_worktree: Option<GlobalWorktreeConfig>,
}

pub struct ConfigBuilder {
    config: ConfigDescriptor,
}

impl ConfigBuilder {
    pub fn defaults(&mut self);
    pub fn worktree_global(&mut self, on_create: &[&str], symlink_files: &[&str]);
}

// apps/cli/tests/common/rafaeltab_descriptors/workspace.rs
pub struct WorkspaceDescriptor {
    id: String,
    name: String,
    tags: Vec<String>,
    worktree_config: Option<WorkspaceWorktreeConfig>,
}

pub struct WorkspaceBuilder {
    workspace: WorkspaceDescriptor,
}

impl WorkspaceBuilder {
    pub fn tag(&mut self, tag: &str);
    pub fn worktree(&mut self, on_create: &[&str], symlink_files: &[&str]);
}
```

#### Step 5.3: Implement Mixin Trait

```rust
// apps/cli/tests/common/rafaeltab_descriptors/mod.rs
pub trait RafaeltabDescriptorMixin {
    fn rafaeltab_workspace<F>(&mut self, id: &str, name: &str, f: F)
    where F: FnOnce(&mut WorkspaceBuilder);
}

// Extend test-descriptors builders
impl RafaeltabDescriptorMixin for test_descriptors::DirBuilder {
    fn rafaeltab_workspace<F>(&mut self, id: &str, name: &str, f: F)
    where F: FnOnce(&mut WorkspaceBuilder)
    {
        // Use parent_path as workspace root
        // Auto-register in config
    }
}

impl<'a> RafaeltabDescriptorMixin for test_descriptors::RootBuilder<'a> {
    fn rafaeltab_config<F>(&mut self, f: F)
    where F: FnOnce(&mut ConfigBuilder);
}
```

---

## Implementation Order (TDD-Driven)

### Stage 1: Basic Hierarchical Structure

1. Migrate `tests/directory_tests.rs` to new API (will fail)
2. Implement `DirBuilder` to make tests pass
3. Migrate `tests/git_repo_tests.rs` to new API (will fail)
4. Implement `GitBuilder` and hierarchy to make tests pass

### Stage 2: Complete Git Features

5. Migrate `tests/branch_tests.rs` and `tests/commit_tests.rs`
6. Implement `BranchBuilder` and `CommitBuilder`
7. Add staged/unstaged support (new tests)
8. Add worktree descriptor (new tests)

### Stage 3: Tmux Integration

9. Migrate `tests/tmux_tests.rs` to new API
10. Implement `SessionBuilder` with parent path awareness
11. Ensure sessions use correct working directory

### Stage 4: Query API

12. Migrate `tests/environment_tests.rs` to use query API
13. Implement `find_*()` methods and ref types
14. Add `run_cli()` support

### Stage 5: Integration Tests

15. Migrate `tests/integration_tests.rs` to new API
16. Verify everything works end-to-end

### Stage 6: Rafaeltab Integration

17. Add rafaeltab descriptors in CLI package
18. Write CLI integration tests using full system

### Stage 7: Cleanup

19. Remove old builder methods (`.with_*()`)
20. Remove `add_descriptor()` method
21. Update documentation

---

## Success Criteria

- ✅ All existing 83 tests pass with new API
- ✅ New features added (worktree, staged/unstaged, query API)
- ✅ Hierarchical nesting works correctly
- ✅ Automatic path resolution from parent
- ✅ Mixin traits allow extension
- ✅ Old API completely removed
- ✅ Rafaeltab descriptors work in CLI package
- ✅ Clean, intuitive API matching original design

---

## Timeline Estimate

- Stage 1: Basic Hierarchical Structure - 3-4 hours
- Stage 2: Complete Git Features - 3-4 hours
- Stage 3: Tmux Integration - 2 hours
- Stage 4: Query API - 3 hours
- Stage 5: Integration Tests - 2 hours
- Stage 6: Rafaeltab Integration - 2-3 hours
- Stage 7: Cleanup - 1 hour

**Total: ~16-20 hours of work**
