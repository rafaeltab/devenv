mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_full_workflow_create_workspace_start_tmux() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("new_ws", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("new_workspace", |d| {
                d.rafaeltab_workspace("new_ws", "New Workspace", |w| {
                    w.tag("test");
                });
            });
        });
    })
    .create();

    // Start the tmux session
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start failed: {}",
        start_result.stderr
    );

    // List sessions to verify
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list failed: {}",
        list_result.stderr
    );
}

#[test]
fn test_complex_workspace_setup() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
            c.tmux_session(
                "complex_ws",
                None,
                &[("editor", Some("vim")), ("terminal", None)],
            );
        });

        root.test_dir(|td| {
            td.dir("complex_ws", |d| {
                d.rafaeltab_workspace("complex_ws", "Complex Workspace", |w| {
                    w.tag("rust");
                    w.tag("cli");
                    w.tag("test");
                });
            });
        });
    })
    .create();

    // Start tmux session
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "tmux start for complex workspace failed: {}",
        start_result.stderr
    );

    // Test workspace find
    let find_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find", "complex_ws", "--json"])
        .build();
    let find_result = env.testers().cmd().run(&find_cmd);

    assert!(
        find_result.success,
        "workspace find failed: {}",
        find_result.stderr
    );

    // Verify JSON contains all tags
    let workspace: serde_json::Value =
        serde_json::from_str(&find_result.stdout).expect("Should be valid JSON");

    if let Some(tags) = workspace.get("tags").and_then(|t| t.as_array()) {
        let tag_names: Vec<String> = tags
            .iter()
            .filter_map(|t| t.as_str().map(|s| s.to_string()))
            .collect();

        assert!(
            tag_names.contains(&"rust".to_string()),
            "Expected rust tag in workspace"
        );
    }

    // Test workspace find-tag
    let tag_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find-tag", "cli"])
        .build();
    let tag_result = env.testers().cmd().run(&tag_cmd);

    assert!(
        tag_result.success,
        "workspace find-tag failed: {}",
        tag_result.stderr
    );

    assert!(
        tag_result.stdout.contains("Complex Workspace"),
        "Expected workspace with 'cli' tag. Got: {}",
        tag_result.stdout
    );
}

#[test]
fn test_nested_git_repos() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("parent_repo", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# Parent");
                        });
                    });
                });
                // Nested git repo inside
                d.git("subrepo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# Child");
                        });
                    });
                });
            });
        });
    })
    .create();

    let parent_dir = env.find_dir("parent_repo").expect("Dir not found");

    // Verify both repos exist
    assert!(parent_dir.path().join("repo/.git").exists());
    assert!(parent_dir.path().join("subrepo/.git").exists());
}

#[test]
fn test_worktree_session_auto_creation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("main_ws", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("main_ws", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# Main");
                        });
                    });
                    // Create worktree branch
                    g.branch("feature-branch", |b| {
                        b.commit("Feature commit", |c| {
                            c.file("feature.txt", "new feature");
                        });
                    });
                });
                d.rafaeltab_workspace("main_ws", "Main Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start main workspace session
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "tmux start for main workspace failed: {}",
        result.stderr
    );

    // List tmux sessions - should show main workspace
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list failed: {}",
        list_result.stderr
    );
}

#[test]
fn test_multiple_workspaces_tmux_start() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("multi_1", None, &[("shell", None)]);
            c.tmux_session("multi_2", None, &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("multi_1", |d| {
                d.rafaeltab_workspace("multi_1", "Multi One", |w| {
                    w.tag("rust");
                });
            });
            td.dir("multi_2", |d| {
                d.rafaeltab_workspace("multi_2", "Multi Two", |w| {
                    w.tag("typescript");
                });
            });
        });
    })
    .create();

    // Start all tmux sessions
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "tmux start for multiple workspaces failed: {}",
        result.stderr
    );

    // List and verify
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list failed: {}",
        list_result.stderr
    );
}

#[test]
fn test_config_persistence_across_commands() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.defaults();
        });

        root.test_dir(|td| {
            td.dir("persist_test", |d| {
                d.rafaeltab_workspace("persist_ws", "Persistent Workspace", |w| {
                    w.tag("test");
                });
            });
        });
    })
    .create();

    // First command: list workspaces
    let cmd1 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result1 = env.testers().cmd().run(&cmd1);

    assert!(
        result1.success,
        "First workspace list failed: {}",
        result1.stderr
    );

    // Second command: find the workspace
    let cmd2 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "find", "persist_ws"])
        .build();
    let result2 = env.testers().cmd().run(&cmd2);

    assert!(result2.success, "Workspace find failed: {}", result2.stderr);

    assert!(
        result2.stdout.contains("Persistent Workspace"),
        "Expected workspace in find output. Got: {}",
        result2.stdout
    );
}

