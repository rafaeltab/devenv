use tui_test::spawn_tui;

// ============================================================================
// Wait and Settle Tests
// ============================================================================

#[test]
fn wait_ms_delays_execution() {
    let mut tui = spawn_tui("echo", &["test"])
        .spawn()
        .expect("Failed to spawn");

    let start = std::time::Instant::now();
    tui.wait_ms(100);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() >= 100,
        "wait_ms should delay at least 100ms"
    );
}

#[test]
fn wait_for_settle_detects_stable_screen() {
    // Echo outputs once and stops - should settle
    let mut tui = spawn_tui("echo", &["stable"])
        .settle_timeout(50)
        .spawn()
        .expect("Failed to spawn");

    let start = std::time::Instant::now();
    tui.wait_for_settle();
    let elapsed = start.elapsed();

    // Should settle in reasonable time (not hang indefinitely)
    // Note: settle time = timeout after screen stops changing + process exit time
    assert!(
        elapsed.as_millis() < 2000,
        "Should settle within 2 seconds for stable output"
    );

    // Verify content appeared
    tui.find_text("stable").assert_visible();
}

#[test]
fn wait_for_settle_with_custom_timeout() {
    let mut tui = spawn_tui("echo", &["test"])
        .spawn()
        .expect("Failed to spawn");

    // Use custom timeout of 100ms instead of default 300ms
    tui.wait_for_settle_ms(100, 1000);

    // Should complete without error
    tui.expect_completion();
}

#[test]
fn wait_for_settle_max_wait_timeout() {
    // Create a process that continuously outputs
    let mut tui = spawn_tui("sh", &["-c", "while true; do echo tick; sleep 0.05; done"])
        .spawn()
        .expect("Failed to spawn");

    let start = std::time::Instant::now();

    // Should timeout after max_wait (200ms), not wait forever
    tui.wait_for_settle_ms(50, 200);

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() >= 200,
        "Should wait at least max_wait time"
    );
    assert!(elapsed.as_millis() < 500, "Should timeout after max_wait");
}
