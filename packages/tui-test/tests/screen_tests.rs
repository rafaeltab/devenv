use tui_test::spawn_tui;

// ============================================================================
// Screen Capture Tests
// ============================================================================

#[test]
fn screen_returns_full_buffer() {
    let mut tui = spawn_tui("echo", &["test output"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    let screen = tui.screen();
    assert!(
        screen.contains("test output"),
        "Screen should contain the output"
    );
}

#[test]
fn screen_reflects_current_state() {
    let mut tui = spawn_tui("cat", &[]).spawn().expect("Failed to spawn");

    tui.wait_for_settle();

    let screen1 = tui.screen();

    tui.type_text("new text");
    tui.wait_for_settle();

    let screen2 = tui.screen();

    assert_ne!(screen1, screen2, "Screen should change after input");
    assert!(
        screen2.contains("new text"),
        "New screen should contain new text"
    );
}