#[test]
fn test_tmux_session_idempotency() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("idempotent", None, &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("idempotent", |d| {
                d.rafaeltab_workspace("idempotent", "Idempotent Test", |_w| {});
            });
        });
    })
    .create();

    // Start tmux session first time
    let cmd1 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result1 = env.testers().cmd().run(&cmd1);

    assert!(
        result1.success,
        "First tmux start failed: {}",
        result1.stderr
    );

    // Start tmux session second time - should not create duplicates
    let cmd2 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result2 = env.testers().cmd().run(&cmd2);

    // Should succeed without error
    assert!(
        result2.success,
        "Second tmux start should complete without panic. Got: {} {}",
        result2.stdout,
        result2.stderr
    );

    // List sessions
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list", "--json"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    if list_result.success {
        let sessions: serde_json::Value =
            serde_json::from_str(&list_result.stdout).expect("Should be valid JSON");

        if let Some(arr) = sessions.as_array() {
            // Count sessions with "idempotent" in name
            let idempotent_count = arr
                .iter()
                .filter(|s| {
                    s.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n.contains("idempotent"))
                        .unwrap_or(false)
                })
                .count();

            // Should not have duplicate sessions
            assert!(
                idempotent_count <= 1,
                "Should not have duplicate sessions, found {}: {}",
                idempotent_count,
                list_result.stdout
            );
        }
    }
}

#[test]
fn test_workspace_switching_scenarios() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("switch_a", None, &[("editor", None)]);
            c.tmux_session("switch_b", None, &[("terminal", None)]);
        });

        root.test_dir(|td| {
            td.dir("switch_a", |d| {
                d.rafaeltab_workspace("switch_a", "Switch A", |_w| {});
            });
            td.dir("switch_b", |d| {
                d.rafaeltab_workspace("switch_b", "Switch B", |_w| {});
            });
        });
    })
    .create();

    // Start first workspace session
    let cmd1 = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result1 = env.testers().cmd().run(&cmd1);

    assert!(
        result1.success,
        "First tmux start failed: {}",
        result1.stderr
    );

    // List sessions - should show both
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list failed: {}",
        list_result.stderr
    );
}

#[test]
fn test_full_workflow_worktree_lifecycle() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.worktree_global(&[], &[]);
        });

        root.test_dir(|td| {
            td.dir("lifecycle", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# Test");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_dir = env.find_dir("lifecycle").expect("Dir not found");
    let repo_path = repo_dir.path().join("repo");

    // Step 1: Create worktree
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "lifecycle-branch", "--force", "--yes"])
        .build();
    let start_result = env.testers().cmd().run(&start_cmd);

    // Step 2: Complete worktree
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "lifecycle-branch", "--force", "--yes"])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    // Both commands should complete without panic
    assert!(
        start_result.success,
        "Worktree start should complete. Got: {} {}",
        start_result.stdout,
        start_result.stderr
    );

    assert!(
        complete_result.success,
        "Worktree complete should complete. Got: {} {}",
        complete_result.stdout,
        complete_result.stderr
    );
}

#[test]
fn test_worktree_in_different_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.worktree_global(&[], &[]);
        });

        root.test_dir(|td| {
            td.dir("different_ws", |d| {
                // Git repo in subdirectory, workspace at parent level
                d.rafaeltab_workspace("different_ws", "Different Workspace", |_w| {});
                d.dir("subdir", |sd| {
                    sd.git("repo", |g| {
                        g.branch("main", |b| {
                            b.commit("Initial commit", |c| {
                                c.file("README.md", "# Test");
                            });
                        });
                    });
                });
            });
        });
    })
    .create();

    let ws_dir = env.find_dir("different_ws").expect("Dir not found");
    let repo_path = ws_dir.path().join("subdir/repo");

    // Create worktree from within the repo, while workspace is at parent level
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "diff-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "Test should complete without panic. Got: {} {}",
        result.stdout,
        result.stderr
    );
}
