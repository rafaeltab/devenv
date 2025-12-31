use std::path::PathBuf;
use test_descriptors::descriptor::{ResourceRegistry, TmuxSessionInfo};

#[test]
fn test_register_and_get_git_repo() {
    let mut registry = ResourceRegistry::new();
    let path = PathBuf::from("/tmp/repo");

    registry.register_git_repo("repo-0".to_string(), path.clone());

    let retrieved = registry.get_git_repo("repo-0");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &path);
}

#[test]
fn test_register_and_get_worktree() {
    let mut registry = ResourceRegistry::new();
    let path = PathBuf::from("/tmp/worktree");

    registry.register_worktree(
        "repo-0".to_string(),
        "feature/test".to_string(),
        path.clone(),
    );

    let retrieved = registry.get_worktree("repo-0", "feature/test");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &path);
}

#[test]
fn test_register_and_get_tmux_session() {
    let mut registry = ResourceRegistry::new();
    let info = TmuxSessionInfo {
        name: "session-0".to_string(),
        working_dir: PathBuf::from("/tmp/work"),
    };

    registry.register_tmux_session("session-0".to_string(), info.clone());

    let retrieved = registry.get_tmux_session("session-0");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "session-0");
    assert_eq!(retrieved.unwrap().working_dir, PathBuf::from("/tmp/work"));
}

#[test]
fn test_register_and_get_dir() {
    let mut registry = ResourceRegistry::new();
    let path = PathBuf::from("/tmp/mydir");

    registry.register_dir("main".to_string(), path.clone());

    let retrieved = registry.get_dir("main");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &path);
}

#[test]
fn test_get_nonexistent_resources_return_none() {
    let registry = ResourceRegistry::new();

    assert!(registry.get_git_repo("nonexistent").is_none());
    assert!(registry.get_worktree("nonexistent", "branch").is_none());
    assert!(registry.get_tmux_session("nonexistent").is_none());
    assert!(registry.get_dir("nonexistent").is_none());
}

#[test]
fn test_overwrite_existing_resource() {
    let mut registry = ResourceRegistry::new();

    let path1 = PathBuf::from("/tmp/repo1");
    let path2 = PathBuf::from("/tmp/repo2");

    registry.register_git_repo("repo".to_string(), path1);
    registry.register_git_repo("repo".to_string(), path2.clone());

    let retrieved = registry.get_git_repo("repo");
    assert_eq!(retrieved.unwrap(), &path2);
}
