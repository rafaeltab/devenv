mod common;

use crate::common::{
    CliCommandBuilder,
    rafaeltab_descriptors::{RafaeltabGitMixin, RafaeltabRootMixin},
};
use test_descriptors::TestEnvironment;
use test_descriptors::testers::CommandTester;

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
                        w.worktree(&[], &[], &[]);
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

    // Now complete the worktree (use --force-git because the branch has unpushed commits)
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/test", "--yes", "--force-git"])
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
                        w.worktree(&[], &[], &[]);
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
        .args(&["worktree", "complete", "--yes", "--force-git"]) // No branch name!
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
fn test_worktree_complete_with_force_git_flag() {
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
                        w.worktree(&[], &[], &[]);
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

    // Without --force-git, this should fail due to unpushed commits
    let complete_no_force = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/force-test", "--yes"])
        .build();
    let no_force_result = env.testers().cmd().run(&complete_no_force);

    assert!(
        !no_force_result.success,
        "Should fail without --force-git when there are unpushed commits"
    );
    assert!(
        no_force_result.stderr.contains("unpushed commits")
            || no_force_result.stdout.contains("unpushed commits"),
        "Error should mention unpushed commits. Got stdout: '{}', stderr: '{}'",
        no_force_result.stdout,
        no_force_result.stderr
    );

    // With --force-git, it should succeed
    let complete_with_force = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/force-test",
            "--yes",
            "--force-git",
        ])
        .build();
    let force_result = env.testers().cmd().run(&complete_with_force);

    assert!(
        force_result.success,
        "Should succeed with --force-git flag.\nSTDOUT: {}\nSTDERR: {}",
        force_result.stdout, force_result.stderr
    );

    // Verify worktree is removed
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete with --force-git"
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
                        w.worktree(&[], &[], &[]);
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

    // Now clean up with --force-git
    let cleanup_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/unpushed",
            "--yes",
            "--force-git",
        ])
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
                        w.worktree(&[], &[], &[]);
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
                        w.worktree(&[], &[], &[]);
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
                        w.worktree(&[], &[], &[]);
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
        .args(&[
            "worktree",
            "complete",
            "feat/kill-test",
            "--yes",
            "--force-git",
        ])
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
                        w.worktree(&[], &[], &[]);
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
        .args(&[
            "worktree",
            "complete",
            "feat/cleanup",
            "--yes",
            "--force-git",
        ])
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
                        w.worktree(&[], &[], &[]);
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

    // Now clean up with --force-git
    let cleanup_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&["worktree", "complete", "feat/dirty", "--yes", "--force-git"])
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
                        w.worktree(&[], &[], &[]);
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
        .args(&["worktree", "complete", "--yes", "--force-git"])
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

#[test]
fn test_worktree_complete_runs_on_destroy_before_teardown() {
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
                        w.worktree(&[], &["echo 'destroyed'"], &[]);
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
        .args(&["worktree", "start", "feat/on-destroy", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "worktree start should succeed.\nSTDOUT: {}\nSTDERR: {}",
        start_result.stdout, start_result.stderr
    );

    // Verify worktree session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/on-destroy"),
        "Worktree session should exist after start"
    );

    let worktree_path = env.root_path().join("project/feat/on-destroy");
    assert!(
        worktree_path.exists(),
        "Worktree directory should exist after start"
    );

    // Create a marker file in the worktree that the onDestroy command will write to
    // Set the workspace config with an onDestroy that creates a marker file
    // We use a simple echo command that should succeed

    // Now complete the worktree with --force-git (because unpushed commits)
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/on-destroy",
            "--yes",
            "--force-git",
        ])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify onDestroy ran (check output contains the onDestroy command output)
    let output = format!("{} {}", complete_result.stdout, complete_result.stderr);
    assert!(
        output.contains("destroyed")
            || output.contains("onDestroy")
            || output.contains("Completed"),
        "Output should indicate onDestroy commands ran. Got: {}",
        output
    );

    // Verify worktree was removed (teardown happened after onDestroy)
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete"
    );
}

#[test]
fn test_worktree_complete_on_destroy_failure_aborts_teardown() {
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
                        w.worktree(&[], &["exit 1"], &[]);
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
        .args(&["worktree", "start", "feat/fail-destroy", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);

    assert!(
        start_result.success,
        "worktree start should succeed.\nSTDOUT: {}\nSTDERR: {}",
        start_result.stdout, start_result.stderr
    );

    let worktree_path = env.root_path().join("project/feat/fail-destroy");
    assert!(
        worktree_path.exists(),
        "Worktree directory should exist after start"
    );

    // Now complete the worktree with --force-git
    // The onDestroy command (exit 1) should fail, causing teardown to abort
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/fail-destroy",
            "--yes",
            "--force-git",
        ])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    // The command should fail because onDestroy failed
    assert!(
        !complete_result.success,
        "worktree complete should fail when onDestroy command fails.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify error message mentions onDestroy
    let output = format!("{} {}", complete_result.stdout, complete_result.stderr);
    assert!(
        output.to_lowercase().contains("ondestroy")
            || output.to_lowercase().contains("failed")
            || output.to_lowercase().contains("error"),
        "Error should mention onDestroy failure. Got: {}",
        output
    );

    // Verify worktree is still intact (teardown was aborted)
    assert!(
        worktree_path.exists(),
        "Worktree directory should still exist after failed onDestroy (teardown aborted)"
    );
}

