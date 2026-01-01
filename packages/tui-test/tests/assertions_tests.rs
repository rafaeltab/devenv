use tui_test::spawn_tui;

// ============================================================================
// Text Match Assertion Tests
// ============================================================================

#[test]
fn assert_visible_succeeds() {
    let mut tui = spawn_tui("echo", &["visible text"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should not panic
    tui.find_text("visible text").assert_visible();
}

#[test]
#[should_panic(expected = "should be visible on screen")]
fn assert_visible_fails_with_message() {
    let mut tui = spawn_tui("echo", &["Hello"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should panic with helpful message
    tui.find_text("Goodbye").assert_visible();
}

#[test]
fn assert_not_visible_succeeds() {
    let mut tui = spawn_tui("echo", &["Hello"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should not panic
    tui.find_text("Goodbye").assert_not_visible();
}

#[test]
#[should_panic(expected = "should NOT be visible")]
fn assert_not_visible_fails() {
    let mut tui = spawn_tui("echo", &["Hello"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should panic
    tui.find_text("Hello").assert_not_visible();
}

#[test]
fn text_match_position_returns_coords() {
    let mut tui = spawn_tui("echo", &["Test"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    let pos = tui.find_text("Test").position();
    assert!(pos.is_some());

    let (row, col) = pos.unwrap();
    assert!(row < 40); // Within terminal bounds
    assert!(col < 120);
}

#[test]
fn text_match_is_snapshot() {
    let mut tui = spawn_tui("cat", &[]).spawn().expect("Failed to spawn");

    tui.wait_for_settle();

    tui.type_text("first");
    tui.wait_for_settle();

    // Capture the match at this point
    let first_match = tui.find_text("first");

    // Change the screen
    tui.type_text(" second");
    tui.wait_for_settle();

    // The original match should still reference the old state
    first_match.assert_visible(); // Should still work with captured snapshot
}
