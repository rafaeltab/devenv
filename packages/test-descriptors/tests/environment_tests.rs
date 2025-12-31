use std::fs;
use test_descriptors::{
    BranchDescriptor, CommitDescriptor, DirectoryDescriptor, GitRepoDescriptor, RemoteDescriptor,
    TestEnvironment, TmuxSessionDescriptor, WindowDescriptor,
};

#[test]
fn test_environment_creates_temp_dir() {
    let env = TestEnvironment::new();

    assert!(env.root_path().exists());
    assert!(env.root_path().is_dir());
}

#[test]
fn test_environment_creates_tmux_socket() {
    let env = TestEnvironment::new();

    assert!(env.tmux_socket().len() > 0);
}

#[test]
fn test_environment_with_directory() {
    let mut env = TestEnvironment::new();
    env.add_descriptor(DirectoryDescriptor::new("test-dir"));
    env.create().unwrap();

    let dir_path = env.root_path().join("test-dir");
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

#[test]
fn test_environment_with_git_repo() {
    let mut env = TestEnvironment::new();
    env.add_descriptor(GitRepoDescriptor::new("my-repo"));
    env.create().unwrap();

    let repo_path = env.root_path().join("my-repo");
    assert!(repo_path.exists());
    assert!(repo_path.join(".git").exists());
}

#[test]
fn test_environment_with_tmux_session() {
    let mut env = TestEnvironment::new();
    env.add_descriptor(TmuxSessionDescriptor::new("dev-session"));
    env.create().unwrap();

    // Verify session exists
    assert!(env.tmux().session_exists("dev-session"));
}

#[test]
fn test_environment_with_multiple_descriptors() {
    let mut env = TestEnvironment::new();
    env.add_descriptor(DirectoryDescriptor::new("workspace"));
    env.add_descriptor(GitRepoDescriptor::new("repo1"));
    env.add_descriptor(GitRepoDescriptor::new("repo2"));
    env.add_descriptor(TmuxSessionDescriptor::new("session1"));
    env.create().unwrap();

    assert!(env.root_path().join("workspace").exists());
    assert!(env.root_path().join("repo1").exists());
    assert!(env.root_path().join("repo2").exists());
    assert!(env.tmux().session_exists("session1"));
}

#[test]
fn test_environment_with_complex_git_repo() {
    let mut env = TestEnvironment::new();

    let branch = BranchDescriptor::new("feature")
        .with_commit(CommitDescriptor::new("Add feature").with_file("feature.txt", "content"));

    let remote = RemoteDescriptor::new("origin");

    let repo = GitRepoDescriptor::new("complex-repo")
        .with_branch(branch)
        .with_remote(remote);

    env.add_descriptor(repo);
    env.create().unwrap();

    let repo_path = env.root_path().join("complex-repo");
    assert!(repo_path.exists());

    // Check that the branch exists
    let output = std::process::Command::new("git")
        .args(&["branch", "--list", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("feature"));
}

#[test]
fn test_environment_with_tmux_windows() {
    let mut env = TestEnvironment::new();

    let session = TmuxSessionDescriptor::new("multi-window")
        .with_window(WindowDescriptor::new("editor"))
        .with_window(WindowDescriptor::new("terminal"))
        .with_window(WindowDescriptor::new("server"));

    env.add_descriptor(session);
    env.create().unwrap();

    assert!(env.tmux().session_exists("multi-window"));

    // Check window count
    let output = env
        .tmux()
        .run_tmux(&["list-windows", "-t", "multi-window", "-F", "#{window_name}"])
        .unwrap();
    let window_count = output.lines().count();
    assert_eq!(window_count, 3);
}

#[test]
fn test_environment_cleanup_on_drop() {
    let root_path;
    let socket_name;

    {
        let mut env = TestEnvironment::new();
        root_path = env.root_path().to_path_buf();
        socket_name = env.tmux_socket().to_string();

        env.add_descriptor(DirectoryDescriptor::new("test-dir"));
        env.add_descriptor(TmuxSessionDescriptor::new("test-session"));
        env.create().unwrap();

        assert!(root_path.exists());
        assert!(env.tmux().session_exists("test-session"));
    } // env is dropped here

    // Verify temp dir is cleaned up
    assert!(!root_path.exists());

    // Verify tmux sessions are cleaned up
    let check = std::process::Command::new("tmux")
        .args(&["-L", &socket_name, "has-session", "-t", "test-session"])
        .output()
        .unwrap();
    assert!(!check.status.success());
}

#[test]
fn test_environment_get_context() {
    let env = TestEnvironment::new();
    let context = env.context();

    assert_eq!(context.root_path(), env.root_path());
    assert_eq!(context.tmux_socket().as_deref(), Some(env.tmux_socket()));
}

#[test]
fn test_environment_registry_tracking() {
    let mut env = TestEnvironment::new();

    env.add_descriptor(GitRepoDescriptor::new("tracked-repo"));
    env.add_descriptor(DirectoryDescriptor::new("tracked-dir"));
    env.add_descriptor(TmuxSessionDescriptor::new("tracked-session"));
    env.create().unwrap();

    let context = env.context();
    let registry = context.registry().borrow();

    // Check that resources are tracked
    assert!(registry.get_git_repo("tracked-repo").is_some());
    assert!(registry.get_tmux_session("tracked-session").is_some());
}

#[test]
fn test_environment_create_file_in_directory() {
    let mut env = TestEnvironment::new();
    env.add_descriptor(DirectoryDescriptor::new("workspace"));
    env.create().unwrap();

    // Create a file in the directory
    let file_path = env.root_path().join("workspace/test.txt");
    fs::write(&file_path, "test content").unwrap();

    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "test content");
}
