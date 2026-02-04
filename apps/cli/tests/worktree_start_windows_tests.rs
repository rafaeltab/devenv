mod common;

use crate::common::{
    rafaeltab_descriptors::{RafaeltabDirMixin, RafaeltabGitMixin, RafaeltabRootMixin},
    run_cli_with_tmux,
};
use std::process::Command;
use test_descriptors::TestEnvironment;

#[test]
fn test_worktree_start_uses_default_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // Set custom default windows
            c.default_windows(&[
                ("editor", Some("vim")),
                ("shell", None),
                ("build", Some("npm run dev")),
            ]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |_w| {});
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let repo_path = env.root_path().join("project/repo");

    // Run worktree start
    let (stdout, stderr, success) = run_cli_with_tmux(
        &["worktree", "start", "feat/test", "--yes"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(
        success,
        "Command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        stdout, stderr
    );

    // Verify session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/test"),
        "Worktree session should exist"
    );

    // Verify windows using the session API
    let session = env
        .find_tmux_session("MyProject-feat/test")
        .expect("Session should exist");
    let windows = session.windows();
    assert_eq!(
        windows.len(),
        3,
        "Should have 3 windows from default config"
    );
    assert!(
        windows.iter().any(|w| w.contains("editor")),
        "Should have editor window"
    );
    assert!(
        windows.iter().any(|w| w.contains("shell")),
        "Should have shell window"
    );
    assert!(
        windows.iter().any(|w| w.contains("build")),
        "Should have build window"
    );

    // Cleanup
    Command::new("git")
        .args(["worktree", "remove", "--force", "../feat-test"])
        .current_dir(&repo_path)
        .output()
        .ok();
}

#[test]
fn test_worktree_start_uses_workspace_specific_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // Set default windows
            c.default_windows(&[("default", None)]);

            // Set workspace-specific session config
            c.tmux_session(
                "proj",
                Some("MyProject"),
                &[
                    ("nvim", Some("nvim .")),
                    ("terminal", None),
                    ("server", Some("npm start")),
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
                    g.rafaeltab_workspace("proj", "MyProject", |_w| {});
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let repo_path = env.root_path().join("project/repo");

    // Run worktree start
    let (stdout, stderr, success) = run_cli_with_tmux(
        &["worktree", "start", "feat/api", "--yes"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(
        success,
        "Command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        stdout, stderr
    );

    // Verify session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/api"),
        "Worktree session should exist"
    );

    // Verify windows match workspace config (not default)
    let session = env
        .find_tmux_session("MyProject-feat/api")
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
        windows.iter().any(|w| w.contains("server")),
        "Should have server window"
    );

    // Should NOT have default window
    assert!(
        !windows.iter().any(|w| w.contains("default")),
        "Should not have default window"
    );

    // Cleanup
    Command::new("git")
        .args(["worktree", "remove", "--force", "../feat-api"])
        .current_dir(&repo_path)
        .output()
        .ok();
}

#[test]
fn test_worktree_start_handles_empty_default_windows() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            // No default windows configured
            c.default_windows(&[]);
        });

        root.test_dir(|td| {
            td.dir("project", |d| {
                d.git("repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Project");
                        });
                    });
                    g.rafaeltab_workspace("proj", "MyProject", |_w| {});
                });
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();
    let repo_path = env.root_path().join("project/repo");

    // Run worktree start
    let (stdout, stderr, success) = run_cli_with_tmux(
        &["worktree", "start", "feat/empty", "--yes"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(
        success,
        "Command should succeed even with empty windows.\nSTDOUT: {}\nSTDERR: {}",
        stdout, stderr
    );

    // Verify session exists
    assert!(
        env.tmux().session_exists("MyProject-feat/empty"),
        "Worktree session should exist"
    );

    // Verify no windows or one default window (implementation dependent)
    let session = env
        .find_tmux_session("MyProject-feat/empty")
        .expect("Session should exist");
    let windows = session.windows();
    // Accept either 0 or 1 window depending on tmux behavior
    assert!(windows.len() <= 1, "Should have at most 1 window");

    // Cleanup
    Command::new("git")
        .args(["worktree", "remove", "--force", "../feat-empty"])
        .current_dir(&repo_path)
        .output()
        .ok();
}
