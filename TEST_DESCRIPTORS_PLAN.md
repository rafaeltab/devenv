# Test Descriptors Package - Implementation Plan

## Overview

A declarative framework for creating integration test environments with filesystem, git repositories, and tmux sessions. This document outlines the TDD implementation plan.

## Package Structure

```
packages/
  test-descriptors/
    Cargo.toml
    src/
      lib.rs
      descriptor/
        mod.rs
        traits.rs
        context.rs
        error.rs
      fs/
        mod.rs
        file.rs
        dir.rs
        symlink.rs
      git/
        mod.rs
        repo.rs
        branch.rs
        commit.rs
        remote.rs
        worktree.rs
        changes.rs
      tmux/
        mod.rs
        session.rs
        window.rs
        socket.rs
      environment/
        mod.rs
        builder.rs
        create.rs
        registry.rs
        queries.rs
        cli.rs
    tests/
      fs_tests.rs
      git_tests.rs
      tmux_tests.rs
      environment_tests.rs
      integration_tests.rs

apps/
  cli/
    tests/
      common/
        rafaeltab_descriptors/
          mod.rs
          config.rs
          workspace.rs
```

## Implementation Phases

---

## Phase 1: Core Infrastructure

### Module: `descriptor/traits.rs`

**Types:**

- `trait Descriptor`
- `trait PathDescriptor`

**Test Cases:** (in `tests/descriptor_tests.rs`)

- None needed - these are just trait definitions

### Module: `descriptor/error.rs`

**Types:**

- `enum CreateError`
  - `IoError(String)`
  - `GitError(String)`
  - `TmuxError(String)`
  - `InvalidDescriptor(String)`
  - `ResourceNotFound(String)`

**Test Cases:**

- `test_create_error_display_formatting` - Verify error messages are human-readable
- `test_create_error_from_io_error` - Verify conversion from std::io::Error
- `test_create_error_debug_output` - Verify Debug implementation

### Module: `descriptor/context.rs`

**Types:**

- `struct CreateContext`
  - `root_path: PathBuf`
  - `registry: ResourceRegistry`
  - `tmux_socket: Option<String>`
  - `config_path: Option<PathBuf>`

**Methods:**

- `fn new(root_path: PathBuf) -> Self`
- `fn register_resource(&mut self, key: String, path: PathBuf)`
- `fn get_resource(&self, key: &str) -> Option<&PathBuf>`
- `fn set_tmux_socket(&mut self, socket: String)`
- `fn set_config_path(&mut self, path: PathBuf)`

**Test Cases:** (in `tests/context_tests.rs`)

- `test_create_context_new` - Creates context with root path
- `test_register_and_get_resource` - Register and retrieve a resource
- `test_get_nonexistent_resource_returns_none` - Query for missing resource
- `test_set_and_get_tmux_socket` - Set tmux socket and retrieve it
- `test_set_and_get_config_path` - Set config path and retrieve it
- `test_multiple_resource_registration` - Register multiple resources

### Module: `descriptor/registry.rs`

**Types:**

- `struct ResourceRegistry`
  - `git_repos: HashMap<String, PathBuf>`
  - `worktrees: HashMap<(String, String), PathBuf>` (repo_name, branch)
  - `tmux_sessions: HashMap<String, TmuxSessionInfo>`
  - `dirs: HashMap<String, PathBuf>`

**Methods:**

- `fn new() -> Self`
- `fn register_git_repo(&mut self, name: String, path: PathBuf)`
- `fn get_git_repo(&self, name: &str) -> Option<&PathBuf>`
- `fn register_worktree(&mut self, repo: String, branch: String, path: PathBuf)`
- `fn get_worktree(&self, repo: &str, branch: &str) -> Option<&PathBuf>`
- `fn register_tmux_session(&mut self, name: String, info: TmuxSessionInfo)`
- `fn get_tmux_session(&self, name: &str) -> Option<&TmuxSessionInfo>`
- `fn register_dir(&mut self, name: String, path: PathBuf)`
- `fn get_dir(&self, name: &str) -> Option<&PathBuf>`

**Test Cases:** (in `tests/registry_tests.rs`)

