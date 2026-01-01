use tui_test::spawn_tui;

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn builder_default_values() {
    let tui = spawn_tui("echo", &["test"])
        .spawn()
        .expect("Failed to spawn");

    // Just verify it builds with defaults
    drop(tui);
}

#[test]
fn builder_precedence_explicit_over_env() {
    // Set env var
    std::env::set_var("TUI_TEST_SETTLE_MS", "999");

    // Explicit value should override env var - this just verifies the builder accepts it
    // The actual timeout behavior is tested in wait_settle_tests.rs
    let tui = spawn_tui("echo", &["test"])
        .settle_timeout(100) // Explicit
        .spawn()
        .expect("Failed to spawn");

    // Just verify it built successfully with explicit config
    drop(tui);

    std::env::remove_var("TUI_TEST_SETTLE_MS");
}
