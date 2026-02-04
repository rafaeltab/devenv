//! 5.1 Command Struct Tests
//!
//! Tests for the Command builder.

use std::path::PathBuf;
use test_descriptors::testers::Command;

/// Program is set correctly.
#[test]
fn command_new_sets_program() {
    let cmd = Command::new("my-program");
    assert_eq!(cmd.program(), "my-program");
}

/// Arguments are added.
#[test]
fn command_args_adds_arguments() {
    let cmd = Command::new("echo").args(&["hello", "world"]);
    let args = cmd.build_args();
    assert_eq!(args, vec!["hello", "world"]);
}

/// Environment variable added.
#[test]
fn command_env_adds_variable() {
    let cmd = Command::new("sh").env("MY_VAR", "my_value");
    let envs = cmd.build_env();
    assert_eq!(envs.get("MY_VAR"), Some(&"my_value".to_string()));
}

/// Working directory set.
#[test]
fn command_cwd_sets_directory() {
    let cmd = Command::new("pwd").cwd("/some/path");
    assert_eq!(cmd.cwd(), Some(PathBuf::from("/some/path")));
}

/// All args returned in order.
#[test]
fn command_build_args_returns_all() {
    let cmd = Command::new("test").args(&["a", "b"]).args(&["c", "d"]);
    let args = cmd.build_args();
    assert_eq!(args, vec!["a", "b", "c", "d"]);
}

/// All env vars returned.
#[test]
fn command_build_env_returns_all() {
    let cmd = Command::new("sh")
        .env("VAR1", "val1")
        .env("VAR2", "val2")
        .env("VAR3", "val3");

    let envs = cmd.build_env();
    assert_eq!(envs.len(), 3);
    assert_eq!(envs.get("VAR1"), Some(&"val1".to_string()));
    assert_eq!(envs.get("VAR2"), Some(&"val2".to_string()));
    assert_eq!(envs.get("VAR3"), Some(&"val3".to_string()));
}

/// Command chaining works.
#[test]
fn command_builder_chaining() {
    let cmd = Command::new("myapp")
        .args(&["--flag"])
        .env("CONFIG", "test")
        .cwd("/app")
        .args(&["subcommand"]);

    assert_eq!(cmd.program(), "myapp");
    assert_eq!(cmd.build_args(), vec!["--flag", "subcommand"]);
    assert_eq!(cmd.build_env().get("CONFIG"), Some(&"test".to_string()));
    assert_eq!(cmd.cwd(), Some(PathBuf::from("/app")));
}

/// PTY size configuration.
#[test]
fn command_pty_size_sets_dimensions() {
    let cmd = Command::new("tput").pty_size(30, 100);
    let (rows, cols) = cmd.pty_size();
    assert_eq!(rows, 30);
    assert_eq!(cols, 100);
}

/// Default PTY size.
#[test]
fn command_default_pty_size() {
    let cmd = Command::new("echo");
    let (rows, cols) = cmd.pty_size();
    // Default is typically 24x80
    assert_eq!(rows, 24);
    assert_eq!(cols, 80);
}
