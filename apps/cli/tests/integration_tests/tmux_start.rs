use crate::common::{rafaeltab_descriptors::RafaeltabRootMixin, run_cli_with_tmux};
use test_descriptors::TestEnvironment;

#[test]
fn test_start_creates_sessions_from_workspace_config() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws_1", |_d| {});
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    // Manually create a config with a session
    let config = format!(
        r#"{{
        "workspaces": [{{
            "id": "ws_1",
            "name": "test ws",
            "root": "{}",
            "tags": []
        }}],
        "tmux": {{
            "sessions": [{{
                "workspace": "ws_1",
                "name": "test-ws",
                "windows": [{{ "name": "shell" }}]
            }}],
            "defaultWindows": []
        }}
    }}"#,
        env.root_path().join("ws_1").display()
    );

    std::fs::write(&config_path, config).expect("Failed to write config");

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
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws_2", |_d| {});
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let config = format!(
        r#"{{
        "workspaces": [{{
            "id": "ws_2",
            "name": "idempotent ws",
            "root": "{}",
            "tags": []
        }}],
        "tmux": {{
            "sessions": [{{
                "workspace": "ws_2",
                "name": "idempotent-ws",
                "windows": [{{ "name": "shell" }}]
            }}],
            "defaultWindows": []
        }}
    }}"#,
        env.root_path().join("ws_2").display()
    );

    std::fs::write(&config_path, config).expect("Failed to write config");

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
        root.rafaeltab_config(|_c| {});
        root.test_dir(|_td| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let config = format!(
        r#"{{
        "workspaces": [],
        "tmux": {{
            "sessions": [{{
                "path": "{}",
                "name": "path-session",
                "windows": [{{ "name": "shell" }}]
            }}],
            "defaultWindows": []
        }}
    }}"#,
        env.root_path().display()
    );

    std::fs::write(&config_path, config).expect("Failed to write config");

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
        root.rafaeltab_config(|_c| {});

        root.test_dir(|td| {
            td.dir("ws_m1", |_d| {});
            td.dir("ws_m2", |_d| {});
        });
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let config = format!(
        r#"{{
        "workspaces": [
            {{
                "id": "ws_m1",
                "name": "multi ws 1",
                "root": "{}",
                "tags": []
            }},
            {{
                "id": "ws_m2",
                "name": "multi ws 2",
                "root": "{}",
                "tags": []
            }}
        ],
        "tmux": {{
            "sessions": [
                {{
                    "workspace": "ws_m1",
                    "name": "multi-ws-1",
                    "windows": [{{ "name": "shell" }}]
                }},
                {{
                    "workspace": "ws_m2",
                    "name": "multi-ws-2",
                    "windows": [{{ "name": "shell" }}]
                }}
            ],
            "defaultWindows": []
        }}
    }}"#,
        env.root_path().join("ws_m1").display(),
        env.root_path().join("ws_m2").display()
    );

    std::fs::write(&config_path, config).expect("Failed to write config");

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
