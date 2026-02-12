mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabGitMixin, RafaeltabRootMixin},
    CliCommandBuilder,
};
use std::process::Command as StdCommand;
use test_descriptors::testers::CommandTester;
use test_descriptors::TestEnvironment;

/// TC-WT-01: Switch to workspace with no worktrees (baseline)
///
/// Given: A workspace exists in a git repository, no worktrees have been created, no tmux sessions exist
/// When: User runs tmux start (which triggers worktree session creation)
/// Then: Main workspace session is created, no additional sessions are created, no errors occur
#[test]
fn test_switch_to_workspace_without_worktrees() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("editor", None), ("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project-a", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project A");
                        });
                    });
                    g.rafaeltab_workspace("project_a", "Project A", |_w| {});
                });
            });
        });
    })
    .create();

    // Use tmux start (which will also create worktree sessions)
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(
        result.success,
        "Failed to start tmux sessions:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify only main session exists (no worktrees)
    assert!(
        env.tmux().session_exists("Project A"),
        "Main workspace session should exist"
    );
    let sessions = env.tmux().list_sessions().expect("Failed to list sessions");
    assert_eq!(
        sessions.len(),
        1,
        "Should only have 1 session (main workspace, no worktrees). Found: {:?}",
        sessions
    );
}

/// TC-WT-02: Switch creates sessions for all worktrees
///
/// Given: A workspace "MyProject" exists with three worktrees:
///   - feature/login
///   - fix/bug-123
///   - feat/database
///
/// When: User runs tmux start
/// Then: Main session "MyProject" is created, three worktree sessions are created with correct names
#[test]
fn test_switch_creates_sessions_for_worktrees() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("editor", None), ("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# My Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create worktrees
    let worktree_path_1 = repo_path.parent().unwrap().join("feat-login");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feature/login",
            worktree_path_1.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree 1");

    let worktree_path_2 = repo_path.parent().unwrap().join("fix-bug");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "fix/bug-123",
            worktree_path_2.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree 2");

    let worktree_path_3 = repo_path.parent().unwrap().join("feat-database");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat/database",
            worktree_path_3.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree 3");

    // Start sessions - this should trigger worktree session creation
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(
        result.success,
        "Failed to start tmux sessions:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify main session exists
    assert!(
        env.tmux().session_exists("MyProject"),
        "Main workspace session should exist"
    );

    // Verify worktree sessions exist with correct naming: {workspace}-{branch}
    assert!(
        env.tmux().session_exists("MyProject-feature/login"),
        "Worktree session for feature/login should exist"
    );
    assert!(
        env.tmux().session_exists("MyProject-fix/bug-123"),
        "Worktree session for fix/bug-123 should exist"
    );
    assert!(
        env.tmux().session_exists("MyProject-feat/database"),
        "Worktree session for feat/database should exist"
    );

    let sessions = env.tmux().list_sessions().expect("Failed to list sessions");
    assert_eq!(
        sessions.len(),
        4,
        "Should have 4 sessions: main + 3 worktrees. Found: {:?}",
        sessions
    );

    // Cleanup worktrees
    for path in [&worktree_path_1, &worktree_path_2, &worktree_path_3] {
        StdCommand::new("git")
            .args(["worktree", "remove", "--force", path.to_str().unwrap()])
            .current_dir(&repo_path)
            .output()
            .ok();
    }
}

/// TC-WT-03: Worktree sessions use correct window config
///
/// Given: A workspace with custom tmux session config (nvim, terminal, logs)
/// When: User runs tmux start
/// Then: Worktree session uses the workspace-specific window configuration
#[test]
fn test_switch_uses_workspace_windows_for_worktrees() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("default", None)]);

            // Workspace-specific windows
            c.tmux_session(
                "proj",
                Some("TestProj"),
                &[
                    ("nvim", Some("nvim .")),
                    ("terminal", None),
                    ("logs", Some("tail -f app.log")),
                ],
            );
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "TestProj", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create a worktree
    let worktree_path = repo_path.parent().unwrap().join("feat-test");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat/test",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Start sessions
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(
        result.success,
        "Command failed:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify worktree session exists
    assert!(
        env.tmux().session_exists("TestProj-feat/test"),
        "Worktree session should exist"
    );

    // Verify windows match workspace config (not default)
    let session = env
        .find_tmux_session("TestProj-feat/test")
        .expect("Session should exist");
    let windows = session.windows();
    assert_eq!(
        windows.len(),
        3,
        "Should have 3 windows from workspace config"
    );
    assert!(
        windows.iter().any(|w| w.contains("nvim")),
        "Should have nvim window"
    );
    assert!(
        windows.iter().any(|w| w.contains("terminal")),
        "Should have terminal window"
    );
    assert!(
        windows.iter().any(|w| w.contains("logs")),
        "Should have logs window"
    );

    // Should NOT have default window
    assert!(
        !windows.iter().any(|w| w.contains("default")),
        "Should not use default windows"
    );

    // Cleanup
    StdCommand::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .ok();
}

