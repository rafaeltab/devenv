# Test Descriptors - Usage Examples

This document shows how to use the test-descriptors framework for declarative integration testing.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Git Repository Testing](#git-repository-testing)
- [Tmux Session Testing](#tmux-session-testing)
- [Complex Scenarios](#complex-scenarios)
- [Real-World Example](#real-world-example)

## Basic Usage

### Simple Directory Creation

```rust
use test_descriptors::{TestEnvironment, DirectoryDescriptor};

#[test]
fn test_create_directory() {
    let mut env = TestEnvironment::new();

    env.add_descriptor(DirectoryDescriptor::new("my-workspace"));
    env.create().unwrap();

    assert!(env.root_path().join("my-workspace").exists());
}
```

### Creating Nested Directories

```rust
use test_descriptors::{TestEnvironment, DirectoryDescriptor};

#[test]
fn test_nested_directories() {
    let mut env = TestEnvironment::new();

    env.add_descriptor(DirectoryDescriptor::new("project/src/components"));
    env.create().unwrap();

    assert!(env.root_path().join("project/src/components").exists());
}
```

## Git Repository Testing

### Basic Git Repository

```rust
use test_descriptors::{TestEnvironment, GitRepoDescriptor};

#[test]
fn test_git_repo() {
    let mut env = TestEnvironment::new();

    let repo = GitRepoDescriptor::new("my-repo");
    env.add_descriptor(repo);
    env.create().unwrap();

    assert!(env.root_path().join("my-repo/.git").exists());
}
```

### Repository with Branches and Commits

```rust
use test_descriptors::{
    TestEnvironment, GitRepoDescriptor, BranchDescriptor, CommitDescriptor
};

#[test]
fn test_repo_with_branches() {
    let mut env = TestEnvironment::new();

    // Create a feature branch with commits
    let feature_branch = BranchDescriptor::new("feature")
        .with_commit(
            CommitDescriptor::new("Add feature")
                .with_file("feature.txt", "feature implementation")
        )
        .with_commit(
            CommitDescriptor::new("Update feature")
                .with_file("feature.txt", "improved implementation")
        );

    let repo = GitRepoDescriptor::new("my-repo")
        .with_branch(feature_branch);

    env.add_descriptor(repo);
    env.create().unwrap();

    // Verify the branch exists
    let repo_path = env.root_path().join("my-repo");
    assert!(repo_path.join(".git").exists());
}
```

### Repository with Remote

```rust
use test_descriptors::{
    TestEnvironment, GitRepoDescriptor, RemoteDescriptor
};

#[test]
fn test_repo_with_remote() {
    let mut env = TestEnvironment::new();

    let remote = RemoteDescriptor::new("origin");
    let repo = GitRepoDescriptor::new("my-repo")
        .with_remote(remote);

    env.add_descriptor(repo);
    env.create().unwrap();

    // You can now push to the local bare remote!
}
```

### Testing Branch Creation from Another Branch

```rust
use test_descriptors::{
    TestEnvironment, GitRepoDescriptor, BranchDescriptor, CommitDescriptor
};

#[test]
fn test_branch_from_base() {
    let mut env = TestEnvironment::new();

    // Create develop branch
    let develop = BranchDescriptor::new("develop")
        .with_commit(
            CommitDescriptor::new("Dev work")
                .with_file("dev.txt", "development")
        );

    // Create feature branch from develop
    let feature = BranchDescriptor::from("feature", "develop")
        .with_commit(
            CommitDescriptor::new("Feature work")
                .with_file("feature.txt", "new feature")
        );

    let repo = GitRepoDescriptor::new("my-repo")
        .with_branch(develop)
        .with_branch(feature);

    env.add_descriptor(repo);
    env.create().unwrap();
}
```

## Tmux Session Testing

### Basic Tmux Session

```rust
use test_descriptors::{TestEnvironment, TmuxSessionDescriptor};

#[test]
fn test_tmux_session() {
    let mut env = TestEnvironment::new();

    let session = TmuxSessionDescriptor::new("dev-session");
    env.add_descriptor(session);
    env.create().unwrap();

    assert!(env.tmux().session_exists("dev-session"));
}
```

### Session with Multiple Windows

```rust
use test_descriptors::{
    TestEnvironment, TmuxSessionDescriptor, WindowDescriptor
};

#[test]
fn test_tmux_with_windows() {
    let mut env = TestEnvironment::new();

    let session = TmuxSessionDescriptor::new("dev-session")
        .with_window(WindowDescriptor::new("editor"))
        .with_window(WindowDescriptor::new("server"))
        .with_window(WindowDescriptor::new("tests"));

    env.add_descriptor(session);
    env.create().unwrap();

    // Verify windows exist
    let output = env.tmux()
        .run_tmux(&["list-windows", "-t", "dev-session"])
        .unwrap();

    assert!(output.contains("editor"));
    assert!(output.contains("server"));
    assert!(output.contains("tests"));
}
```

### Complete Isolation

All tmux sessions created through test-descriptors are completely isolated from your actual tmux server:

```rust
use test_descriptors::{TestEnvironment, TmuxSessionDescriptor};

#[test]
fn test_tmux_isolation() {
    let mut env = TestEnvironment::new();

    let session = TmuxSessionDescriptor::new("test-session");
    env.add_descriptor(session);
    env.create().unwrap();

    // This session exists in the test environment
    assert!(env.tmux().session_exists("test-session"));

    // But NOT in your default tmux server
    let check = std::process::Command::new("tmux")
        .args(&["has-session", "-t", "test-session"])
        .output()
        .unwrap();
    assert!(!check.status.success());
}
```

## Complex Scenarios

### Full Development Environment

```rust
use test_descriptors::{
    TestEnvironment, DirectoryDescriptor, GitRepoDescriptor,
    BranchDescriptor, CommitDescriptor, RemoteDescriptor,
    TmuxSessionDescriptor, WindowDescriptor
};

#[test]
fn test_full_dev_environment() {
    let mut env = TestEnvironment::new();

    // Create workspace directory
    env.add_descriptor(DirectoryDescriptor::new("workspace"));

    // Create git repository with branches
    let feature = BranchDescriptor::new("feature/new-feature")
        .with_commit(
            CommitDescriptor::new("WIP: new feature")
                .with_file("src/feature.rs", "// feature code")
        );

    let repo = GitRepoDescriptor::new("workspace/my-project")
        .with_remote(RemoteDescriptor::new("origin"))
        .with_branch(feature);

    // Create tmux session for development
    let session = TmuxSessionDescriptor::new("my-project")
        .with_window(WindowDescriptor::new("editor"))
        .with_window(WindowDescriptor::new("server"))
        .with_window(WindowDescriptor::new("tests"));

    env.add_descriptor(repo);
    env.add_descriptor(session);
    env.create().unwrap();

    // Now you have a complete development environment ready for testing!
    assert!(env.root_path().join("workspace/my-project/.git").exists());
    assert!(env.tmux().session_exists("my-project"));
}
```

### Multiple Repositories

```rust
use test_descriptors::{TestEnvironment, GitRepoDescriptor, BranchDescriptor, CommitDescriptor};

#[test]
fn test_multiple_repos() {
    let mut env = TestEnvironment::new();

    // Frontend repository
    let frontend_feature = BranchDescriptor::new("feature/ui")
        .with_commit(
            CommitDescriptor::new("Add UI components")
                .with_file("components/Button.tsx", "export const Button = () => {}")
        );

    let frontend = GitRepoDescriptor::new("frontend")
        .with_branch(frontend_feature);

    // Backend repository
    let backend_feature = BranchDescriptor::new("feature/api")
        .with_commit(
            CommitDescriptor::new("Add API endpoints")
                .with_file("src/api.rs", "// API implementation")
        );

    let backend = GitRepoDescriptor::new("backend")
        .with_branch(backend_feature);

    env.add_descriptor(frontend);
    env.add_descriptor(backend);
    env.create().unwrap();

    // Both repos exist and don't interfere with each other
    assert!(env.root_path().join("frontend/.git").exists());
    assert!(env.root_path().join("backend/.git").exists());
}
```

## Real-World Example

### Testing a CLI Tool's Workspace Management

Here's a complete example from the rafaeltab CLI integration tests:

```rust
use test_descriptors::{TestEnvironment, GitRepoDescriptor, TmuxSessionDescriptor, WindowDescriptor};

#[test]
fn test_cli_workspace_scenario() {
    let mut env = TestEnvironment::new();

    // Setup: Create a workspace with a git repository
    let repo = GitRepoDescriptor::new("my-project/app");

    // Create tmux session for the workspace
    let session = TmuxSessionDescriptor::new("my-project")
        .with_window(WindowDescriptor::new("code"))
        .with_window(WindowDescriptor::new("server"));

    env.add_descriptor(repo);
    env.add_descriptor(session);
    env.create().unwrap();

    // Test: Run your CLI command
    let output = std::process::Command::new("my-cli")
        .args(&["workspace", "list"])
        .env("MY_CLI_ROOT", env.root_path())
        .output()
        .unwrap();

    // Verify: Check the output
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("my-project"));
}
```

## Automatic Cleanup

All test environments automatically clean up after themselves:

```rust
use test_descriptors::{TestEnvironment, GitRepoDescriptor, TmuxSessionDescriptor};

#[test]
fn test_automatic_cleanup() {
    let temp_path;
    let socket_name;

    {
        let mut env = TestEnvironment::new();
        temp_path = env.root_path().to_path_buf();
        socket_name = env.tmux_socket().to_string();

        env.add_descriptor(GitRepoDescriptor::new("my-repo"));
        env.add_descriptor(TmuxSessionDescriptor::new("my-session"));
        env.create().unwrap();

        assert!(temp_path.exists());
        assert!(env.tmux().session_exists("my-session"));
    } // env is dropped here

    // Everything is cleaned up automatically!
    assert!(!temp_path.exists());
}
```

## Tips and Best Practices

1. **Use descriptors declaratively** - Add all descriptors before calling `create()`
2. **Leverage the context** - Use `env.context()` to access the shared context
3. **Check registry** - Use `env.context().registry()` to look up created resources
4. **Tmux isolation** - All tmux sessions are isolated by default via unique sockets
5. **Git config** - All git repos have test user.name and user.email pre-configured
6. **Root path access** - Use `env.root_path()` to get the temp directory path

## See Also

- `packages/test-descriptors/tests/integration_tests.rs` - Comprehensive integration tests
- `apps/cli/tests/cli_integration_tests.rs` - Real-world CLI testing examples
- `packages/test-descriptors/tests/environment_tests.rs` - TestEnvironment examples
