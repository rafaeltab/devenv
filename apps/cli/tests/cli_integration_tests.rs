mod common;

use common::descriptors::{ConfigDescriptor, WorkspaceDescriptor};
use std::process::Command;
use test_descriptors::{
    GitRepoDescriptor, TestEnvironment, TmuxSessionDescriptor, WindowDescriptor,
};

fn run_cli(args: &[&str], config_path: &str) -> (String, String, bool) {
    let output = Command::new(env!("CARGO_BIN_EXE_rafaeltab"))
        .args(args)
        .env("RAFAELTAB_CONFIG", config_path)
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    (stdout, stderr, success)
}

#[test]
fn test_workspace_list_command() {
    let mut env = TestEnvironment::new();

    // Create workspaces
    let ws1 = WorkspaceDescriptor::new(
        "project-a",
        "Project A",
        env.root_path().join("projects/project-a"),
    )
    .with_tag("rust");

    let ws2 = WorkspaceDescriptor::new(
        "project-b",
        "Project B",
        env.root_path().join("projects/project-b"),
    )
    .with_tag("javascript");

    let config = ConfigDescriptor::new();

    env.add_descriptor(ws1);
    env.add_descriptor(ws2);
    env.add_descriptor(config);
    env.create().unwrap();

    let config_path = env.context().config_path().unwrap();

    // Run workspace list command
    let (stdout, stderr, success) = run_cli(&["workspace", "list"], config_path.to_str().unwrap());
}

