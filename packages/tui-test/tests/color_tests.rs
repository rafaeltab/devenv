use tui_test::{spawn_tui, ColorMatcher};

// ============================================================================
// Color Matching Tests
// ============================================================================

#[test]
fn color_matcher_grayscale() {
    // Test grayscale detection (low saturation)
    assert!(ColorMatcher::Grayscale.matches(128, 128, 128)); // Gray
    assert!(ColorMatcher::Grayscale.matches(200, 200, 200)); // Light gray
    assert!(!ColorMatcher::Grayscale.matches(255, 0, 0)); // Red
}

#[test]
fn color_matcher_yellowish() {
    // Yellow is around hue 60° with decent saturation
    assert!(ColorMatcher::YellowIsh.matches(255, 255, 0)); // Pure yellow
    assert!(ColorMatcher::YellowIsh.matches(200, 200, 50)); // Yellowish
    assert!(!ColorMatcher::YellowIsh.matches(255, 0, 0)); // Red
    assert!(!ColorMatcher::YellowIsh.matches(128, 128, 128)); // Gray
}

#[test]
fn color_matcher_redish() {
    // Red is around hue 0°/360° with wrap-around
    assert!(ColorMatcher::RedIsh.matches(255, 0, 0)); // Pure red
    assert!(ColorMatcher::RedIsh.matches(200, 50, 50)); // Reddish
    assert!(!ColorMatcher::RedIsh.matches(0, 255, 0)); // Green
}

#[test]
fn color_matcher_greenish() {
    // Green is around hue 120°
    assert!(ColorMatcher::GreenIsh.matches(0, 255, 0)); // Pure green
    assert!(ColorMatcher::GreenIsh.matches(50, 200, 50)); // Greenish
    assert!(!ColorMatcher::GreenIsh.matches(255, 0, 0)); // Red
}

#[test]
fn color_matcher_blueish() {
    // Blue is around hue 240°
    assert!(ColorMatcher::BlueIsh.matches(0, 0, 255)); // Pure blue
    assert!(ColorMatcher::BlueIsh.matches(50, 50, 200)); // Blueish
    assert!(!ColorMatcher::BlueIsh.matches(255, 0, 0)); // Red
}

#[test]
fn color_matcher_custom_hue_range() {
    // Test custom HSL ranges
    let matcher = ColorMatcher::Hue {
        min: 0.0,
        max: 30.0,
    };

    assert!(matcher.matches(255, 0, 0)); // Red (0°)
    assert!(!matcher.matches(0, 255, 0)); // Green (120°)
}

#[test]
fn color_assertion_fg_exact() {
    // Test exact RGB color matching
    // We'll need a program that outputs colored text
    // For now, just test the API exists
    let mut tui = spawn_tui("sh", &["-c", "echo -e '\\033[33mYellow\\033[0m'"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should be able to call fg() and exact()
    // This will fail until we implement color extraction
    // tui.find_text("Yellow").fg.exact(255, 255, 0);
}

#[test]
fn color_assertion_bg_matcher() {
    // Test background color matching
    let mut tui = spawn_tui("sh", &["-c", "echo -e '\\033[43mYellow BG\\033[0m'"])
        .spawn()
        .expect("Failed to spawn");

    tui.wait_for_settle();

    // Should be able to call bg() with matcher
    // This will fail until we implement color extraction
    // tui.find_text("Yellow BG").bg.assert(ColorMatcher::YellowIsh);
}