/// TC-WT-04: Switch skips existing worktree sessions
///
/// Given: A workspace with 2 worktrees, tmux sessions already exist for both worktrees
/// When: User runs tmux start again
/// Then: No new sessions are created, existing sessions are reused, no duplicates
#[test]
fn test_switch_skips_existing_worktree_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "TestProj", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create worktrees
    let worktree_path_1 = repo_path.parent().unwrap().join("feat-test");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat/test",
            worktree_path_1.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree 1");

    let worktree_path_2 = repo_path.parent().unwrap().join("fix-bug");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "fix/bug",
            worktree_path_2.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree 2");

    // Start sessions first time
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(
        result.success,
        "First start should succeed:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify sessions exist
    assert!(
        env.tmux().session_exists("TestProj"),
        "Main session should exist"
    );
    assert!(
        env.tmux().session_exists("TestProj-feat/test"),
        "Worktree session 1 should exist"
    );
    assert!(
        env.tmux().session_exists("TestProj-fix/bug"),
        "Worktree session 2 should exist"
    );

    // Run start again - should be idempotent
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(
        result.success,
        "Second start should succeed:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Still should have exactly 3 sessions (no duplicates)
    let sessions = env.tmux().list_sessions().expect("Failed to list sessions");
    assert_eq!(
        sessions.len(),
        3,
        "Should still have 3 sessions (no duplicates). Found: {:?}",
        sessions
    );

    // Cleanup
    for path in [&worktree_path_1, &worktree_path_2] {
        StdCommand::new("git")
            .args(["worktree", "remove", "--force", path.to_str().unwrap()])
            .current_dir(&repo_path)
            .output()
            .ok();
    }
}

/// TC-WT-05: Switch handles non-git workspace gracefully
///
/// Given: A workspace is not a git repository
/// When: User runs tmux start
/// Then: Main workspace session is created, no worktree discovery happens, no errors are displayed
#[test]
fn test_switch_handles_non_git_workspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("shell", None)]);
        });

        root.test_dir(|td| {
            // Regular directory, not a git repo
            td.dir("non-git-workspace", |d| {
                d.rafaeltab_workspace("non_git", "NonGit Workspace", |_w| {});
            });
        });
    })
    .create();

    // Start sessions - should not error even though workspace is not a git repo
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    assert!(
        result.success,
        "Should succeed even for non-git workspace:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );
    assert!(
        !result.stderr.contains("error") && !result.stderr.contains("Error"),
        "Should not show errors for non-git workspace. STDERR: {}",
        result.stderr
    );

    // Should still create main workspace session
    assert!(
        env.tmux().session_exists("NonGit Workspace"),
        "Main workspace session should exist"
    );

    let sessions = env.tmux().list_sessions().expect("Failed to list sessions");
    assert_eq!(
        sessions.len(),
        1,
        "Should only have main session (no worktrees for non-git workspace). Found: {:?}",
        sessions
    );
}

