use tui_test::spawn_tui;

// ============================================================================
// Basic Lifecycle Tests
// ============================================================================

#[test]
fn spawn_simple_command() {
    // Test that we can spawn a simple command and it completes
    let mut tui = spawn_tui("echo", &["hello"])
        .spawn()
        .expect("Failed to spawn echo");

    tui.wait_for_settle();
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn spawn_with_custom_env() {
    // Spawn with custom environment variable
    let mut tui = spawn_tui("sh", &["-c", "echo $TEST_VAR"])
        .env("TEST_VAR", "test_value")
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should be able to see the env var value in output
    tui.find_text("test_value").assert_visible();

    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn spawn_with_custom_terminal_size() {
    // Verify we can set custom terminal size
    let mut tui = spawn_tui("tput", &["cols"])
        .terminal_size(24, 80)
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // tput cols should output "80"
    tui.find_text("80").assert_visible();

    tui.expect_completion();
}

#[test]
fn expect_completion_returns_exit_code() {
    let mut tui = spawn_tui("sh", &["-c", "exit 42"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 42);
}

#[test]
fn expect_exit_code_matches() {
    let mut tui = spawn_tui("sh", &["-c", "exit 0"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();
    tui.expect_exit_code(0); // Should not panic
}

#[test]
#[should_panic(expected = "expected exit code")]
fn expect_exit_code_panics_on_mismatch() {
    let mut tui = spawn_tui("sh", &["-c", "exit 42"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();
    tui.expect_exit_code(0); // Should panic
}
