mod common;

use common::rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabRootMixin};
use std::fs;
use test_descriptors::TestEnvironment;

#[test]
fn test_workspace_descriptor_creates_directory() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("workspace_one", "My Workspace", |_w| {});
            });
        });
    })
    .create();

    assert!(env.root_path().join("ws1").exists());
    assert!(env.root_path().join("ws1").is_dir());
}

#[test]
fn test_workspace_descriptor_with_tags() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("workspace_one", "My Workspace", |w| {
                    w.tag("rust");
                    w.tag("cli");
                });
            });
        });
    })
    .create();

    assert!(env.root_path().join("ws1").exists());
}

#[test]
fn test_config_descriptor_creates_file() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|_td| {});
    })
    .create();

    let config_path = env.context().config_path();
    assert!(config_path.is_some());

    let config_file = config_path.unwrap();
    assert!(config_file.exists());
}

#[test]
fn test_config_descriptor_with_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("workspace_one", "My Workspace", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();

    // Parse JSON and verify workspace is included
    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert!(json.get("workspaces").is_some());

    let workspaces = json["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0]["id"], "workspace_one");
    assert_eq!(workspaces[0]["name"], "My Workspace");
}

#[test]
fn test_config_descriptor_with_multiple_workspaces() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws1", |d| {
                d.rafaeltab_workspace("workspace_one", "Workspace One", |_w| {});
            });
            td.dir("ws2", |d| {
                d.rafaeltab_workspace("workspace_two", "Workspace Two", |w| {
                    w.tag("rust");
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();

    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    let workspaces = json["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 2);
}

#[test]
fn test_config_descriptor_valid_json_schema() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("test", |d| {
                d.rafaeltab_workspace("test_ws", "Test Workspace", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();

    // Verify it's valid JSON
    let result: Result<serde_json::Value, _> = serde_json::from_str(&config_content);
    assert!(result.is_ok());

    let json = result.unwrap();

    // Verify required fields exist
    assert!(json.get("workspaces").is_some());
    // New API uses "tmux" with "sessions" inside, not "tmuxSessions"
    assert!(json.get("tmux").is_some());
}

#[test]
fn test_workspace_with_worktree_config() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws", |d| {
                d.rafaeltab_workspace("ws_with_worktree", "Worktree Workspace", |w| {
                    w.worktree(&["npm install"], &[".env"]);
                });
            });
        });
    })
    .create();

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
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|_td| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let config_content = fs::read_to_string(&config_path).unwrap();

    let json: serde_json::Value = serde_json::from_str(&config_content).unwrap();

    // Should have default windows in tmux config
    assert!(json["tmux"]["defaultWindows"].is_array());
}
