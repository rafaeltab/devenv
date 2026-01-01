mod common;

use common::rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabGitMixin, RafaeltabRootMixin};
use std::fs;
use test_descriptors::TestEnvironment;

#[test]
fn test_rafaeltab_config_creates_file() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|_td| {
            // Empty test dir
        });
    })
    .create();

    let config_path = env.context().config_path();
    assert!(config_path.is_some(), "Config path should be set");

    let config_file = config_path.unwrap();
    assert!(config_file.exists(), "Config file should exist");

    let content = fs::read_to_string(&config_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Should have workspaces array (empty)
    assert!(json.get("workspaces").is_some());
    assert!(json["workspaces"].is_array());

    // Should have tmux config with defaults
    assert!(json.get("tmux").is_some());
    assert!(json["tmux"]["defaultWindows"].is_array());
}

#[test]
fn test_rafaeltab_config_with_worktree_global() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
            c.worktree_global(&["pnpm install"], &["**/.env"]);
        });

        root.test_dir(|_td| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let content = fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Should have worktree config
    assert!(json.get("worktree").is_some());
    assert_eq!(json["worktree"]["onCreate"][0], "pnpm install");
    assert_eq!(json["worktree"]["symlinkFiles"][0], "**/.env");
}

#[test]
fn test_workspace_inside_dir() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.rafaeltab_workspace("my_workspace", "My Workspace", |w| {
                    w.tag("rust");
                    w.tag("cli");
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let content = fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let workspaces = json["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0]["id"], "my_workspace");
    assert_eq!(workspaces[0]["name"], "My Workspace");
    assert_eq!(workspaces[0]["tags"][0], "rust");
    assert_eq!(workspaces[0]["tags"][1], "cli");

    // Path should be the directory path
    let path = workspaces[0]["root"].as_str().unwrap();
    assert!(path.ends_with("workspace"));
}

#[test]
fn test_workspace_inside_git() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("projects", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Test");
                        });
                    });

                    g.rafaeltab_workspace("my_project", "My Project", |w| {
                        w.tag("rust");
                    });
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let content = fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let workspaces = json["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0]["id"], "my_project");
    assert_eq!(workspaces[0]["name"], "My Project");

    // Path should be the git repo path
    let path = workspaces[0]["root"].as_str().unwrap();
    assert!(path.ends_with("my-repo"));
}

#[test]
fn test_workspace_with_worktree_config() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("projects", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("package.json", "{}");
                        });
                    });

                    g.rafaeltab_workspace("my_project", "My Project", |w| {
                        w.worktree(&["npm install", "npm run build"], &[".env", "node_modules"]);
                    });
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let content = fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let workspace = &json["workspaces"][0];
    assert!(workspace.get("worktree").is_some());
    assert_eq!(workspace["worktree"]["onCreate"][0], "npm install");
    assert_eq!(workspace["worktree"]["onCreate"][1], "npm run build");
    assert_eq!(workspace["worktree"]["symlinkFiles"][0], ".env");
    assert_eq!(workspace["worktree"]["symlinkFiles"][1], "node_modules");
}

#[test]
fn test_multiple_workspaces() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("projects", |d| {
                d.git("repo-a", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# A");
                        });
                    });
                    g.rafaeltab_workspace("project_a", "Project A", |w| {
                        w.tag("rust");
                    });
                });

                d.git("repo-b", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# B");
                        });
                    });
                    g.rafaeltab_workspace("project_b", "Project B", |w| {
                        w.tag("node");
                    });
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let content = fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let workspaces = json["workspaces"].as_array().unwrap();
    assert_eq!(workspaces.len(), 2);

    // Check both workspaces exist
    let ids: Vec<&str> = workspaces
        .iter()
        .map(|w| w["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"project_a"));
    assert!(ids.contains(&"project_b"));
}

#[test]
fn test_config_with_custom_default_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window_with_command("editor", "vim .");
            c.default_window("terminal");
        });

        root.test_dir(|_td| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let content = fs::read_to_string(&config_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let windows = json["tmux"]["defaultWindows"].as_array().unwrap();
    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0]["name"], "editor");
    assert_eq!(windows[0]["command"], "vim .");
    assert_eq!(windows[1]["name"], "terminal");
    assert!(windows[1].get("command").is_none() || windows[1]["command"].is_null());
}