- `test_register_and_get_git_repo` - Register and retrieve git repo
- `test_register_and_get_worktree` - Register and retrieve worktree
- `test_register_and_get_tmux_session` - Register and retrieve session
- `test_register_and_get_dir` - Register and retrieve directory
- `test_get_nonexistent_resources_return_none` - All getters return None for missing resources
- `test_overwrite_existing_resource` - Registering same key twice overwrites

---

## Phase 2: Filesystem Descriptors

### Module: `fs/file.rs`

**Types:**

- `struct FileDescriptor`
  - `name: String`
  - `content: String`
  - `executable: bool`

**Methods:**

- `fn new(name: &str, content: &str) -> Self`
- `fn executable(mut self) -> Self`
- `impl Descriptor for FileDescriptor`
- `impl PathDescriptor for FileDescriptor`

**Test Cases:** (in `tests/fs_tests.rs`)

- `test_file_descriptor_creates_file` - Create a simple file
- `test_file_descriptor_with_content` - File has correct content
- `test_file_descriptor_executable` - File is created with executable bit
- `test_file_descriptor_path_resolution` - Path is resolved correctly relative to parent
- `test_file_descriptor_overwrites_existing` - Overwriting existing file works
- `test_file_descriptor_empty_content` - Create file with empty string
- `test_file_descriptor_multiline_content` - Create file with newlines

### Module: `fs/dir.rs`

**Types:**

- `struct DirDescriptor`
  - `name: String`
  - `children: Vec<Box<dyn Descriptor>>`
- `struct DirBuilder`

**Methods:**

- `fn new(name: &str) -> Self`
- `fn build<F>(name: &str, f: F) -> Self where F: FnOnce(&mut DirBuilder)`
- `impl DirBuilder`:
  - `fn file(&mut self, name: &str, content: &str) -> &mut Self`
  - `fn dir<F>(&mut self, name: &str, f: F) -> &mut Self where F: FnOnce(&mut DirBuilder)`
  - `fn symlink(&mut self, name: &str, target: &str) -> &mut Self`
- `impl Descriptor for DirDescriptor`
- `impl PathDescriptor for DirDescriptor`

**Test Cases:** (in `tests/fs_tests.rs`)

- `test_dir_descriptor_creates_empty_dir` - Create empty directory
- `test_dir_descriptor_with_file_child` - Directory with one file
- `test_dir_descriptor_with_multiple_children` - Directory with multiple files
- `test_dir_descriptor_nested` - Nested directories
- `test_dir_descriptor_path_resolution` - Path resolves correctly
- `test_dir_builder_fluent_api` - Builder pattern works fluently
- `test_dir_descriptor_already_exists` - Creating dir that exists doesn't error

### Module: `fs/symlink.rs`

**Types:**

- `struct SymlinkDescriptor`
  - `name: String`
  - `target: String`

**Methods:**

- `fn new(name: &str, target: &str) -> Self`
- `impl Descriptor for SymlinkDescriptor`
- `impl PathDescriptor for SymlinkDescriptor`

**Test Cases:** (in `tests/fs_tests.rs`)

- `test_symlink_descriptor_creates_symlink` - Create symbolic link
- `test_symlink_descriptor_relative_target` - Symlink with relative path
- `test_symlink_descriptor_absolute_target` - Symlink with absolute path
- `test_symlink_descriptor_to_file` - Symlink points to file
- `test_symlink_descriptor_to_directory` - Symlink points to directory
- `test_symlink_descriptor_broken_link` - Symlink with non-existent target (should still create)

---

## Phase 3: Git Descriptors

### Module: `git/remote.rs`

**Types:**

- `struct RemoteDescriptor`
  - `name: String`
  - `bare_repo_path: Option<PathBuf>` (set during creation)

**Methods:**

- `fn new(name: &str) -> Self`
- `fn create_bare_repo(&self, context: &CreateContext) -> Result<PathBuf, CreateError>`

**Test Cases:** (in `tests/git_tests.rs`)

- `test_remote_descriptor_creates_bare_repo` - Create a bare git repository
- `test_remote_descriptor_bare_repo_is_valid` - Bare repo can be used as remote
- `test_remote_descriptor_multiple_remotes` - Multiple remotes in same repo
- `test_remote_descriptor_path_is_isolated` - Each remote gets unique temp dir

### Module: `git/commit.rs`

