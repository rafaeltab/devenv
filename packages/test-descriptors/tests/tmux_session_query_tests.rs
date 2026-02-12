//! Tests for verifying TestEnvironment tmux query methods check actual tmux sessions

use test_descriptors::{TestEnvironment, TmuxSocket};

#[test]
fn test_find_tmux_session_returns_none_for_registry_only_session() {
    // Create environment but DON'T call .create() so no actual tmux session is created
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.tmux_session("phantom-session", |s| {
                s.window("main");
            });
        });
    });
    // Note: NOT calling .create() - registry has entry but no actual tmux session

    // This should return None because the session doesn't actually exist in tmux
    let session = env.find_tmux_session("phantom-session");
    assert!(
        session.is_none(),
        "find_tmux_session should return None when session only exists in registry, not in actual tmux"
    );
}

#[test]
fn test_find_tmux_session_returns_some_for_actual_session() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.tmux_session("real-session", |s| {
                s.window("main");
            });
        });
    })
    .create(); // Actually creates the tmux session

    // This should return Some because the session actually exists in tmux
    let session = env.find_tmux_session("real-session");
    assert!(
        session.is_some(),
        "find_tmux_session should return Some when session exists in actual tmux"
    );

    // And the session should report as existing
    assert!(session.unwrap().exists());
}

#[test]
fn test_list_tmux_sessions_returns_actual_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.tmux_session("session-alpha", |s| s.window("main"));
            td.tmux_session("session-beta", |s| s.window("main"));
        });
    })
    .create();

    let sessions = env.list_tmux_sessions();
    let session_names: Vec<&str> = sessions.iter().map(|s| s.name()).collect();

    assert_eq!(sessions.len(), 2);
    assert!(session_names.contains(&"session-alpha"));
    assert!(session_names.contains(&"session-beta"));
}

#[test]
fn test_list_tmux_sessions_empty_when_no_sessions() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|_td| {
            // No tmux sessions defined
        });
    })
    .create();

    let sessions = env.list_tmux_sessions();
    assert!(sessions.is_empty());
}

#[test]
fn test_list_tmux_sessions_ignores_registry_only() {
    // Create env but don't call .create()
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.tmux_session("phantom", |s| s.window("main"));
        });
    });

    // Should return empty because no actual tmux sessions exist
    let sessions = env.list_tmux_sessions();
    assert!(
        sessions.is_empty(),
        "list_tmux_sessions should only return actual tmux sessions, not registry entries"
    );
}

#[test]
fn test_tmux_session_ref_exists_checks_actual_tmux() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.tmux_session("test", |s| s.window("main"));
        });
    })
    .create();

    let session = env.find_tmux_session("test").unwrap();
    assert!(session.exists());

    // Kill the session directly
    let socket = TmuxSocket::from_name(env.tmux_socket().to_string());
    socket.run_tmux(&["kill-session", "-t", "test"]).unwrap();

    // Session ref should now report as not existing
    assert!(
        !session.exists(),
        "exists() should return false after session is killed"
    );
}

#[test]
fn test_tmux_session_ref_windows_checks_actual_tmux() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.tmux_session("test", |s| {
                s.window("first");
                s.window("second");
            });
        });
    })
    .create();

    let session = env.find_tmux_session("test").unwrap();
    let windows = session.windows();
    assert_eq!(windows.len(), 2);

    // Add a window directly via tmux
    let socket = TmuxSocket::from_name(env.tmux_socket().to_string());
    socket
        .run_tmux(&["new-window", "-t", "test", "-n", "third"])
        .unwrap();

    // Should now see 3 windows
    let windows = session.windows();
    assert_eq!(windows.len(), 3);
    assert!(windows.contains(&"third".to_string()));
}

#[test]
fn test_tmux_session_lifecycle() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.tmux_session("lifecycle", |s| s.window("main"));
        });
    })
    .create();

    // 1. Session exists and is findable
    assert!(env.find_tmux_session("lifecycle").is_some());
    assert_eq!(env.list_tmux_sessions().len(), 1);

    // 2. Kill session externally
    let socket = TmuxSocket::from_name(env.tmux_socket().to_string());
    socket
        .run_tmux(&["kill-session", "-t", "lifecycle"])
        .unwrap();

    // 3. Session is no longer findable
    assert!(env.find_tmux_session("lifecycle").is_none());
    assert!(env.list_tmux_sessions().is_empty());
}

