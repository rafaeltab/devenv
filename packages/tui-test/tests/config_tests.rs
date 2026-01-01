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

    // Explicit value should override env var
    let mut tui = spawn_tui("echo", &["test"])
        .settle_timeout(100) // Explicit
        .spawn()
        .expect("Failed to spawn");

    // Should use 100ms, not 999ms from env
    let start = std::time::Instant::now();
    tui.wait_for_settle();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 500, "Should use explicit timeout");

    std::env::remove_var("TUI_TEST_SETTLE_MS");
}
