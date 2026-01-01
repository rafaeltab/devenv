use super::helpers::{TestContext, TmuxTestContext};
use std::process::Command;

#[test]
fn test_start_creates_sessions_from_workspace_config() {
    let tmux_ctx = TmuxTestContext::new().expect("Failed to create tmux test context");
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
        tmux_ctx.temp_dir_path().display()
    );

    let config_ctx = TestContext::new(&config).expect("Failed to create config context");

    // Run CLI with isolated tmux socket
    let output = Command::new("target/debug/rafaeltab")
        .args(["--config", config_ctx.config_path()])
        .env("RAFAELTAB_TMUX_SOCKET", tmux_ctx.socket_name())
        .args(["tmux", "start"])
        .output()
        .expect("Failed to run CLI");

    assert!(
        output.status.success(),
        "Command failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let sessions = tmux_ctx.list_sessions();
    // Session name comes from the config, defaults to workspace name if not specified
    assert!(
        sessions.contains(&"test ws".to_string()),
        "Expected session 'test ws' to be created. Found sessions: {:?}",
        sessions
    );
}

#[test]
fn test_start_is_idempotent() {
    let tmux_ctx = TmuxTestContext::new().expect("Failed to create tmux test context");
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
        tmux_ctx.temp_dir_path().display()
    );

    let config_ctx = TestContext::new(&config).expect("Failed to create config context");

    // Run tmux start twice
    for i in 1..=2 {
        let output = Command::new("target/debug/rafaeltab")
            .args(["--config", config_ctx.config_path()])
            .env("RAFAELTAB_TMUX_SOCKET", tmux_ctx.socket_name())
            .args(["tmux", "start"])
            .output()
            .expect("Failed to run CLI");

        assert!(
            output.status.success(),
            "Command failed on run {i}:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let sessions = tmux_ctx.list_sessions();
    let count = sessions
        .iter()
        .filter(|s| s.as_str() == "idempotent ws")
        .count();
    assert_eq!(
        count, 1,
        "Expected exactly one session, but found {count} sessions named 'idempotent ws'. Sessions: {:?}",
        sessions
    );
}

#[test]
fn test_start_with_empty_config() {
    let tmux_ctx = TmuxTestContext::new().expect("Failed to create tmux test context");
    let config = r#"{
        "workspaces": [],
        "tmux": {
            "sessions": [],
            "defaultWindows": []
        }
    }"#;

    let config_ctx = TestContext::new(config).expect("Failed to create config context");

    let output = Command::new("target/debug/rafaeltab")
        .args(["--config", config_ctx.config_path()])
        .env("RAFAELTAB_TMUX_SOCKET", tmux_ctx.socket_name())
        .args(["tmux", "start"])
        .output()
        .expect("Failed to run CLI");

    assert!(
        output.status.success(),
        "Command failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let sessions = tmux_ctx.list_sessions();
    assert!(
        sessions.is_empty(),
        "Expected no sessions to be created, but found: {:?}",
        sessions
    );
}

#[test]
fn test_start_creates_path_based_session() {
    let tmux_ctx = TmuxTestContext::new().expect("Failed to create tmux test context");
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
        tmux_ctx.temp_dir_path().display()
    );

    let config_ctx = TestContext::new(&config).expect("Failed to create config context");

    let output = Command::new("target/debug/rafaeltab")
        .args(["--config", config_ctx.config_path()])
        .env("RAFAELTAB_TMUX_SOCKET", tmux_ctx.socket_name())
        .args(["tmux", "start"])
        .output()
        .expect("Failed to run CLI");

    assert!(
        output.status.success(),
        "Command failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let sessions = tmux_ctx.list_sessions();
    assert!(
        !sessions.is_empty(),
        "Expected at least one session to be created"
    );
}

#[test]
fn test_start_creates_multiple_sessions() {
    let tmux_ctx = TmuxTestContext::new().expect("Failed to create tmux test context");
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
        tmux_ctx.temp_dir_path().display(),
        tmux_ctx.temp_dir_path().display()
    );

    let config_ctx = TestContext::new(&config).expect("Failed to create config context");

    let output = Command::new("target/debug/rafaeltab")
        .args(["--config", config_ctx.config_path()])
        .env("RAFAELTAB_TMUX_SOCKET", tmux_ctx.socket_name())
        .args(["tmux", "start"])
        .output()
        .expect("Failed to run CLI");

    assert!(
        output.status.success(),
        "Command failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let sessions = tmux_ctx.list_sessions();
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