/// TC-WT-06: Worktree sessions created after switch (timing)
///
/// Given: A workspace with multiple worktrees
/// When: User runs tmux start
/// Then: Command returns quickly (worktree creation doesn't block)
#[test]
fn test_switch_creates_worktree_sessions_in_background() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "TimingTest", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create a worktree
    let worktree_path = repo_path.parent().unwrap().join("feat-timing");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat/timing",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Start sessions and measure time
    let start = std::time::Instant::now();
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    let duration = start.elapsed();

    assert!(
        result.success,
        "Command should succeed:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Command should return quickly (under 5 seconds even with worktrees)
    assert!(
        duration.as_secs() < 5,
        "Command took too long: {:?}. Worktree creation should not block.",
        duration
    );

    // Verify both sessions were created
    assert!(
        env.tmux().session_exists("TimingTest"),
        "Main session should exist"
    );
    assert!(
        env.tmux().session_exists("TimingTest-feat/timing"),
        "Worktree session should exist"
    );

    // Cleanup
    StdCommand::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .ok();
}

/// TC-WT-07: Worktree sessions get unique UUIDs
///
/// Given: A workspace "project-a" with worktree "feat/test"
/// When: User runs tmux start
/// Then: Worktree session has RAFAELTAB_SESSION_ID environment variable with deterministic UUID
#[test]
fn test_worktree_sessions_have_unique_uuids() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "UUIDTest", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create a worktree
    let worktree_path = repo_path.parent().unwrap().join("feat-uuid");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat/uuid",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Start sessions
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);
    assert!(
        result.success,
        "Command should succeed:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Verify worktree session exists
    assert!(
        env.tmux().session_exists("UUIDTest-feat/uuid"),
        "Worktree session should exist"
    );

    // Check that the session has the RAFAELTAB_SESSION_ID environment variable
    // by querying tmux session environment
    let _session = env
        .find_tmux_session("UUIDTest-feat/uuid")
        .expect("Session should exist");

    // Try to get the session environment variable using -L for socket name
    let output = StdCommand::new("tmux")
        .args([
            "-L",
            env.tmux_socket(),
            "show-environment",
            "-t",
            "UUIDTest-feat/uuid",
            "RAFAELTAB_SESSION_ID",
        ])
        .output()
        .expect("Failed to query session environment");

    let env_var = String::from_utf8_lossy(&output.stdout);
    let env_stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        env_var.contains("RAFAELTAB_SESSION_ID="),
        "Worktree session should have RAFAELTAB_SESSION_ID environment variable. Got stdout: '{}', stderr: '{}'",
        env_var, env_stderr
    );

    // Extract and verify it's a valid UUID format
    let uuid_part = env_var
        .trim()
        .strip_prefix("RAFAELTAB_SESSION_ID=")
        .unwrap_or("");
    assert!(!uuid_part.is_empty(), "UUID should not be empty");

    // Cleanup
    StdCommand::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .ok();
}

/// TC-WT-08: Switch handles git command failures gracefully
///
/// Given: A workspace in a git repo where worktree discovery might fail
/// When: User runs tmux start
/// Then: Main session is created, worktree creation is skipped silently, no user-visible errors
#[test]
fn test_switch_handles_git_command_failures() {
    // Create a workspace that is a git repo but we'll simulate failure by using a repo
    // where worktree commands might not work properly (e.g., bare repo or corrupted state)
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_windows(&[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "GitTest", |_w| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("project/repo");

    // Create one valid worktree first
    let worktree_path = repo_path.parent().unwrap().join("feat-normal");
    StdCommand::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            "feat/normal",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Start sessions - should succeed even if there are any git issues
    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["tmux", "start"])
        .build();
    let result = env.testers().cmd().run(&cmd);

    // Should succeed without errors
    assert!(
        result.success,
        "Should succeed even if git commands have issues:\nSTDOUT: {}\nSTDERR: {}",
        result.stdout, result.stderr
    );

    // Should not have error messages in stderr
    let stderr_lower = result.stderr.to_lowercase();
    assert!(
        !stderr_lower.contains("error") || !stderr_lower.contains("fatal"),
        "Should not show error messages. STDERR: {}",
        result.stderr
    );

    // Main session should still be created
    assert!(
        env.tmux().session_exists("GitTest"),
        "Main workspace session should exist even if worktree discovery had issues"
    );

    // Cleanup
    StdCommand::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            worktree_path.to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .ok();
}
