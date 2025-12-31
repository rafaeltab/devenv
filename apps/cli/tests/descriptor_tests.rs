mod common;

use common::descriptors::{ConfigDescriptor, WorkspaceDescriptor};
use std::fs;
use test_descriptors::TestEnvironment;

#[test]
fn test_workspace_descriptor_creates_directory() {
    let mut env = TestEnvironment::new();
    
    let workspace = WorkspaceDescriptor::new("workspace-1", "My Workspace", env.root_path().join("ws1"));
    env.add_descriptor(workspace);
    env.create().unwrap();

    assert!(env.root_path().join("ws1").exists());
    assert!(env.root_path().join("ws1").is_dir());
}

#[test]
fn test_workspace_descriptor_with_tags() {
    let mut env = TestEnvironment::new();
    
    let workspace = WorkspaceDescriptor::new("workspace-1", "My Workspace", env.root_path().join("ws1"))
        .with_tag("rust")
        .with_tag("cli");
    
    env.add_descriptor(workspace);
    env.create().unwrap();

    assert!(env.root_path().join("ws1").exists());
}

#[test]
fn test_config_descriptor_creates_file() {
    let mut env = TestEnvironment::new();
    
    let config = ConfigDescriptor::new();
    env.add_descriptor(config);
    env.create().unwrap();

    let config_path = env.context().config_path();
    assert!(config_path.is_some());
    
    let config_file = config_path.unwrap();
    assert!(config_file.exists());
}

#[test]
fn test_config_descriptor_with_workspace() {
    let mut env = TestEnvironment::new();
    
    let workspace = WorkspaceDescriptor::new("workspace-1", "My Workspace", env.root_path().join("ws1"));
    let config = ConfigDescriptor::new();
    
    env.add_descriptor(workspace);
    env.add_descriptor(config);
    env.create().unwrap();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();
    
    // Parse JSON and verify workspace is included
    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert!(json.get("workspaces").is_some());
    
    let workspaces = json["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0]["id"], "workspace-1");
    assert_eq!(workspaces[0]["name"], "My Workspace");
}

#[test]
fn test_config_descriptor_with_multiple_workspaces() {
    let mut env = TestEnvironment::new();
    
    let ws1 = WorkspaceDescriptor::new("workspace-1", "Workspace One", env.root_path().join("ws1"));
    let ws2 = WorkspaceDescriptor::new("workspace-2", "Workspace Two", env.root_path().join("ws2"))
        .with_tag("rust");
    let config = ConfigDescriptor::new();
    
    env.add_descriptor(ws1);
    env.add_descriptor(ws2);
    env.add_descriptor(config);
    env.create().unwrap();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();
    
    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    let workspaces = json["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 2);
}

#[test]
fn test_config_descriptor_valid_json_schema() {
    let mut env = TestEnvironment::new();
    
    let workspace = WorkspaceDescriptor::new("test-ws", "Test Workspace", env.root_path().join("test"));
    let config = ConfigDescriptor::new();
    
    env.add_descriptor(workspace);
    env.add_descriptor(config);
    env.create().unwrap();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();
    
    // Verify it's valid JSON
    let result: Result<serde_json::Value, _> = serde_json::from_str(&config_content);
    assert!(result.is_ok());
    
    let json = result.unwrap();
    
    // Verify required fields exist
    assert!(json.get("workspaces").is_some());
    assert!(json.get("tmuxSessions").is_some());
}

#[test]
fn test_workspace_with_worktree_config() {
    let mut env = TestEnvironment::new();
    
    let workspace = WorkspaceDescriptor::new("ws-with-worktree", "Worktree Workspace", env.root_path().join("ws"))
        .with_worktree_config(vec!["npm install".to_string()], vec![".env".to_string()]);
    
    env.add_descriptor(workspace);
    let config = ConfigDescriptor::new();
    env.add_descriptor(config);
    env.create().unwrap();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();
    
    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    let workspace = &json["workspaces"][0];
    
    assert!(workspace.get("worktree").is_some());
    assert_eq!(workspace["worktree"]["onCreate"][0], "npm install");
    assert_eq!(workspace["worktree"]["symlinkFiles"][0], ".env");
}

#[test]
fn test_config_descriptor_default_windows() {
    let mut env = TestEnvironment::new();
    
    let config = ConfigDescriptor::new();
    env.add_descriptor(config);
    env.create().unwrap();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();
    
    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    
    // Should have default empty arrays
    assert!(json["defaultWindows"].is_array());
}