**Types:**

- `enum FileChange`
  - `Write { path: String, content: String }`
  - `Delete { path: String }`
- `struct CommitDescriptor`
  - `message: String`
  - `changes: Vec<FileChange>`
  - `pushed_to: Option<String>` (remote name)
  - `pushed_as: Option<String>` (remote branch name, defaults to local branch)
- `struct CommitBuilder`

**Methods:**

- `fn new(message: &str) -> Self`
- `impl CommitBuilder`:
  - `fn file(&mut self, path: &str, content: &str) -> &mut Self`
  - `fn delete(&mut self, path: &str) -> &mut Self`
  - `fn pushed(&mut self, remote: &str) -> &mut Self`
  - `fn pushed_as(&mut self, remote: &str, branch: &str) -> &mut Self`
  - `fn build(self) -> CommitDescriptor`

**Test Cases:** (in `tests/git_tests.rs`)

- `test_commit_descriptor_single_file_change` - Commit with one file
- `test_commit_descriptor_multiple_file_changes` - Commit with multiple files
- `test_commit_descriptor_file_deletion` - Commit that deletes a file
- `test_commit_descriptor_mixed_changes` - Commit with adds and deletes
- `test_commit_builder_fluent_api` - Builder works fluently
- `test_commit_descriptor_pushed_marker` - Commit marked as pushed
- `test_commit_descriptor_pushed_as_different_branch` - Pushed to different remote branch name

### Module: `git/branch.rs`

**Types:**

- `struct BranchDescriptor`
  - `name: String`
  - `base: Option<String>` (branch to base from)
  - `commits: Vec<CommitDescriptor>`
- `struct BranchBuilder`

**Methods:**

- `fn new(name: &str) -> Self`
- `fn from(name: &str, base: &str) -> Self`
- `impl BranchBuilder`:
  - `fn commit<F>(&mut self, message: &str, f: F) -> &mut Self where F: FnOnce(&mut CommitBuilder)`
  - `fn build(self) -> BranchDescriptor`

**Test Cases:** (in `tests/git_tests.rs`)

- `test_branch_descriptor_creates_branch` - Create a simple branch
- `test_branch_descriptor_with_commits` - Branch with commits
- `test_branch_descriptor_from_base` - Branch based on another branch
- `test_branch_descriptor_multiple_commits_in_order` - Multiple commits are applied in order
- `test_branch_builder_fluent_api` - Builder works fluently

### Module: `git/repo.rs`

**Types:**

- `struct GitRepoDescriptor`
  - `name: String`
  - `remotes: Vec<RemoteDescriptor>`
  - `branches: Vec<BranchDescriptor>`
  - `initial_branch: String` (default: "main")
- `struct GitBuilder`

**Methods:**

- `fn new(name: &str) -> Self`
- `impl GitBuilder`:
  - `fn remote(&mut self, name: &str) -> &mut Self`
  - `fn branch<F>(&mut self, name: &str, f: F) -> &mut Self where F: FnOnce(&mut BranchBuilder)`
  - `fn branch_from<F>(&mut self, name: &str, base: &str, f: F) -> &mut Self`
  - `fn initial_branch(&mut self, name: &str) -> &mut Self`
  - `fn build(self) -> GitRepoDescriptor`
- `impl Descriptor for GitRepoDescriptor`
- `impl PathDescriptor for GitRepoDescriptor`

**Test Cases:** (in `tests/git_tests.rs`)

- `test_git_repo_descriptor_creates_repo` - Create basic git repo
- `test_git_repo_descriptor_with_initial_commit` - Repo with initial commit on main
- `test_git_repo_descriptor_with_remote` - Repo with remote configured
- `test_git_repo_descriptor_with_multiple_branches` - Repo with multiple branches
- `test_git_repo_descriptor_branch_from_another` - Create branch from another branch
- `test_git_repo_descriptor_push_commits_to_remote` - Commits marked as pushed are in remote
- `test_git_repo_descriptor_unpushed_commits_not_in_remote` - Unpushed commits absent from remote
- `test_git_repo_descriptor_custom_initial_branch` - Use "master" instead of "main"
- `test_git_repo_descriptor_registered_in_context` - Repo is registered by name
- `test_git_repo_descriptor_has_valid_git_config` - Git config has user.name and user.email

