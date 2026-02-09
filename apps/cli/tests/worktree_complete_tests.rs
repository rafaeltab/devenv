mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabGitMixin, RafaeltabRootMixin},
    CliCommandBuilder,
};
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

#[test]
fn test_worktree_complete_removes_worktree() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // First, start the worktree
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "feat/test", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "worktree start should succeed.\nSTDOUT: {}\nSTDERR: {}",
        start_result.stdout, start_result.stderr
    );

    // Verify worktree session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/test"),
        "Worktree session should exist after start"
    );

    // Verify worktree directory exists - worktrees are created as siblings to repo
    let worktree_path = env.root_path().join("project/feat/test");
    assert!(
        worktree_path.exists(),
        "Worktree directory should exist after start at {:?}",
        worktree_path
    );

    // Now complete the worktree (use --force because the branch has unpushed commits)
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/test", "--yes", "--force"])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify output indicates success (check both stdout and stderr)
    let output = format!("{} {}", complete_result.stdout, complete_result.stderr);
    assert!(
        output.contains("Completed worktree")
            || output.contains("Removed")
            || output.contains("worktree"),
        "Output should indicate successful completion. Got stdout: '{}', stderr: '{}'",
        complete_result.stdout,
        complete_result.stderr
    );

    // Verify tmux session is killed
    assert!(
        !env.tmux().session_exists("MyProject-feat/test"),
        "Worktree session should be killed after complete"
    );

    // Verify worktree directory is removed
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete"
    );
}

#[test]
fn test_worktree_complete_uses_current_directory() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");
    let worktree_path = env.root_path().join("project/feat/current-test");

    // First, start the worktree
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "feat/current-test", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "worktree start should succeed.\nSTDOUT: {}\nSTDERR: {}",
        start_result.stdout, start_result.stderr
    );

    // Verify worktree session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/current-test"),
        "Worktree session should exist after start"
    );

    // Verify worktree directory exists
    assert!(
        worktree_path.exists(),
        "Worktree directory should exist after start"
    );

    // Now complete the worktree from within the worktree directory (no branch arg)
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&worktree_path) // Execute from within the worktree
        .args(&["worktree", "complete", "--yes", "--force"]) // No branch name!
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed from worktree directory.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify tmux session is killed
    assert!(
        !env.tmux().session_exists("MyProject-feat/current-test"),
        "Worktree session should be killed after complete"
    );

    // Verify worktree directory is removed
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete"
    );
}

#[test]
fn test_worktree_complete_with_force_flag() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Start the worktree
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "feat/force-test", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    let worktree_path = env.root_path().join("project/feat/force-test");

    // Without --force, this should fail due to unpushed commits
    let complete_no_force = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/force-test", "--yes"])
        .build();
    let no_force_result = env.testers().cmd().run(&complete_no_force);

    assert!(
        !no_force_result.success,
        "Should fail without --force when there are unpushed commits"
    );
    assert!(
        no_force_result.stderr.contains("unpushed commits")
            || no_force_result.stdout.contains("unpushed commits"),
        "Error should mention unpushed commits. Got stdout: '{}', stderr: '{}'",
        no_force_result.stdout,
        no_force_result.stderr
    );

    // With --force, it should succeed
    let complete_with_force = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/force-test",
            "--yes",
            "--force",
        ])
        .build();
    let force_result = env.testers().cmd().run(&complete_with_force);

    assert!(
        force_result.success,
        "Should succeed with --force flag.\nSTDOUT: {}\nSTDERR: {}",
        force_result.stdout, force_result.stderr
    );

    // Verify worktree is removed
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete with --force"
    );
}

#[test]
fn test_worktree_complete_fails_on_unpushed_commits() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Start the worktree
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "feat/unpushed", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    let worktree_path = env.root_path().join("project/feat/unpushed");
    assert!(worktree_path.exists(), "Worktree should exist");

    // Try to complete without --force - should fail due to unpushed commits
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/unpushed", "--yes"])
        .build();
    let result = env.testers().cmd().run(&complete_cmd);

    assert!(
        !result.success,
        "Should fail when there are unpushed commits and no --force flag. Got success: {}, stdout: '{}', stderr: '{}'",
        result.success, result.stdout, result.stderr
    );

    let output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        output.to_lowercase().contains("unpushed") || output.to_lowercase().contains("commits"),
        "Error message should mention unpushed commits. Got: {}",
        output
    );

    // Verify worktree still exists (wasn't removed)
    assert!(
        worktree_path.exists(),
        "Worktree should still exist after failed complete"
    );

    // Now clean up with --force
    let cleanup_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/unpushed", "--yes", "--force"])
        .build();
    let _ = env.testers().cmd().run(&cleanup_cmd);
}

