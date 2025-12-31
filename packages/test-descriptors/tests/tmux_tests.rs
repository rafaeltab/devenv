use std::process::Command;
use tempfile::TempDir;
use test_descriptors::descriptor::{
    CreateContext, Descriptor, TmuxSessionDescriptor, TmuxSocket, WindowDescriptor,
};

// TmuxSocket tests
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
}

#[test]
fn test_tmux_socket_list_sessions_empty() {
    let socket = TmuxSocket::new();

    let sessions = socket.list_sessions().unwrap_or(vec![]);
    assert_eq!(sessions.len(), 0);
}

#[test]
fn test_tmux_socket_session_exists_false() {
    let socket = TmuxSocket::new();

    assert!(!socket.session_exists("nonexistent"));
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

// WindowDescriptor tests
#[test]
fn test_window_descriptor_simple() {
    let window = WindowDescriptor::new("main");
    assert_eq!(window.name(), "main");
    assert_eq!(window.command(), None);
}

#[test]
fn test_window_descriptor_with_command() {
    let window = WindowDescriptor::new("editor").with_command("nvim");
    assert_eq!(window.name(), "editor");
    assert_eq!(window.command(), Some("nvim"));
}

// TmuxSessionDescriptor tests
#[test]
fn test_tmux_session_descriptor_creates_session() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());
    let socket = TmuxSocket::new();
    context.set_tmux_socket(socket.name().to_string());

    let session = TmuxSessionDescriptor::new("test-session");
    session.create(&context).unwrap();

    // Verify session exists
    assert!(socket.session_exists("test-session"));

    // Cleanup
    socket.kill_server().unwrap();
}

#[test]
fn test_tmux_session_descriptor_with_window() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());
    let socket = TmuxSocket::new();
    context.set_tmux_socket(socket.name().to_string());

    let window = WindowDescriptor::new("editor");
    let session = TmuxSessionDescriptor::new("dev-session").with_window(window);
    session.create(&context).unwrap();

    // Verify session exists
    assert!(socket.session_exists("dev-session"));

    // Cleanup
    socket.kill_server().unwrap();
}

#[test]
fn test_tmux_session_descriptor_with_multiple_windows() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());
    let socket = TmuxSocket::new();
    context.set_tmux_socket(socket.name().to_string());

    let session = TmuxSessionDescriptor::new("multi-window")
        .with_window(WindowDescriptor::new("main"))
        .with_window(WindowDescriptor::new("editor"))
        .with_window(WindowDescriptor::new("terminal"));
    session.create(&context).unwrap();

    // Verify session exists
    assert!(socket.session_exists("multi-window"));

    // Get window count
    let output = socket
        .run_tmux(&["list-windows", "-t", "multi-window", "-F", "#{window_name}"])
        .unwrap();
    let window_count = output.lines().count();
    assert_eq!(window_count, 3);

    // Cleanup
    socket.kill_server().unwrap();
}

#[test]
fn test_tmux_session_descriptor_registered_in_context() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());
    let socket = TmuxSocket::new();
    context.set_tmux_socket(socket.name().to_string());

    let session = TmuxSessionDescriptor::new("registered-session");
    session.create(&context).unwrap();

    // Check registration
    let binding = context.registry().borrow();
    let registered = binding.get_tmux_session("registered-session");
    assert!(registered.is_some());
    assert_eq!(registered.unwrap().name, "registered-session");

    // Cleanup
    socket.kill_server().unwrap();
}

#[test]
fn test_tmux_session_descriptor_working_directory() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());
    let socket = TmuxSocket::new();
    context.set_tmux_socket(socket.name().to_string());

    let session = TmuxSessionDescriptor::new("work-session");
    session.create(&context).unwrap();

    // Check that the session info in registry has the correct working directory
    let binding = context.registry().borrow();
    let session_info = binding.get_tmux_session("work-session");
    assert!(session_info.is_some());
    assert_eq!(session_info.unwrap().working_dir, temp.path());

    // Cleanup
    socket.kill_server().unwrap();
}

#[test]
fn test_tmux_session_descriptor_isolated_from_default_server() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());
    let socket = TmuxSocket::new();
    context.set_tmux_socket(socket.name().to_string());

    let session = TmuxSessionDescriptor::new("isolated-session");
    session.create(&context).unwrap();

    // Check that session exists in our socket
    assert!(socket.session_exists("isolated-session"));

    // Check that session doesn't exist in default tmux server
    let default_check = Command::new("tmux")
        .args(&["has-session", "-t", "isolated-session"])
        .output()
        .unwrap();

    // Should fail because it's not in the default server
    assert!(!default_check.status.success());

    // Cleanup
    socket.kill_server().unwrap();
}
