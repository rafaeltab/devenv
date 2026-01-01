use tui_test::{spawn_tui, Key};

// ============================================================================
// Text Input Tests
// ============================================================================

#[test]
fn type_text_sends_to_pty() {
    // Use 'cat' which echoes input back
    let mut tui = spawn_tui("cat", &[]).spawn().expect("Failed to spawn");

    tui.wait_for_settle();
    tui.type_text("hello world");
    tui.wait_for_settle();

    // cat should echo back what we typed
    tui.find_text("hello world").assert_visible();
}

#[test]
fn press_key_enter() {
    let mut tui = spawn_tui("cat", &[]).spawn().expect("Failed to spawn");

    tui.wait_for_settle();
    tui.type_text("line1");
    tui.press_key(Key::Enter);
    tui.wait_for_settle();

    tui.find_text("line1").assert_visible();
}

#[test]
fn press_key_arrows() {
    // Test with a simple read command
    let mut tui = spawn_tui("sh", &["-c", "read input; echo $input"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();
    tui.type_text("test");
    tui.press_key(Key::Left);
    tui.press_key(Key::Right);
    tui.press_key(Key::Enter);
    tui.wait_for_settle();

    // Should have echoed the input
    tui.find_text("test").assert_visible();
}

#[test]
fn press_key_esc() {
    let mut tui = spawn_tui("cat", &[]).spawn().expect("Failed to spawn");

    tui.wait_for_settle();
    tui.press_key(Key::Esc);
    tui.wait_ms(50);

    // Esc shouldn't crash the session
}

#[test]
fn press_key_backspace() {
    let mut tui = spawn_tui("cat", &[]).spawn().expect("Failed to spawn");

    tui.wait_for_settle();
    tui.type_text("hello");
    tui.press_key(Key::Backspace);
    tui.press_key(Key::Enter);
    tui.wait_for_settle();

    // Should see "hell" (last char deleted)
    tui.find_text("hell").assert_visible();
}

#[test]
fn send_keys_ctrl_c() {
    // Start a long-running process
    let mut tui = spawn_tui("sleep", &["10"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Send Ctrl+C to interrupt
    tui.send_keys(&[Key::Ctrl, Key::Char('c')]);
    tui.wait_ms(100);

    // Process should have been interrupted (non-zero exit code)
    let exit_code = tui.expect_completion();
    assert_ne!(exit_code, 0, "Ctrl+C should interrupt the process");
}

#[test]
#[should_panic(expected = "at least one non-modifier key")]
fn send_keys_requires_non_modifier() {
    let mut tui = spawn_tui("cat", &[]).spawn().expect("Failed to spawn");

    tui.wait_for_settle();

    // Should panic - only modifiers, no regular key
    tui.send_keys(&[Key::Ctrl, Key::Shift]);
}

#[test]
#[should_panic(expected = "only send one non-modifier key")]
fn send_keys_single_regular_key_only() {
    let mut tui = spawn_tui("cat", &[]).spawn().expect("Failed to spawn");

    tui.wait_for_settle();

    // Should panic - multiple regular keys
    tui.send_keys(&[Key::Char('a'), Key::Char('b')]);
}