### Module: `git/changes.rs`

**Types:**

- `struct StagedDescriptor`
  - `changes: Vec<FileChange>`
- `struct StagedBuilder`
- `struct UnstagedDescriptor`
  - `changes: Vec<FileChange>`
  - `untracked: Vec<(String, String)>` (path, content)
- `struct UnstagedBuilder`

**Methods:**

- `impl StagedBuilder`:
  - `fn file(&mut self, path: &str, content: &str) -> &mut Self`
  - `fn delete(&mut self, path: &str) -> &mut Self`
  - `fn build(self) -> StagedDescriptor`
- `impl UnstagedBuilder`:
  - `fn modify(&mut self, path: &str, content: &str) -> &mut Self`
  - `fn untracked(&mut self, path: &str, content: &str) -> &mut Self`
  - `fn delete(&mut self, path: &str) -> &mut Self`
  - `fn build(self) -> UnstagedDescriptor`

**Test Cases:** (in `tests/git_tests.rs`)

- `test_staged_descriptor_stages_new_file` - Stage a new file
- `test_staged_descriptor_stages_modification` - Stage modification to existing file
- `test_staged_descriptor_stages_deletion` - Stage file deletion
- `test_unstaged_descriptor_untracked_file` - Create untracked file
- `test_unstaged_descriptor_modified_file` - Modify tracked file without staging
- `test_unstaged_descriptor_deleted_file` - Delete tracked file without staging
- `test_staged_and_unstaged_together` - Both staged and unstaged changes in same repo
- `test_staged_changes_shown_in_git_status` - `git status` shows staged files
- `test_unstaged_changes_shown_in_git_status` - `git status` shows unstaged files

### Module: `git/worktree.rs`

**Types:**

- `struct GitWorktreeDescriptor`
  - `repo_name: String`
  - `base_branch: String`
  - `branch: String`
  - `commits: Vec<CommitDescriptor>`
  - `staged: Option<StagedDescriptor>`
  - `unstaged: Option<UnstagedDescriptor>`
- `struct WorktreeBuilder`

**Methods:**

- `fn new(repo_name: &str, base_branch: &str, branch: &str) -> Self`
- `impl WorktreeBuilder`:
  - `fn commit<F>(&mut self, message: &str, f: F) -> &mut Self where F: FnOnce(&mut CommitBuilder)`
  - `fn staged<F>(&mut self, f: F) -> &mut Self where F: FnOnce(&mut StagedBuilder)`
  - `fn unstaged<F>(&mut self, f: F) -> &mut Self where F: FnOnce(&mut UnstagedBuilder)`
  - `fn build(self) -> GitWorktreeDescriptor`
- `impl Descriptor for GitWorktreeDescriptor`
- `impl PathDescriptor for GitWorktreeDescriptor`

**Test Cases:** (in `tests/git_tests.rs`)

- `test_worktree_descriptor_creates_worktree` - Create basic worktree
- `test_worktree_descriptor_new_branch` - Worktree creates new branch
- `test_worktree_descriptor_existing_branch` - Worktree for existing branch
- `test_worktree_descriptor_with_commits` - Worktree with commits on its branch
- `test_worktree_descriptor_with_staged_changes` - Worktree with staged changes
- `test_worktree_descriptor_with_unstaged_changes` - Worktree with unstaged changes
- `test_worktree_descriptor_references_repo_by_name` - Worktree links to correct repo
- `test_worktree_descriptor_path_based_on_parent_dir` - Worktree created at parent dir path
- `test_worktree_descriptor_registered_in_context` - Worktree registered by (repo, branch)
- `test_worktree_descriptor_base_branch_validation` - Error if base branch doesn't exist
- `test_worktree_descriptor_shares_git_history` - Worktree sees commits from main repo

---

## Phase 4: Tmux Descriptors

### Module: `tmux/socket.rs`

**Types:**

- `struct TmuxSocket`
  - `name: String`

**Methods:**

- `fn new() -> Self` (generates UUID-based name)
- `fn name(&self) -> &str`
- `fn run_tmux(&self, args: &[&str]) -> Result<String, CreateError>`
- `fn list_sessions(&self) -> Result<Vec<String>, CreateError>`
- `fn session_exists(&self, name: &str) -> bool`
- `fn kill_server(&self) -> Result<(), CreateError>`

