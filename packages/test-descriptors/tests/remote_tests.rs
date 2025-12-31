use std::process::Command;
use tempfile::TempDir;
use test_descriptors::descriptor::{CreateContext, RemoteDescriptor};

#[test]
fn test_remote_descriptor_creates_bare_repo() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let remote = RemoteDescriptor::new("origin");
    let bare_path = remote.create_bare_repo(&context).unwrap();

    assert!(bare_path.exists());
    assert!(bare_path.join("HEAD").exists());
    assert!(bare_path.join("refs").exists());
    assert!(bare_path.join("objects").exists());
}

#[test]
fn test_remote_descriptor_bare_repo_is_valid() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let remote = RemoteDescriptor::new("origin");
    let bare_path = remote.create_bare_repo(&context).unwrap();

    // Verify it's a bare repository
    let output = Command::new("git")
        .args(&["config", "core.bare"])
        .current_dir(&bare_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let is_bare = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(is_bare, "true");
}

#[test]
fn test_remote_descriptor_path_is_isolated() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let remote1 = RemoteDescriptor::new("origin");
    let remote2 = RemoteDescriptor::new("upstream");

    let path1 = remote1.create_bare_repo(&context).unwrap();
    let path2 = remote2.create_bare_repo(&context).unwrap();

    // Each remote should have a unique path
    assert_ne!(path1, path2);
    assert!(path1.exists());
    assert!(path2.exists());
}

#[test]
fn test_remote_descriptor_name() {
    let remote = RemoteDescriptor::new("my-remote");
    assert_eq!(remote.name(), "my-remote");
}
