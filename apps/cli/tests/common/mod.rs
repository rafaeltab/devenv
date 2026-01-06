pub mod rafaeltab_descriptors;

use std::process::Command;
use tui_test::{spawn_tui, TuiSession};

/// Helper function to run the CLI with specified arguments and config path.
/// Returns (stdout, stderr, success_status).
pub fn run_cli(args: &[&str], config_path: &str) -> (String, String, bool) {
    // Build args with --config flag prepended
    let mut full_args = vec!["--config", config_path];
    full_args.extend_from_slice(args);

    let output = Command::new(env!("CARGO_BIN_EXE_rafaeltab"))
        .args(&full_args)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to execute CLI with args {:?} and config {}: {}",
                args, config_path, e
            )
        });

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    (stdout, stderr, success)
}

/// Helper function to run the CLI with tmux socket isolation.
/// Returns (stdout, stderr, success_status).
pub fn run_cli_with_tmux(
    args: &[&str],
    config_path: &str,
    tmux_socket: &str,
) -> (String, String, bool) {
    // Build args with --config flag prepended
    let mut full_args = vec!["--config", config_path];
    full_args.extend_from_slice(args);

    let output = Command::new(env!("CARGO_BIN_EXE_rafaeltab"))
        .args(&full_args)
        .env("RAFAELTAB_TMUX_SOCKET", tmux_socket)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to execute CLI with args {:?}, config {}, and tmux socket {}: {}",
                args, config_path, tmux_socket, e
            )
        });

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let success = output.status.success();

    (stdout, stderr, success)
}

/// Helper function to run the CLI in TUI mode for interactive testing.
/// Returns a TuiSession that can be used to interact with the terminal.
pub fn run_cli_tui(args: &[&str], config_path: &str, tmux_socket: &str) -> TuiSession {
    // Build args with --config flag prepended
    let mut full_args = vec!["--config", config_path];
    full_args.extend_from_slice(args);

    spawn_tui(env!("CARGO_BIN_EXE_rafaeltab"), &full_args)
        .env("RAFAELTAB_TMUX_SOCKET", tmux_socket)
        .terminal_size(40, 120)
        .settle_timeout(300)
        .spawn()
        .expect("Failed to spawn TUI")
}