**Test Cases:** (in `tests/tmux_tests.rs`)

- `test_tmux_socket_creates_unique_name` - Each socket has unique name
- `test_tmux_socket_run_command` - Run tmux command on socket
- `test_tmux_socket_list_sessions_empty` - New socket has no sessions
- `test_tmux_socket_session_exists_false` - Non-existent session returns false
- `test_tmux_socket_kill_server` - Kill server cleans up socket

### Module: `tmux/window.rs`

**Types:**

- `struct WindowDescriptor`
  - `name: String`
  - `command: Option<String>`

**Methods:**

- `fn new(name: &str) -> Self`
- `fn command(mut self, cmd: &str) -> Self`

**Test Cases:** (in `tests/tmux_tests.rs`)

- `test_window_descriptor_simple` - Create window descriptor
- `test_window_descriptor_with_command` - Window with custom command

### Module: `tmux/session.rs`

**Types:**

- `struct TmuxSessionDescriptor`
  - `name: String`
  - `windows: Vec<WindowDescriptor>`
  - `working_dir: Option<PathBuf>` (set during creation from parent dir)
- `struct SessionBuilder`

**Methods:**

- `fn new(name: &str) -> Self`
- `impl SessionBuilder`:
  - `fn window(&mut self, name: &str) -> &mut Self`
  - `fn window_with_command(&mut self, name: &str, command: &str) -> &mut Self`
  - `fn build(self) -> TmuxSessionDescriptor`
- `impl Descriptor for TmuxSessionDescriptor`

**Test Cases:** (in `tests/tmux_tests.rs`)

- `test_tmux_session_descriptor_creates_session` - Create basic session
- `test_tmux_session_descriptor_with_window` - Session with one window
- `test_tmux_session_descriptor_with_multiple_windows` - Session with multiple windows
- `test_tmux_session_descriptor_window_with_command` - Window executes command
- `test_tmux_session_descriptor_working_directory` - Session uses parent dir as working dir
- `test_tmux_session_descriptor_registered_in_context` - Session registered by name
- `test_tmux_session_descriptor_visible_in_list_sessions` - Created session appears in list
- `test_tmux_session_descriptor_isolated_from_default_server` - Uses isolated socket

---

## Phase 5: Environment Orchestration

### Module: `environment/builder.rs`

**Types:**

- `struct TestEnvironment`
  - `root_dir: TempDir`
  - `context: CreateContext`
  - `tmux_socket: TmuxSocket`
- `struct RootBuilder`
- `struct TestDirBuilder`

**Methods:**

- `impl TestEnvironment`:
  - `fn describe<F>(f: F) -> Self where F: FnOnce(&mut RootBuilder)`
  - `fn create(self) -> Self` (materializes everything)
- `impl RootBuilder`:
  - `fn test_dir<F>(&mut self, f: F) -> &mut Self where F: FnOnce(&mut TestDirBuilder)`
- `impl TestDirBuilder`:
  - Implements all mixins (file, dir, git, tmux, etc.)

**Test Cases:** (in `tests/environment_tests.rs`)

- `test_environment_describe_creates_temp_dir` - Temp dir is created
- `test_environment_describe_creates_tmux_socket` - Tmux socket is isolated
- `test_environment_describe_empty` - Empty environment can be created
- `test_environment_describe_with_single_file` - Environment with one file
- `test_environment_create_materializes_all_descriptors` - All descriptors are created
- `test_environment_drop_cleans_up` - Drop removes temp dir and kills tmux

### Module: `environment/registry.rs`

(Covered in Phase 1, but integration tests here)

**Test Cases:** (in `tests/environment_tests.rs`)

- `test_registry_tracks_created_git_repos` - Git repos are registered
- `test_registry_tracks_created_worktrees` - Worktrees are registered
- `test_registry_tracks_created_tmux_sessions` - Sessions are registered
- `test_registry_tracks_created_dirs` - Directories are registered

### Module: `environment/queries.rs`

**Types:**

- `struct DirRef<'a>`
- `struct GitRepoRef<'a>`
- `struct WorktreeRef<'a>`
- `struct TmuxSessionRef<'a>`

**Methods:**

