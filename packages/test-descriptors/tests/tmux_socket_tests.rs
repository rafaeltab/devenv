//! Tests for TmuxSocket helper methods

use std::path::PathBuf;
use test_descriptors::TmuxSocket;

#[test]
fn test_get_session_path_returns_path_for_existing_session() {
    let socket = TmuxSocket::new();

    // Create a session with specific working directory
    socket
        .run_tmux(&["new-session", "-d", "-s", "path-test", "-c", "/tmp"])
        .unwrap();

    // Should be able to get the path
    let path = socket.get_session_path("path-test").unwrap();
    assert_eq!(path, PathBuf::from("/tmp"));

    let _ = socket.kill_server();
}

#[test]
fn test_get_session_path_returns_none_for_nonexistent_session() {
    let socket = TmuxSocket::new();

    let path = socket.get_session_path("nonexistent");
    assert!(path.is_none());

    let _ = socket.kill_server();
}

#[test]
fn test_get_session_path_returns_actual_working_dir() {
    let socket = TmuxSocket::new();

    // Create session with different working dir
    socket
        .run_tmux(&["new-session", "-d", "-s", "wd-test", "-c", "/var"])
        .unwrap();

    let path = socket.get_session_path("wd-test").unwrap();
    assert_eq!(path, PathBuf::from("/var"));

    let _ = socket.kill_server();
}
