use tui_test::spawn_tui;

// ============================================================================
// Text Finding Tests
// ============================================================================

#[test]
fn find_text_returns_position() {
    let mut tui = spawn_tui("echo", &["Hello World"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    let text_match = tui.find_text("Hello");
    assert!(text_match.position().is_some(), "Should find 'Hello'");
}

#[test]
fn find_text_not_found_returns_none() {
    let mut tui = spawn_tui("echo", &["Hello"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    let text_match = tui.find_text("Goodbye");
    assert!(text_match.position().is_none(), "Should not find 'Goodbye'");
}

#[test]
#[should_panic(expected = "found multiple occurrences")]
fn find_text_panics_on_multiple_matches() {
    let mut tui = spawn_tui("sh", &["-c", "echo 'test'; echo 'test'"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should panic - "test" appears twice
    tui.find_text("test");
}

#[test]
fn find_all_text_returns_all_positions() {
    let mut tui = spawn_tui("sh", &["-c", "echo 'test'; echo 'test'"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    let matches = tui.find_all_text("test");
    assert_eq!(matches.len(), 2, "Should find both occurrences of 'test'");
}

#[test]
fn find_all_text_empty_when_not_found() {
    let mut tui = spawn_tui("echo", &["Hello"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    let matches = tui.find_all_text("Goodbye");
    assert!(matches.is_empty(), "Should return empty vec when not found");
}

#[test]
fn find_text_exact_match_only() {
    let mut tui = spawn_tui("echo", &["Hello World"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should find exact substring
    tui.find_text("Hello").assert_visible();
    tui.find_text("World").assert_visible();
    tui.find_text("Hello World").assert_visible();
}

#[test]
fn find_text_case_sensitive() {
    let mut tui = spawn_tui("echo", &["Hello"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    tui.find_text("Hello").assert_visible();
    tui.find_text("hello").assert_not_visible(); // Different case
}
