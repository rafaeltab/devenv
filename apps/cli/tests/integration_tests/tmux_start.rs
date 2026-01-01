use crate::common::{rafaeltab_descriptors::RafaeltabRootMixin, run_cli_with_tmux};
use test_descriptors::TestEnvironment;

#[test]
fn test_start_creates_sessions_from_workspace_config() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabDirMixin;

        root.rafaeltab_config(|c| {
            c.tmux_session("ws_1", Some("test-ws"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("ws_1", |d| {
                d.rafaeltab_workspace("ws_1", "test ws", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Run CLI with isolated tmux socket
    let (stdout, stderr, success) = run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(
        success,
        "Command failed:\nstdout: {}\nstderr: {}",
        stdout, stderr
    );

    // Session name comes from the config, defaults to workspace name if not specified
    assert!(
        env.tmux().session_exists("test ws"),
        "Expected session 'test ws' to be created. Found sessions: {:?}",
        env.tmux().list_sessions()
    );
}

#[test]
fn test_start_is_idempotent() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabDirMixin;

        root.rafaeltab_config(|c| {
            c.tmux_session("ws_2", Some("idempotent-ws"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("ws_2", |d| {
                d.rafaeltab_workspace("ws_2", "idempotent ws", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Run tmux start twice
    for i in 1..=2 {
        let (stdout, stderr, success) = run_cli_with_tmux(
            &["tmux", "start"],
            config_path.to_str().unwrap(),
            env.tmux_socket(),
        );

        assert!(
            success,
            "Command failed on run {i}:\nstdout: {}\nstderr: {}",
            stdout, stderr
        );
    }

    let sessions = env.tmux().list_sessions().expect("Failed to list sessions");
    let count = sessions.iter().filter(|s| *s == "idempotent ws").count();
    assert_eq!(
        count, 1,
        "Expected exactly one session, but found {count} sessions named 'idempotent ws'. Sessions: {:?}",
        sessions
    );
}

#[test]
fn test_start_with_empty_config() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let (stdout, stderr, success) = run_cli_with_tmux(
        &["tmux", "start"],
        env.context().config_path().unwrap().to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(
        success,
        "Command failed:\nstdout: {}\nstderr: {}",
        stdout, stderr
    );

    let sessions = env.tmux().list_sessions().expect("Failed to list sessions");
    assert!(
        sessions.is_empty(),
        "Expected no sessions to be created, but found: {:?}",
        sessions
    );
}

#[test]
fn test_start_creates_path_based_session() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|_td| {});
    })
    .create();

    let test_path = env.root_path().to_string_lossy().to_string();

    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(move |c| {
            c.tmux_session_path(&test_path, "path-session", &[("shell", None)]);
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let (stdout, stderr, success) = run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(
        success,
        "Command failed:\nstdout: {}\nstderr: {}",
        stdout, stderr
    );

    let sessions = env.tmux().list_sessions().expect("Failed to list sessions");
    assert!(
        !sessions.is_empty(),
        "Expected at least one session to be created"
    );
}

#[test]
fn test_start_creates_multiple_sessions() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabDirMixin;

        root.rafaeltab_config(|c| {
            c.tmux_session("ws_m1", Some("multi-ws-1"), &[("shell", None)]);
            c.tmux_session("ws_m2", Some("multi-ws-2"), &[("shell", None)]);
        });

        root.test_dir(|td| {
            td.dir("ws_m1", |d| {
                d.rafaeltab_workspace("ws_m1", "multi ws 1", |_w| {});
            });
            td.dir("ws_m2", |d| {
                d.rafaeltab_workspace("ws_m2", "multi ws 2", |_w| {});
            });
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let (stdout, stderr, success) = run_cli_with_tmux(
        &["tmux", "start"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    assert!(
        success,
        "Command failed:\nstdout: {}\nstderr: {}",
        stdout, stderr
    );

    let sessions = env.tmux().list_sessions().expect("Failed to list sessions");
    assert!(
        sessions.contains(&"multi ws 1".to_string()),
        "Expected session 'multi ws 1' to be created. Found sessions: {:?}",
        sessions
    );
    assert!(
        sessions.contains(&"multi ws 2".to_string()),
        "Expected session 'multi ws 2' to be created. Found sessions: {:?}",
        sessions
    );
}