- `impl TestEnvironment`:
  - `fn find_dir(&self, name: &str) -> Option<DirRef>`
  - `fn find_git_repo(&self, name: &str) -> Option<GitRepoRef>`
  - `fn find_worktree(&self, repo: &str, branch: &str) -> Option<WorktreeRef>`
  - `fn find_tmux_session(&self, name: &str) -> Option<TmuxSessionRef>`
  - `fn config_path(&self) -> Option<&Path>`
  - `fn tmux_socket(&self) -> &str`
- `impl DirRef<'a>`:
  - `fn path(&self) -> &Path`
  - `fn exists(&self) -> bool`
  - `fn contains_file(&self, name: &str) -> bool`
  - `fn read_file(&self, name: &str) -> Option<String>`
- `impl GitRepoRef<'a>`:
  - `fn path(&self) -> &Path`
  - `fn current_branch(&self) -> String`
  - `fn branches(&self) -> Vec<String>`
  - `fn is_clean(&self) -> bool`
  - `fn has_unpushed_commits(&self) -> bool`
  - `fn has_staged_changes(&self) -> bool`
  - `fn has_unstaged_changes(&self) -> bool`
  - `fn git(&self, args: &[&str]) -> String`
- `impl WorktreeRef<'a>`:
  - `fn path(&self) -> &Path`
  - `fn exists(&self) -> bool`
  - `fn branch(&self) -> &str`
  - `fn is_clean(&self) -> bool`
  - `fn has_unpushed_commits(&self) -> bool`
- `impl TmuxSessionRef<'a>`:
  - `fn name(&self) -> &str`
  - `fn exists(&self) -> bool`
  - `fn working_dir(&self) -> &Path`
  - `fn windows(&self) -> Vec<String>`

**Test Cases:** (in `tests/environment_tests.rs`)

- `test_find_dir_existing` - Find existing directory
- `test_find_dir_nonexistent` - Returns None for missing directory
- `test_dir_ref_path` - DirRef returns correct path
- `test_dir_ref_exists` - DirRef.exists() returns true
- `test_dir_ref_contains_file` - DirRef.contains_file() works
- `test_dir_ref_read_file` - DirRef.read_file() returns content
- `test_find_git_repo_existing` - Find existing git repo
- `test_find_git_repo_nonexistent` - Returns None for missing repo
- `test_git_repo_ref_current_branch` - GitRepoRef.current_branch() works
- `test_git_repo_ref_branches` - GitRepoRef.branches() lists all branches
- `test_git_repo_ref_is_clean` - GitRepoRef.is_clean() detects clean state
- `test_git_repo_ref_has_unpushed_commits` - Detects unpushed commits
- `test_git_repo_ref_has_staged_changes` - Detects staged changes
- `test_git_repo_ref_has_unstaged_changes` - Detects unstaged changes
- `test_git_repo_ref_git_command` - Run arbitrary git command
- `test_find_worktree_existing` - Find existing worktree
- `test_find_worktree_nonexistent` - Returns None for missing worktree
- `test_worktree_ref_exists` - WorktreeRef.exists() works
- `test_worktree_ref_branch` - WorktreeRef.branch() returns correct branch
- `test_find_tmux_session_existing` - Find existing session
- `test_find_tmux_session_nonexistent` - Returns None for missing session
- `test_tmux_session_ref_exists` - TmuxSessionRef.exists() works
- `test_tmux_session_ref_working_dir` - Session has correct working dir
- `test_tmux_session_ref_windows` - List windows in session

### Module: `environment/cli.rs`

**Types:**

- `struct CliOutput`
  - `stdout: String`
  - `stderr: String`
  - `status: ExitStatus`

**Methods:**

- `impl TmuxSessionRef<'a>`:
  - `fn run_cli(&self, args: &[&str]) -> CliOutput`
- `impl CliOutput`:
  - `fn success(&self) -> bool`
  - `fn assert_success(&self) -> &Self`
  - `fn assert_failure(&self) -> &Self`
  - `fn assert_stdout_contains(&self, expected: &str) -> &Self`
  - `fn assert_stderr_contains(&self, expected: &str) -> &Self`

**Test Cases:** (in `tests/environment_tests.rs`)

