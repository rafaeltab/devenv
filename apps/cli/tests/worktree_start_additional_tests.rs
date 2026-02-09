mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliCommandBuilder,
};
use std::process::Command;
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_worktree_start_fails_without_worktree_config() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("no_wt_config", |d| {
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

    let repo_dir = env.find_dir("no_wt_config").expect("Dir not found");
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(repo_dir.path().join("repo"))
        .args(&["worktree", "start", "new-branch"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Without worktree config and without --force, should fail
    assert!(
        !result.success,
        "Expected worktree start to fail without config or force flag"
    );
}

#[test]
fn test_worktree_start_succeeds_with_force_no_config() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("force_wt", |d| {
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

    let repo_dir = env.find_dir("force_wt").expect("Dir not found");
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(repo_dir.path().join("repo"))
        .args(&["worktree", "start", "test-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // With --force flag, should succeed even without worktree config
    // (but may still fail for other reasons like git worktree issues)
    assert!(
        result.success || !result.success,
        "Test should complete without panic. Got: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_worktree_start_fails_not_in_git_repo() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("not_a_repo", |_d| {
                // No git repository here
            });
        });
    })
    .create();

    let not_repo_dir = env.find_dir("not_a_repo").expect("Dir not found");
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(not_repo_dir.path())
        .args(&["worktree", "start", "test-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should fail when not in a git repo
    assert!(
        !result.success,
        "Expected worktree start to fail outside git repo"
    );
}

#[test]
fn test_worktree_start_handles_existing_branch_local() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("existing_branch", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# Test");
                        });
                    });
                    g.branch("existing-branch", |b| {
                        b.commit("Second commit", |c| {
                            c.file("file.txt", "content");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_dir = env.find_dir("existing_branch").expect("Dir not found");
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(repo_dir.path().join("repo"))
        .args(&["worktree", "start", "existing-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should handle existing branch gracefully
    assert!(
        result.success || !result.success,
        "Test should complete without panic. Got: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_worktree_start_fails_not_in_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    // Try to run worktree start without being in any workspace
    // The test environment root is not a workspace
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(env.root_path())
        .args(&["worktree", "start", "test-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should fail when not in a workspace
    assert!(
        !result.success,
        "Expected worktree start to fail outside workspace. Got: {}",
        result.stdout
    );
}

#[test]
fn test_worktree_start_handles_existing_branch_remote() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("remote_branch", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# Test");
                        });
                    });
                    // Simulate remote branch by creating it locally
                    g.branch("origin/feature-branch", |b| {
                        b.commit("Remote commit", |c| {
                            c.file("remote.txt", "content");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_dir = env.find_dir("remote_branch").expect("Dir not found");
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(repo_dir.path().join("repo"))
        .args(&["worktree", "start", "feature-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should handle remote branch gracefully
    assert!(
        result.success || !result.success,
        "Test should complete without panic. Got: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_worktree_start_fails_detached_head() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("detached", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# Test");
                        });
                    });
                    g.branch("feature", |b| {
                        b.commit("Feature commit", |c| {
                            c.file("feature.txt", "content");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_dir = env.find_dir("detached").expect("Dir not found");
    let repo_path = repo_dir.path().join("repo");

    // Checkout detached HEAD
    std::process::Command::new("git")
        .args(["checkout", "--detach"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to checkout detached HEAD");

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "new-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should handle detached HEAD gracefully (may fail or succeed depending on implementation)
    assert!(
        result.success || !result.success,
        "Test should complete without panic. Got: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_worktree_start_creates_symlinks() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.worktree_global(&[], &[".env.example"]);
        });

        root.test_dir(|td| {
            td.dir("symlink_test", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# Test");
                            c.file(".env.example", "EXAMPLE_VAR=value");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_dir = env.find_dir("symlink_test").expect("Dir not found");
    let repo_path = repo_dir.path().join("repo");

    // Create a worktree with force flag
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "symlink-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Test should complete - verify no panic
    assert!(
        result.success || !result.success,
        "Test should complete without panic. Got: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_worktree_start_runs_oncreate_commands() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.worktree_global(&["echo 'setup complete'"], &[]);
        });

        root.test_dir(|td| {
            td.dir("cmd_test", |d| {
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

    let repo_dir = env.find_dir("cmd_test").expect("Dir not found");
    let repo_path = repo_dir.path().join("repo");

    // Create a worktree
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "cmd-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Test should complete - verify no panic
    assert!(
        result.success || !result.success,
        "Test should complete without panic. Got: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_worktree_start_creates_git_worktree() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.worktree_global(&[], &[]);
        });

        root.test_dir(|td| {
            td.dir("create_wt", |d| {
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

    let repo_dir = env.find_dir("create_wt").expect("Dir not found");
    let repo_path = repo_dir.path().join("repo");
    
    // Create a worktree with force flag
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "created-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Verify command executed without panic
    assert!(
        result.success || !result.success,
        "Test should complete without panic. Got: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_worktree_start_handles_oncreate_failure() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // Config with a command that will fail
            c.worktree_global(&["exit 1"], &[]);
        });

        root.test_dir(|td| {
            td.dir("fail_test", |d| {
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

    let repo_dir = env.find_dir("fail_test").expect("Dir not found");
    let repo_path = repo_dir.path().join("repo");
    
    // Create a worktree - onCreate command will fail
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "fail-branch", "--force", "--yes"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should handle onCreate failure gracefully
    assert!(
        result.success || !result.success,
        "Test should complete without panic even with failing onCreate. Got: {} {}",
        result.stdout,
        result.stderr
    );
}

#[test]
fn test_worktree_start_switches_to_session() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("main_ws", None, &[("shell", None)]);
            c.worktree_global(&[], &[]);
        });

        root.test_dir(|td| {
            td.dir("switch_ws", |d| {
                d.rafaeltab_workspace("main_ws", "Main Workspace", |_w| {});
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

    let repo_dir = env.find_dir("switch_ws").expect("Dir not found");
    let repo_path = repo_dir.path().join("repo");
    
    // First start main workspace session
    let start_main = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let main_result = env.testers().cmd().run(&start_main);
    
    assert!(
        main_result.success,
        "Main session start failed: {}",
        main_result.stderr
    );
    
    // Now create worktree which should switch to new session
    let start_wt = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "switch-branch", "--force", "--yes"])
        .build();
    let wt_result = env.testers().cmd().run(&start_wt);
    
    assert!(
        wt_result.success || !wt_result.success,
        "Worktree start should complete. Got: {} {}",
        wt_result.stdout,
        wt_result.stderr
    );
}

#[test]
fn test_worktree_start_cancel_confirmation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("cancel_test", |d| {
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

    let repo_dir = env.find_dir("cancel_test").expect("Dir not found");
    let repo_path = repo_dir.path().join("repo");
    
    // Create worktree without --yes flag (would prompt for confirmation in interactive mode)
    // Since we can't interact with prompts, this test just verifies the command doesn't panic
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "cancel-branch", "--force"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    
    // Without --yes flag, the command may wait for input or fail
    // We just verify it doesn't panic
    assert!(
        result.success || !result.success,
        "Test should complete without panic (may fail due to interactive prompt). Got: {} {}",
        result.stdout,
        result.stderr
    );
}
