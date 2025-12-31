use std::fs;
use std::process::Command;
use tempfile::TempDir;
use test_descriptors::descriptor::{
    BranchDescriptor, CommitDescriptor, CreateContext, Descriptor, GitRepoDescriptor,
    PathDescriptor, RemoteDescriptor,
};

#[test]
fn test_git_repo_descriptor_creates_repo() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("test-repo");
    repo.create(&context).unwrap();

    let repo_path = temp.path().join("test-repo");
    assert!(repo_path.exists());
    assert!(repo_path.join(".git").exists());
}

#[test]
fn test_git_repo_descriptor_has_initial_commit() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("test-repo");
    repo.create(&context).unwrap();

    let repo_path = temp.path().join("test-repo");
    let output = Command::new("git")
        .args(&["log", "--oneline"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Initial commit"));
}

#[test]
fn test_git_repo_descriptor_on_main_branch() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("test-repo");
    repo.create(&context).unwrap();

    let repo_path = temp.path().join("test-repo");
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(branch, "main");
}

#[test]
fn test_git_repo_descriptor_path_resolution() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("my-repo");
    let path = repo.path(&context);

    assert_eq!(path, temp.path().join("my-repo"));
}

#[test]
fn test_git_repo_descriptor_registered_in_context() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("registered-repo");
    repo.create(&context).unwrap();

    let binding = context.registry().borrow();
    let retrieved = binding.get_git_repo("registered-repo");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &temp.path().join("registered-repo"));
}

#[test]
fn test_git_repo_descriptor_has_git_config() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("test-repo");
    repo.create(&context).unwrap();

    let repo_path = temp.path().join("test-repo");

    // Check user.name
    let output = Command::new("git")
        .args(&["config", "user.name"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(name, "Test User");

    // Check user.email
    let output = Command::new("git")
        .args(&["config", "user.email"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let email = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(email, "test@example.com");
}

#[test]
fn test_git_repo_descriptor_has_readme() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("test-repo");
    repo.create(&context).unwrap();

    let readme_path = temp.path().join("test-repo/README.md");
    assert!(readme_path.exists());

    let content = fs::read_to_string(&readme_path).unwrap();
    assert!(content.contains("test-repo"));
}

#[test]
fn test_git_repo_descriptor_creates_parent_directories() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("parent/child/my-repo");
    repo.create(&context).unwrap();

    let repo_path = temp.path().join("parent/child/my-repo");
    assert!(repo_path.exists());
    assert!(repo_path.join(".git").exists());
}

#[test]
fn test_git_repo_descriptor_with_branch() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let branch = BranchDescriptor::new("feature")
        .with_commit(CommitDescriptor::new("Feature commit").with_file("feature.txt", "content"));

    let repo = GitRepoDescriptor::new("test-repo").with_branch(branch);
    repo.create(&context).unwrap();

    let repo_path = temp.path().join("test-repo");

    // Check that feature branch exists
    let output = Command::new("git")
        .args(&["branch", "--list", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("feature"));

    // Check that feature.txt exists on feature branch
    Command::new("git")
        .args(&["checkout", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(repo_path.join("feature.txt").exists());
}

#[test]
fn test_git_repo_descriptor_with_remote() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let remote = RemoteDescriptor::new("origin");
    let repo = GitRepoDescriptor::new("test-repo").with_remote(remote);
    repo.create(&context).unwrap();

    let repo_path = temp.path().join("test-repo");

    // Check that remote exists
    let output = Command::new("git")
        .args(&["remote", "-v"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let remotes = String::from_utf8_lossy(&output.stdout);
    assert!(remotes.contains("origin"));
}

#[test]
fn test_git_repo_descriptor_custom_initial_branch() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo = GitRepoDescriptor::new("test-repo").with_initial_branch("master");
    repo.create(&context).unwrap();

    let repo_path = temp.path().join("test-repo");
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(branch, "master");
}