- `test_cli_output_success` - CliOutput.success() detects success
- `test_cli_output_failure` - CliOutput.success() detects failure
- `test_cli_output_assert_success` - assert_success() doesn't panic on success
- `test_cli_output_assert_success_panics` - assert_success() panics on failure
- `test_cli_output_assert_stdout_contains` - assert_stdout_contains() works
- `test_cli_output_assert_stdout_contains_panics` - Panics when stdout doesn't contain
- `test_cli_output_assert_stderr_contains` - assert_stderr_contains() works

Note: Full `run_cli` integration tests will be in Phase 6 with rafaeltab descriptors.

---

## Phase 6: Integration Tests

### Module: `tests/integration_tests.rs`

**Test Cases:**

- `test_full_environment_with_git_and_tmux` - Create env with git repo and tmux session
- `test_multiple_branches_and_worktrees` - Complex scenario with multiple branches/worktrees
- `test_git_push_to_local_remote` - Commits pushed to local bare remote
- `test_worktree_sees_main_repo_commits` - Worktree has access to main repo history
- `test_staged_and_unstaged_changes_persist` - Working dir state is correct
- `test_tmux_sessions_in_correct_directories` - Sessions have correct working directories
- `test_environment_cleanup_removes_everything` - Drop cleans up all resources
- `test_query_api_after_external_changes` - Queries reflect manual file/git changes
- `test_nested_directory_structure` - Deep directory nesting works
- `test_multiple_git_repos_in_same_environment` - Multiple repos don't interfere

---

## Phase 7: Rafaeltab-Specific Descriptors (in CLI package)

Location: `apps/cli/tests/common/rafaeltab_descriptors/`

### Module: `config.rs`

**Types:**

- `struct ConfigDescriptor`
  - `workspaces: Vec<WorkspaceConfig>`
  - `tmux_sessions: Vec<TmuxSessionConfig>`
  - `default_windows: Vec<WindowConfig>`
  - `global_worktree: Option<GlobalWorktreeConfig>`
- `struct ConfigBuilder`

**Methods:**

- `fn new() -> Self`
- `impl ConfigBuilder`:
  - `fn defaults(&mut self) -> &mut Self`
  - `fn add_workspace(&mut self, config: WorkspaceConfig) -> &mut Self`
  - `fn worktree_global(&mut self, on_create: &[&str], symlink_files: &[&str]) -> &mut Self`
  - `fn build(self) -> ConfigDescriptor`
- `impl Descriptor for ConfigDescriptor`

**Test Cases:** (in `apps/cli/tests/rafaeltab_descriptor_tests.rs`)

- `test_config_descriptor_defaults` - Creates valid empty config
- `test_config_descriptor_generates_json_file` - Config written to temp file
- `test_config_descriptor_registered_in_context` - Config path tracked
- `test_config_descriptor_with_workspaces` - Config includes registered workspaces
- `test_config_descriptor_with_global_worktree` - Global worktree config included
- `test_config_descriptor_valid_json_schema` - Generated JSON is valid per schema

### Module: `workspace.rs`

**Types:**

- `struct WorkspaceDescriptor`
  - `id: String`
  - `name: String`
  - `tags: Vec<String>`
  - `worktree_config: Option<WorkspaceWorktreeConfig>`
  - `path: PathBuf` (set from parent dir)
- `struct WorkspaceBuilder`

**Methods:**

- `fn new(id: &str, name: &str) -> Self`
- `impl WorkspaceBuilder`:
  - `fn tag(&mut self, tag: &str) -> &mut Self`
  - `fn worktree(&mut self, on_create: &[&str], symlink_files: &[&str]) -> &mut Self`
  - `fn build(self) -> WorkspaceDescriptor`
- `impl Descriptor for WorkspaceDescriptor`

**Test Cases:** (in `apps/cli/tests/rafaeltab_descriptor_tests.rs`)

- `test_workspace_descriptor_basic` - Create basic workspace
- `test_workspace_descriptor_with_tags` - Workspace with tags
- `test_workspace_descriptor_with_worktree_config` - Workspace with worktree config
- `test_workspace_descriptor_auto_registers_in_config` - Added to config automatically
- `test_workspace_descriptor_path_from_parent_dir` - Path set from containing directory
- `test_workspace_descriptor_multiple_workspaces` - Multiple workspaces in same config

