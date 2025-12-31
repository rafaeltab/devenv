use tempfile::TempDir;
use test_descriptors::descriptor::CreateContext;

#[test]
fn test_create_context_new() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().to_path_buf();

    let context = CreateContext::new(path.clone());

    assert_eq!(context.root_path(), &path);
}

#[test]
fn test_register_and_get_resource() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let resource_path = temp.path().join("repo");
    context.register_resource("repo-0".to_string(), resource_path.clone());

    let retrieved = context.get_resource("repo-0");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), resource_path);
}

#[test]
fn test_get_nonexistent_resource_returns_none() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let result = context.get_resource("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_set_and_get_tmux_socket() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    context.set_tmux_socket("test-socket".to_string());

    let socket = context.tmux_socket();
    assert!(socket.is_some());
    assert_eq!(socket.unwrap(), "test-socket");
}

#[test]
fn test_set_and_get_config_path() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let config_path = temp.path().join("config.json");
    context.set_config_path(config_path.clone());

    let retrieved = context.config_path();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), config_path);
}

#[test]
fn test_multiple_resource_registration() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let repo1 = temp.path().join("repo1");
    let repo2 = temp.path().join("repo2");
    let repo3 = temp.path().join("repo3");

    context.register_resource("repo-1".to_string(), repo1.clone());
    context.register_resource("repo-2".to_string(), repo2.clone());
    context.register_resource("repo-3".to_string(), repo3.clone());

    assert_eq!(context.get_resource("repo-1").unwrap(), repo1);
    assert_eq!(context.get_resource("repo-2").unwrap(), repo2);
    assert_eq!(context.get_resource("repo-3").unwrap(), repo3);
}