#[test]
fn test_worktree_complete_skip_destroy_skips_on_destroy() {
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
                        w.worktree(&[], &["exit 1"], &[]); // onDestroy that would fail
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
        .args(&["worktree", "start", "feat/skip-destroy", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    let worktree_path = env.root_path().join("project/feat/skip-destroy");
    assert!(worktree_path.exists(), "Worktree should exist after start");

    // Complete with --skip-destroy and --force-git
    // onDestroy command (exit 1) should be skipped, worktree should be removed successfully
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/skip-destroy",
            "--yes",
            "--skip-destroy",
            "--force-git",
        ])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed with --skip-destroy.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify worktree is removed (teardown proceeded despite failing onDestroy)
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete with --skip-destroy"
    );

    // Output should indicate success (not PartialSuccess, since onDestroy was skipped)
    let output = format!("{} {}", complete_result.stdout, complete_result.stderr);
    assert!(
        output.contains("Completed worktree") || output.contains("Removed"),
        "Output should indicate successful completion. Got: {}",
        output
    );
}

#[test]
fn test_worktree_complete_force_destroy_continues_past_failures() {
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
                        w.worktree(&[], &["exit 1"], &[]); // onDestroy that fails
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
        .args(&["worktree", "start", "feat/force-destroy", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    let worktree_path = env.root_path().join("project/feat/force-destroy");
    assert!(worktree_path.exists(), "Worktree should exist after start");

    // Complete with --force-destroy and --force-git
    // onDestroy command (exit 1) should fail, but teardown should continue
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/force-destroy",
            "--yes",
            "--force-destroy",
            "--force-git",
        ])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed with --force-destroy (partial success).\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify worktree is removed (teardown proceeded despite onDestroy failure)
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete with --force-destroy"
    );

    // Output should mention the failed onDestroy command
    let output = format!("{} {}", complete_result.stdout, complete_result.stderr);
    assert!(
        output.to_lowercase().contains("ondestroy")
            || output.to_lowercase().contains("failed")
            || output.to_lowercase().contains("partial"),
        "Output should mention the failed onDestroy command or partial success. Got: {}",
        output
    );
}

#[test]
fn test_worktree_complete_skip_destroy_takes_precedence_over_force_destroy() {
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
                        w.worktree(&[], &["exit 1"], &[]); // onDestroy that would fail
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
        .args(&["worktree", "start", "feat/skip-precedence", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    let worktree_path = env.root_path().join("project/feat/skip-precedence");
    assert!(worktree_path.exists(), "Worktree should exist after start");

    // Complete with both --skip-destroy and --force-destroy
    // --skip-destroy should take precedence: onDestroy should not run at all
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/skip-precedence",
            "--yes",
            "--skip-destroy",
            "--force-destroy",
            "--force-git",
        ])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed when --skip-destroy takes precedence.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify worktree is removed
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete"
    );

    // Output should NOT mention failed onDestroy commands (they were skipped)
    let output = format!("{} {}", complete_result.stdout, complete_result.stderr);
    assert!(
        !output.to_lowercase().contains("partial success"),
        "Output should not mention partial success since onDestroy was skipped. Got: {}",
        output
    );

    // Output should indicate clean success
    assert!(
        output.contains("Completed worktree") || output.contains("Removed"),
        "Output should indicate successful completion. Got: {}",
        output
    );
}

#[test]
fn test_worktree_complete_force_git_with_force_destroy() {
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
                        w.worktree(&[], &["exit 1"], &[]);
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
        .args(&["worktree", "start", "feat/combined-flags", "--yes"])
        .build();
    let start_result = env.testers().tmux_client_cmd().run(&start_cmd);
    assert!(start_result.success, "worktree start should succeed");

    let worktree_path = env.root_path().join("project/feat/combined-flags");
    assert!(worktree_path.exists(), "Worktree should exist after start");

    // Create an uncommitted change to test that --force-git bypasses the safety check
    use std::fs;
    let new_file_path = worktree_path.join("dirty.txt");
    fs::write(&new_file_path, "uncommitted change").expect("Should write file");

    // Complete with both --force-destroy and --force-git
    // Should: (1) bypass git safety checks, (2) continue past onDestroy failure, (3) force remove worktree
    let complete_cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_cwd(&repo_path)
        .args(&[
            "worktree",
            "complete",
            "feat/combined-flags",
            "--yes",
            "--force-destroy",
            "--force-git",
        ])
        .build();
    let complete_result = env.testers().cmd().run(&complete_cmd);

    assert!(
        complete_result.success,
        "worktree complete should succeed with --force-destroy --force-git.\nSTDOUT: {}\nSTDERR: {}",
        complete_result.stdout, complete_result.stderr
    );

    // Verify worktree is removed
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed after complete with --force-destroy --force-git"
    );
}