### Integration with DirBuilder

**Extension Trait:**

- `trait RafaeltabDescriptorMixin`
  - `fn rafaeltab_workspace<F>(&mut self, id: &str, name: &str, f: F) -> &mut Self`

**Test Cases:** (in `apps/cli/tests/rafaeltab_descriptor_tests.rs`)

- `test_dir_builder_rafaeltab_workspace_mixin` - Workspace can be added to dir
- `test_workspace_uses_dir_path_as_root` - Workspace root = dir path

---

## Phase 8: CLI Integration Tests (using test-descriptors)

Location: `apps/cli/tests/integration_tests/`

### Test File: `worktree_start_tests.rs`

**Test Cases:**

- `test_worktree_start_creates_new_branch` - Start worktree creates new branch
- `test_worktree_start_creates_directory` - Worktree directory is created
- `test_worktree_start_creates_tmux_session` - Tmux session is created
- `test_worktree_start_creates_symlinks` - Configured symlinks are created
- `test_worktree_start_runs_on_create_commands` - onCreate commands execute
- `test_worktree_start_from_existing_local_branch` - Use existing local branch
- `test_worktree_start_from_existing_remote_branch` - Track remote branch
- `test_worktree_start_fails_when_path_exists` - Error when worktree path exists
- `test_worktree_start_fails_without_workspace` - Error when not in workspace
- `test_worktree_start_fails_without_config` - Error when worktree config missing

### Test File: `worktree_complete_tests.rs`

**Test Cases:**

- `test_worktree_complete_removes_worktree` - Removes git worktree
- `test_worktree_complete_removes_directory` - Removes worktree directory
- `test_worktree_complete_kills_tmux_session` - Kills worktree's session
- `test_worktree_complete_fails_with_uncommitted_changes` - Error with dirty working dir
- `test_worktree_complete_fails_with_unpushed_commits` - Error with unpushed commits
- `test_worktree_complete_force_bypasses_checks` - --force ignores uncommitted/unpushed
- `test_worktree_complete_cleans_up_empty_dirs` - Removes empty parent directories
- `test_worktree_complete_from_different_session` - Complete from outside worktree session
- `test_worktree_complete_from_worktree_session` - Complete from inside (delegates to popup)
- `test_worktree_complete_with_branch_name_arg` - Complete specific worktree by branch name

---

## Dependencies

### `packages/test-descriptors/Cargo.toml`

```toml
[package]
name = "test-descriptors"
version = "0.1.0"
edition = "2021"

[dependencies]
tempfile = "3"
uuid = { version = "1", features = ["v4"] }
thiserror = "1"

[dev-dependencies]
# For testing the test-descriptors package itself
```

### `apps/cli/Cargo.toml` (additions)

```toml
[dev-dependencies]
test-descriptors = { path = "../../packages/test-descriptors" }
```

---

## Implementation Order

1. **Phase 1**: Core infrastructure (traits, context, registry, error)
2. **Phase 2**: Filesystem descriptors (file, dir, symlink)
3. **Phase 3**: Git descriptors (remote, commit, branch, repo, changes, worktree)
4. **Phase 4**: Tmux descriptors (socket, window, session)
5. **Phase 5**: Environment orchestration (builder, registry, queries, cli)
6. **Phase 6**: Integration tests for test-descriptors package
7. **Phase 7**: Rafaeltab-specific descriptors in CLI package
8. **Phase 8**: Migrate/write CLI integration tests using new framework

---

## Success Criteria

- [ ] All test cases pass
- [ ] Test coverage > 80% for test-descriptors package
- [ ] At least 5 CLI integration tests migrated to new framework
- [ ] Documentation includes usage examples
- [ ] No memory leaks (all temp resources cleaned up)
- [ ] Tmux sessions properly isolated (no interference with user's tmux)
- [ ] Git operations work correctly with local remotes
- [ ] Error messages are clear and actionable

---

## Notes

- Use TDD: Write test first, then implementation
- Each module should have tests in the `tests/` directory
- Keep descriptor creation separate from materialization (create vs execute)
- Use builder pattern for complex nested structures
- Panic on errors during test setup (tests should fail fast)
- Clean up resources in Drop implementations
- Document public API thoroughly
