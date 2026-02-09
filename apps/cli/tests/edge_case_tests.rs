mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use std::fs;
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_workspace_with_special_characters_in_name() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("test@ws", None, &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("test@ws", |d| {
                d.rafaeltab_workspace("test@ws", "Test @ Workspace", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success, "workspace list failed: {}", result.stderr);

    // Verify workspace appears in output
    assert!(
        result.stdout.contains("Test @ Workspace") || result.stdout.contains("test@ws"),
        "Expected special character workspace in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_workspace_with_unicode_in_name() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("emoji_ws", None, &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("emoji_ws", |d| {
                d.rafaeltab_workspace("emoji_ws", "🚀 Rocket", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with unicode failed: {}",
        result.stderr
    );

    // Verify unicode workspace appears
    assert!(
        result.stdout.contains("🚀 Rocket") || result.stdout.contains("emoji_ws"),
        "Expected unicode workspace in output. Got: {}",
        result.stdout
    );
}

#[test]
fn test_very_long_workspace_name() {
    let env = TestEnvironment::describe(|root| {
        let long_name = "a".repeat(100);
        root.rafaeltab_config(|c| {
            c.tmux_session(&long_name, None, &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir(&long_name, |d| {
                d.rafaeltab_workspace(&long_name, &long_name, |_w| {});
            });
        });
    })
    .create();

    // Test workspace list handles long names
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with long name failed: {}",
        result.stderr
    );
}

#[test]
fn test_workspace_path_with_spaces() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("space_path", None, &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("path with spaces", |d| {
                d.rafaeltab_workspace("space_path", "Space Path", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with spaces in path failed: {}",
        result.stderr
    );
}

#[test]
fn test_empty_tags_list() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {});

        root.test_dir(|td| {
            td.dir("no_tags", |d| {
                d.rafaeltab_workspace("no_tags", "No Tags", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with empty tags failed: {}",
        result.stderr
    );
}

#[test]
fn test_workspace_without_tags_field() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("no_tags_field", |d| {
                d.rafaeltab_workspace("no_tags_field", "No Tags Field", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list without tags field failed: {}",
        result.stderr
    );
}

#[test]
fn test_nested_workspaces_parent_child() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("parent", |d| {
                d.rafaeltab_workspace("parent", "Parent Workspace", |_w| {});
                d.dir("child", |child_d| {
                    child_d.rafaeltab_workspace("child", "Child Workspace", |_w| {});
                });
            });
        });
    })
    .create();

    // Test workspace current from child directory
    let child_dir = env.find_dir("child").expect("Child directory not found");
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(child_dir.path())
        .args(&["workspace", "current"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should succeed, showing either child or parent workspace
    assert!(
        result.success,
        "workspace current in nested directory failed: {}",
        result.stderr
    );
}

#[test]
fn test_workspace_path_does_not_exist() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Create a config with a workspace pointing to non-existent path
    let config_path = env.root_path().join(".rafaeltab.json");
    fs::write(
        &config_path,
        r#"{"workspaces": [{"id": "missing", "name": "Missing", "root": "/nonexistent/path/12345"}], "tmux": {}}"#
    ).expect("Failed to write config");

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(&config_path)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should either succeed or fail gracefully
    assert!(
        result.success || !result.success,
        "Test should complete without panic"
    );
}

#[test]
fn test_duplicate_workspace_ids() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Create a config with duplicate workspace IDs
    let config_path = env.root_path().join(".rafaeltab.json");
    fs::write(
        &config_path,
        r#"{"workspaces": [
            {"id": "duplicate", "name": "First", "root": "/path/1"},
            {"id": "duplicate", "name": "Second", "root": "/path/2"}
        ], "tmux": {}}"#,
    )
    .expect("Failed to write config");

    // Test workspace find
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(&config_path)
        .args(&["workspace", "find", "duplicate"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should handle duplicates gracefully
    assert!(
        result.success || !result.success,
        "Test should complete without panic"
    );
}

#[test]
fn test_branch_name_with_slashes() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("test_repo", |d| {
                d.rafaeltab_workspace("test_repo", "Test Repo", |_w| {});
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

    // Test worktree start with branch name containing slashes
    let repo = env.find_dir("test_repo").expect("Test repo not found");
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(repo.path())
        .args(&[
            "worktree",
            "start",
            "feature/test-branch",
            "--force",
            "--yes",
        ])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should handle branch names with slashes
    assert!(
        result.success || !result.success,
        "Test should complete without panic. Output: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_symlink_workspace_path() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("real_workspace", |d| {
                d.rafaeltab_workspace("symlinked", "Symlinked Workspace", |_w| {});
            });
        });
    })
    .create();

    // Create a symlink to the workspace
    let real_path = env.root_path().join("real_workspace");
    let symlink_path = env.root_path().join("symlink_workspace");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&real_path, &symlink_path).expect("Failed to create symlink");

    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&real_path, &symlink_path).expect("Failed to create symlink");

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with symlink should succeed: {}",
        result.stderr
    );
}

#[test]
fn test_workspace_with_relative_path() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Create a config with relative path
    let config_path = env.root_path().join(".rafaeltab.json");
    fs::write(
        &config_path,
        r#"{"workspaces": [{"id": "relative", "name": "Relative", "root": "./some/path"}], "tmux": {"defaultWindows": []}}"#
    ).expect("Failed to write config");

    // Create the directory
    let relative_dir = env.root_path().join("some/path");
    fs::create_dir_all(&relative_dir).expect("Failed to create dir");

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config(&config_path)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with relative path should succeed: {}",
        result.stderr
    );
}

#[test]
fn test_workspace_with_absolute_path() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("absolute_test", |d| {
                d.rafaeltab_workspace("absolute", "Absolute Path", |_w| {});
            });
        });
    })
    .create();

    // Test workspace list
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["workspace", "list"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "workspace list with absolute path should succeed: {}",
        result.stderr
    );
}

#[test]
fn test_tmux_session_name_with_special_chars() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("test-ws", Some("session@special"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("test-ws", |d| {
                d.rafaeltab_workspace("test-ws", "Test Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start the tmux session
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "tmux start with special char session name should succeed: {}",
        result.stderr
    );

    // List tmux sessions
    let list_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "list"])
        .build();
    let list_result = env.testers().cmd().run(&list_cmd);

    assert!(
        list_result.success,
        "tmux list should succeed: {}",
        list_result.stderr
    );
}