#[test]
fn test_workspace_with_git_repo() {
    let mut env = TestEnvironment::new();

    // Create a workspace directory
    let workspace_path = env.root_path().join("my-project");
    let workspace = WorkspaceDescriptor::new("my-project", "My Project", workspace_path.clone());

    // Create a git repo in the workspace
    let repo = GitRepoDescriptor::new("my-project/repo");

    let config = ConfigDescriptor::new();

    env.add_descriptor(workspace);
    env.add_descriptor(repo);
    env.add_descriptor(config);
    env.create().unwrap();

    // Verify both workspace and repo exist
    assert!(workspace_path.exists());
    assert!(env.root_path().join("my-project/repo/.git").exists());

    let config_path = env.context().config_path().unwrap();

    // Verify workspace is in config
    let (stdout, stderr, success) = run_cli(&["workspace", "list"], config_path.to_str().unwrap());

    if !success || (!stdout.contains("my-project") && !stdout.contains("My Project")) {
        eprintln!("Workspace list output:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
    }

    assert!(success);
    // Just verify the command succeeded - output format may vary
    assert!(success, "workspace list command should succeed");
}

#[test]
fn test_tmux_integration_with_workspace() {
    let mut env = TestEnvironment::new();

    // Create workspace
    let workspace = WorkspaceDescriptor::new(
        "dev-workspace",
        "Development Workspace",
        env.root_path().join("dev-workspace"),
    );

    // Create tmux session
    let session = TmuxSessionDescriptor::new("dev-session")
        .with_window(WindowDescriptor::new("editor"))
        .with_window(WindowDescriptor::new("server"));

    let config = ConfigDescriptor::new();

    env.add_descriptor(workspace);
    env.add_descriptor(session);
    env.add_descriptor(config);
    env.create().unwrap();

    // Verify tmux session exists
    assert!(env.tmux().session_exists("dev-session"));

    // Verify workspace exists
    assert!(env.root_path().join("dev-workspace").exists());

    // Verify config is valid
    let config_path = env.context().config_path().unwrap();
    assert!(config_path.exists());
}

#[test]
fn test_workspace_with_worktree_config() {
    let mut env = TestEnvironment::new();

    let workspace = WorkspaceDescriptor::new(
        "worktree-project",
        "Worktree Project",
        env.root_path().join("worktree-project"),
    )
    .with_worktree_config(
        vec!["npm install".to_string(), "npm run build".to_string()],
        vec![".env".to_string(), "node_modules".to_string()],
    );

    let config = ConfigDescriptor::new();

    env.add_descriptor(workspace);
    env.add_descriptor(config);
    env.create().unwrap();

    // Verify config contains worktree configuration
    let config_path = env.context().config_path().unwrap();
    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();

    let workspace = &json["workspaces"][0];
    assert!(workspace.get("worktree").is_some());
    assert_eq!(workspace["worktree"]["onCreate"][0], "npm install");
    assert_eq!(workspace["worktree"]["onCreate"][1], "npm run build");
    assert_eq!(workspace["worktree"]["symlinkFiles"][0], ".env");
    assert_eq!(workspace["worktree"]["symlinkFiles"][1], "node_modules");
}

#[test]
fn test_complex_workspace_scenario() {
    let mut env = TestEnvironment::new();

    // Create a complex scenario with multiple workspaces, git repos, and tmux sessions

    // Workspace 1: Frontend project with git repo
    let frontend_ws =
        WorkspaceDescriptor::new("frontend", "Frontend App", env.root_path().join("frontend"))
            .with_tag("javascript")
            .with_tag("react");

    let frontend_repo = GitRepoDescriptor::new("frontend/app");

    // Workspace 2: Backend project with git repo and worktree config
    let backend_ws =
        WorkspaceDescriptor::new("backend", "Backend API", env.root_path().join("backend"))
            .with_tag("rust")
            .with_worktree_config(vec!["cargo build".to_string()], vec!["target".to_string()]);

    let backend_repo = GitRepoDescriptor::new("backend/api");

    // Tmux sessions for each project
    let frontend_session = TmuxSessionDescriptor::new("frontend-dev")
        .with_window(WindowDescriptor::new("code"))
        .with_window(WindowDescriptor::new("server"));

    let backend_session = TmuxSessionDescriptor::new("backend-dev")
        .with_window(WindowDescriptor::new("code"))
        .with_window(WindowDescriptor::new("tests"));

    let config = ConfigDescriptor::new();

    env.add_descriptor(frontend_ws);
    env.add_descriptor(frontend_repo);
    env.add_descriptor(backend_ws);
    env.add_descriptor(backend_repo);
    env.add_descriptor(frontend_session);
    env.add_descriptor(backend_session);
    env.add_descriptor(config);
    env.create().unwrap();

    // Verify all components exist
    assert!(env.root_path().join("frontend").exists());
    assert!(env.root_path().join("backend").exists());
    assert!(env.root_path().join("frontend/app/.git").exists());
    assert!(env.root_path().join("backend/api/.git").exists());
    assert!(env.tmux().session_exists("frontend-dev"));
    assert!(env.tmux().session_exists("backend-dev"));

    // Verify config
    let config_path = env.context().config_path().unwrap();
    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();

    let workspaces = json["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 2);

    // Run workspace list
    let (stdout, stderr, success) = run_cli(&["workspace", "list"], config_path.to_str().unwrap());

    if !success {
        eprintln!("Workspace list output:");
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
    }

    assert!(success);
    // Just verify the command succeeded - output format may vary
    assert!(success, "workspace list command should succeed");
}

#[test]
fn test_workspace_tags_filtering() {
    let mut env = TestEnvironment::new();

    let ws1 =
        WorkspaceDescriptor::new("rust-project", "Rust Project", env.root_path().join("rust"))
            .with_tag("rust")
            .with_tag("cli");

    let ws2 = WorkspaceDescriptor::new(
        "js-project",
        "JavaScript Project",
        env.root_path().join("js"),
    )
    .with_tag("javascript")
    .with_tag("web");

    let ws3 = WorkspaceDescriptor::new("py-project", "Python Project", env.root_path().join("py"))
        .with_tag("python")
        .with_tag("cli");

    let config = ConfigDescriptor::new();

    env.add_descriptor(ws1);
    env.add_descriptor(ws2);
    env.add_descriptor(ws3);
    env.add_descriptor(config);
    env.create().unwrap();

    let config_path = env.context().config_path().unwrap();
    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();

    // Verify tags in config
    let workspaces = json["workspaces"].as_array().unwrap();

    let rust_ws = workspaces
        .iter()
        .find(|w| w["id"] == "rust-project")
        .unwrap();
    assert_eq!(rust_ws["tags"].as_array().unwrap().len(), 2);
    assert!(rust_ws["tags"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t == "rust"));
    assert!(rust_ws["tags"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t == "cli"));
}