#[test]
fn test_find_tmux_session_finds_external_session() {
    use std::path::PathBuf;

    let env = TestEnvironment::describe(|root| {
        root.test_dir(|_td| {
            // No tmux sessions defined in descriptor
        });
    })
    .create();

    // Create session directly via tmux socket (bypassing descriptor system)
    let socket = TmuxSocket::from_name(env.tmux_socket().to_string());
    socket
        .run_tmux(&["new-session", "-d", "-s", "external-session", "-c", "/tmp"])
        .unwrap();

    // Should still be findable even though not in registry
    let session = env.find_tmux_session("external-session");
    assert!(
        session.is_some(),
        "find_tmux_session should find external sessions not in registry"
    );

    // Should report correct working directory from tmux
    let session = session.unwrap();
    assert_eq!(session.working_dir(), PathBuf::from("/tmp"));
    assert!(session.exists());
}

#[test]
fn test_find_tmux_session_prefers_registry_working_dir() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                // Session created through descriptor - registry has specific path
                d.tmux_session("managed-session", |s| s.window("main"));
            });
        });
    })
    .create();

    // Should find the session and have the registry's working dir
    let session = env.find_tmux_session("managed-session").unwrap();
    assert!(session.working_dir().ends_with("workspace"));
}

#[test]
fn test_list_tmux_sessions_includes_external_sessions() {
    use std::path::PathBuf;

    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.tmux_session("managed", |s| s.window("main"));
        });
    })
    .create();

    // Create an external session
    let socket = TmuxSocket::from_name(env.tmux_socket().to_string());
    socket
        .run_tmux(&["new-session", "-d", "-s", "external", "-c", "/var"])
        .unwrap();

    // list_tmux_sessions should include both
    let sessions = env.list_tmux_sessions();
    let names: Vec<&str> = sessions.iter().map(|s| s.name()).collect();

    assert_eq!(sessions.len(), 2);
    assert!(names.contains(&"managed"));
    assert!(names.contains(&"external"));

    // Check external session has correct path
    let external = sessions.iter().find(|s| s.name() == "external").unwrap();
    assert_eq!(external.working_dir(), PathBuf::from("/var"));
}

#[test]
fn test_list_tmux_sessions_returns_external_only_when_no_registry() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|_td| {
            // No tmux sessions in descriptor
        });
    })
    .create();

    // Create only external sessions
    let socket = TmuxSocket::from_name(env.tmux_socket().to_string());
    socket
        .run_tmux(&["new-session", "-d", "-s", "ext1", "-c", "/tmp"])
        .unwrap();
    socket
        .run_tmux(&["new-session", "-d", "-s", "ext2", "-c", "/var"])
        .unwrap();

    let sessions = env.list_tmux_sessions();
    assert_eq!(sessions.len(), 2);

    let names: Vec<&str> = sessions.iter().map(|s| s.name()).collect();
    assert!(names.contains(&"ext1"));
    assert!(names.contains(&"ext2"));
}

#[test]
fn test_external_session_ref_methods_work() {
    use std::path::PathBuf;

    let env = TestEnvironment::describe(|root| {
        root.test_dir(|_td| {
            // No descriptor sessions
        });
    })
    .create();

    // Create external session
    let socket = TmuxSocket::from_name(env.tmux_socket().to_string());
    socket
        .run_tmux(&["new-session", "-d", "-s", "external", "-c", "/tmp"])
        .unwrap();
    socket
        .run_tmux(&["new-window", "-t", "external", "-n", "editor"])
        .unwrap();

    let session = env.find_tmux_session("external").unwrap();

    // All TmuxSessionRef methods should work
    assert!(session.exists());
    assert_eq!(session.name(), "external");
    assert_eq!(session.working_dir(), PathBuf::from("/tmp"));

    let windows = session.windows();
    assert_eq!(windows.len(), 2); // default + editor
    assert!(session.has_window("editor"));

    // run_shell should work with tmux working dir
    // Note: On macOS /tmp is a symlink to /private/tmp, so we check suffix
    let output = session.run_shell("pwd");
    assert!(output.success());
    assert!(output.stdout.trim().ends_with("tmp"));
}
