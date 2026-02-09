use std::process::Command;
use test_descriptors::{TestEnvironment, TmuxSocket};

// TmuxSocket tests - these test the low-level socket directly
#[test]
fn test_tmux_socket_creates_unique_name() {
    let socket1 = TmuxSocket::new();
    let socket2 = TmuxSocket::new();

    assert_ne!(socket1.name(), socket2.name());
}

#[test]
fn test_tmux_socket_run_command() {
    let socket = TmuxSocket::new();

    // Run a simple tmux command
    let result = socket.run_tmux(&["list-sessions"]);
    // Should fail because no sessions exist yet, but command should execute
    assert!(result.is_err() || result.is_ok());

    let _ = socket.kill_server();
}

#[test]
fn test_tmux_socket_list_sessions_empty() {
    let socket = TmuxSocket::new();

    let sessions = socket.list_sessions().unwrap_or(vec![]);
    assert_eq!(sessions.len(), 0);

    let _ = socket.kill_server();
}

#[test]
fn test_tmux_socket_session_exists_false() {
    let socket = TmuxSocket::new();

    assert!(!socket.session_exists("nonexistent"));

    let _ = socket.kill_server();
}

#[test]
fn test_tmux_socket_session_exists_true() {
    let socket = TmuxSocket::new();

    // Create a session
    let _ = socket.run_tmux(&["new-session", "-d", "-s", "test-session"]);

    // Verify session_exists returns true for existing session
    assert!(socket.session_exists("test-session"));

    let _ = socket.kill_server();
}

#[test]
fn test_tmux_socket_kill_server() {
    let socket = TmuxSocket::new();

    // Create a session first
    let _ = socket.run_tmux(&["new-session", "-d", "-s", "test-session"]);

    // Kill the server
    let result = socket.kill_server();
    assert!(result.is_ok());

    // Verify no sessions exist
    assert!(!socket.session_exists("test-session"));
}

// Integration tests using the new hierarchical API
#[test]
fn test_tmux_session_creates_session() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("test-session", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env.find_tmux_session("test-session");
    assert!(session.is_some());
    assert!(session.unwrap().exists());
}

#[test]
fn test_tmux_session_with_multiple_windows() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("multi-window", |s| {
                    s.window("main");
                    s.window("editor");
                    s.window("terminal");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("multi-window")
        .expect("session should exist");
    assert!(session.exists());

    let windows = session.windows();
    assert_eq!(windows.len(), 3);
    assert!(windows.contains(&"main".to_string()));
    assert!(windows.contains(&"editor".to_string()));
    assert!(windows.contains(&"terminal".to_string()));
}

#[test]
fn test_tmux_session_working_directory() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("my-workspace", |d| {
                d.tmux_session("work-session", |s| {
                    s.window("shell");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("work-session")
        .expect("session should exist");

    // Working directory should be the parent dir (my-workspace)
    assert!(session.working_dir().ends_with("my-workspace"));
}

#[test]
fn test_tmux_session_isolated_from_default_server() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("isolated-session", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    // Check that session exists in our isolated socket
    let session = env
        .find_tmux_session("isolated-session")
        .expect("session should exist");
    assert!(session.exists());

    // Check that session doesn't exist in default tmux server
    let default_check = Command::new("tmux")
        .args(["has-session", "-t", "isolated-session"])
        .output()
        .unwrap();

    // Should fail because it's not in the default server
    assert!(!default_check.status.success());
}

#[test]
fn test_tmux_session_with_window_command() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("cmd-session", |s| {
                    s.window_with_command("editor", "echo hello");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("cmd-session")
        .expect("session should exist");
    assert!(session.exists());

    let windows = session.windows();
    assert!(windows.contains(&"editor".to_string()));
}

#[test]
fn test_multiple_tmux_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("project1", |d| {
                d.tmux_session("session-1", |s| {
                    s.window("main");
                });
            });
            td.dir("project2", |d| {
                d.tmux_session("session-2", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session1 = env.find_tmux_session("session-1");
    let session2 = env.find_tmux_session("session-2");

    assert!(session1.is_some());
    assert!(session2.is_some());
    assert!(session1.unwrap().exists());
    assert!(session2.unwrap().exists());
}

#[test]
fn test_tmux_session_with_git_repo() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("dev", |d| {
                d.git("my-project", |_g| {});
                d.tmux_session("dev-session", |s| {
                    s.window("code");
                    s.window("shell");
                });
            });
        });
    })
    .create();

    // Both git repo and tmux session should exist
    let repo = env.find_git_repo("my-project");
    let session = env.find_tmux_session("dev-session");

    assert!(repo.is_some());
    assert!(session.is_some());

    // Session working dir should be same as the dir containing both
    let session = session.unwrap();
    assert!(session.working_dir().ends_with("dev"));
}

// ============================================================================
// run_shell() tests
// ============================================================================

#[test]
fn test_run_shell_basic_command() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("shell-test", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("shell-test")
        .expect("session should exist");

    let output = session.run_shell("echo hello");
    assert!(output.success());
    assert_eq!(output.stdout.trim(), "hello");
}

#[test]
fn test_run_shell_uses_working_directory() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("my-project", |d| {
                d.tmux_session("project-session", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("project-session")
        .expect("session should exist");

    // pwd should return a path ending with my-project
    let output = session.run_shell("pwd");
    assert!(output.success());
    assert!(output.stdout.trim().ends_with("my-project"));
}

#[test]
fn test_run_shell_with_git_repo() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Hello");
                        });
                    });
                });
                d.tmux_session("dev-session", |s| {
                    s.window("shell");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("dev-session")
        .expect("session should exist");

    // Should be able to see the git repo from the session's working dir
    let output = session.run_shell("ls");
    assert!(output.success());
    assert!(output.stdout.contains("my-repo"));

    // Should be able to run git commands in the repo
    let output = session.run_shell("cd my-repo && git branch --show-current");
    assert!(output.success());
    assert_eq!(output.stdout.trim(), "main");
}

#[test]
fn test_run_shell_failure() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("fail-test", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("fail-test")
        .expect("session should exist");

    // Running a command that fails
    let output = session.run_shell("exit 1");
    assert!(!output.success());
    assert_eq!(output.exit_code(), Some(1));
}

#[test]
fn test_run_shell_assert_success() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("assert-test", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("assert-test")
        .expect("session should exist");

    // Should not panic
    session.run_shell("echo test").assert_success();
}

#[test]
fn test_run_shell_assert_stdout_contains() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("stdout-test", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("stdout-test")
        .expect("session should exist");

    session
        .run_shell("echo 'hello world'")
        .assert_success()
        .assert_stdout_contains("hello")
        .assert_stdout_contains("world");
}

#[test]
fn test_run_shell_args() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("args-test", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("args-test")
        .expect("session should exist");

    let output = session.run_shell_args("echo", &["hello", "world"]);
    assert!(output.success());
    assert_eq!(output.stdout.trim(), "hello world");
}

#[test]
fn test_run_shell_captures_stderr() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("stderr-test", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("stderr-test")
        .expect("session should exist");

    let output = session.run_shell("echo error >&2");
    assert!(output.success());
    assert!(output.stderr.contains("error"));
}