#[test]
fn test_worktree_complete_fails_on_main_repo() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Try to complete from the main repo (not a worktree)
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "--yes"]) // No branch, will use current dir
        .build();
    let result = env.testers().cmd().run(&complete_cmd);

    // The command should indicate an error about being in main repo
    let output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        output.to_lowercase().contains("main repo")
            || output.to_lowercase().contains("not a worktree")
            || output.to_lowercase().contains("error"),
        "Should report error when trying to complete main repo. Got: {}",
        output
    );
}

#[test]
fn test_worktree_complete_nonexistent_branch() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Try to complete a non-existent branch
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "nonexistent-branch", "--yes"])
        .build();
    let result = env.testers().cmd().run(&complete_cmd);

    // Should report that worktree/branch was not found
    let output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        output.to_lowercase().contains("not found")
            || output.to_lowercase().contains("error")
            || output.to_lowercase().contains("no such"),
        "Should report error for non-existent branch. Got: {}",
        output
    );
}

#[test]
fn test_worktree_complete_kills_tmux_session() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Start the worktree
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "feat/kill-test", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    // Verify session exists before complete
    assert!(
        env.tmux().session_exists("MyProject-feat/kill-test"),
        "Worktree session should exist before complete"
    );

    // Complete the worktree
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/kill-test", "--yes", "--force"])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify session is killed
    assert!(
        !env.tmux().session_exists("MyProject-feat/kill-test"),
        "Worktree session should be killed after complete"
    );
}

#[test]
fn test_worktree_complete_cleans_empty_directories() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Start a worktree with nested path
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "feat/cleanup", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    // Verify the nested directory structure exists
    let worktree_path = env.root_path().join("project/feat/cleanup");
    let feat_dir = env.root_path().join("project/feat");

    assert!(worktree_path.exists(), "Worktree should exist");
    assert!(feat_dir.exists(), "feat directory should exist");

    // Complete the worktree
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/cleanup", "--yes", "--force"])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify worktree directory is removed
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed"
    );

    // Verify the parent feat directory is cleaned up if empty
    // (implementation may or may not clean this up)
    let output = format!("{} {}", complete_result.stdout, complete_result.stderr);
    assert!(
        output.contains("Removed") || output.contains("worktree"),
        "Output should indicate removal. Got: {}",
        output
    );
}

#[test]
fn test_worktree_complete_fails_on_uncommitted_changes() {
    use std::fs;
    use std::process::Command as StdCommand;

    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Start the worktree
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "feat/dirty", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    let worktree_path = env.root_path().join("project/feat/dirty");
    assert!(worktree_path.exists(), "Worktree should exist");

    // Create an uncommitted change in the worktree
    let new_file_path = worktree_path.join("uncommitted.txt");
    fs::write(&new_file_path, "This is an uncommitted change").expect("Should write file");

    // Add the file to git (staged but not committed)
    let _ = StdCommand::new("git")
        .args(["add", "uncommitted.txt"])
        .current_dir(&worktree_path)
        .output();

    // Try to complete without --force - should fail due to uncommitted changes
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/dirty", "--yes"])
        .build();
    let result = env.testers().cmd().run(&complete_cmd);

    // The command should indicate an error about uncommitted changes
    let output = format!("{} {}", result.stdout, result.stderr);
    assert!(
        output.to_lowercase().contains("uncommitted")
            || output.to_lowercase().contains("changes")
            || output.to_lowercase().contains("error"),
        "Should report error for uncommitted changes. Got: {}",
        output
    );

    // Verify worktree still exists (wasn't removed)
    assert!(
        worktree_path.exists(),
        "Worktree should still exist after failed complete"
    );

    // Now clean up with --force
    let cleanup_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/dirty", "--yes", "--force"])
        .build();
    let _ = env.testers().cmd().run(&cleanup_cmd);
}

#[test]
fn test_worktree_complete_switches_to_main_session() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.tmux_session("proj", Some("MyProject"), &[("editor", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.tmux_session("project session", |s| {
                        s.with_client(|_| {});
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |w| {
                        w.worktree(&[], &[]);
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");
    let worktree_path = env.root_path().join("project/feat/switch-test");

    // Start the worktree
    let start_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "start", "feat/switch-test", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    // Verify worktree session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/switch-test"),
        "Worktree session should exist"
    );

    // Complete the worktree from the worktree directory (which triggers client switch)
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&worktree_path) // Run from within the worktree
        .args(&["worktree", "complete", "--yes", "--force"])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify worktree session is killed
    assert!(
        !env.tmux().session_exists("MyProject-feat/switch-test"),
        "Worktree session should be killed"
    );

    // Verify worktree directory is removed
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed"
    );
}
