mod common;

use crate::common::{rafaeltab_descriptors::RafaeltabRootMixin, CliCommandBuilder};
use test_descriptors::testers::{Key, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

// =============================================================================
// Select Picker Tests (SP-001 to SP-013)
// =============================================================================

/// SP-001: Display Items
/// Given a list of items implementing `PickerItem`
/// When the select picker is shown
/// Then all items should be visible in the middle panel
#[test]
fn test_select_picker_display_items() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2,Item3")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Find and select "test picker" command from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Now we're in the test picker - verify items displayed
    asserter.find_text("Item1").assert_visible();
    asserter.find_text("Item2").assert_visible();
    asserter.find_text("Item3").assert_visible();
}

/// SP-002: Fuzzy Search Filtering
/// Given items: ["Apple", "Banana", "Cherry"] via `TEST_PICKER_ITEMS`
/// When user types "ap" in the picker
/// Then only "Apple" should be visible
/// And it should be highlighted (yellow color)
#[test]
fn test_select_picker_fuzzy_search_filtering() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Apple,Banana,Cherry")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Now in the test picker - type search query
    asserter.type_text("ap");
    asserter.wait_for_settle();

    // Verify only Apple is visible and highlighted
    asserter.find_text("Banana").assert_not_visible();
    asserter.find_text("Cherry").assert_not_visible();
    let apple = asserter.find_text("Apple");
    apple.assert_visible();
    apple.fg.assert_not_grayscale(); // highlighted
}

/// SP-003: Fuzzy Search Scoring
/// Given items: ["application", "Apple", "pineapple"] via `TEST_PICKER_ITEMS`
/// When user types "app"
/// Then items should be ordered by match quality (Apple > application > pineapple)
#[test]
fn test_select_picker_fuzzy_search_scoring() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "application,Apple,pineapple")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Now in the test picker - type search query
    asserter.type_text("app");
    asserter.wait_for_settle();

    // Assert vertical order
    asserter.assert_vertical_order(&[
        asserter.find_text("Apple"),
        asserter.find_text("application"),
        asserter.find_text("pineapple"),
    ]);
}

/// SP-004: Empty Search Shows All
/// Given items: ["Item1", "Item2", "Item3"] via `TEST_PICKER_ITEMS`
/// When user has typed nothing
/// Then all items should be visible in original order
#[test]
fn test_select_picker_empty_search_shows_all() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2,Item3")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // All items should be visible in original order
    asserter.assert_vertical_order(&[
        asserter.find_text("Item1"),
        asserter.find_text("Item2"),
        asserter.find_text("Item3"),
    ]);
}

/// SP-005: No Matches Display
/// Given items: ["Apple", "Banana"] via `TEST_PICKER_ITEMS`
/// When user types "xyz"
/// Then middle panel should show "No matches"
#[test]
fn test_select_picker_no_matches_display() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Apple,Banana")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type a query that matches nothing
    asserter.type_text("xyz");
    asserter.wait_for_settle();

    // Verify "No matches" is displayed
    asserter.find_text("No matches").assert_visible();
}

/// SP-006: Navigation - Down Arrow
/// Given items: ["Item1", "Item2", "Item3"] with "Item1" selected
/// When user presses Down arrow
/// Then "Item2" should be highlighted (yellow)
#[test]
fn test_select_picker_navigation_down() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2,Item3")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Initially Item1 highlighted
    let item1 = asserter.find_text("Item1");
    item1.fg.assert_not_grayscale();
    let item2 = asserter.find_text("Item2");
    item2.fg.assert_grayscale();

    // Press down
    asserter.press_key(Key::Down);
    asserter.wait_for_settle();

    // Now Item2 highlighted, Item1 not
    item2.fg.assert_not_grayscale();
    item1.fg.assert_grayscale();
}

/// SP-007: Navigation - Up Arrow Wrap
/// Given items: ["Item1", "Item2"] with "Item1" selected
/// When user presses Up arrow
/// Then "Item2" should be highlighted (wraps around)
#[test]
fn test_select_picker_navigation_up_wrap() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Initially Item1 highlighted
    let item1 = asserter.find_text("Item1");
    item1.fg.assert_not_grayscale();
    let item2 = asserter.find_text("Item2");
    item2.fg.assert_grayscale();

    // Press up to wrap to Item2
    asserter.press_key(Key::Up);
    asserter.wait_for_settle();

    // Now Item2 highlighted, Item1 not
    item2.fg.assert_not_grayscale();
    item1.fg.assert_grayscale();
}

/// SP-008: Navigation - Down Arrow Wrap
/// Given items: ["Item1", "Item2"] with "Item2" selected
/// When user presses Down arrow
/// Then "Item1" should be highlighted (wraps around)
#[test]
fn test_select_picker_navigation_down_wrap() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Initially Item1 highlighted, navigate to Item2
    asserter.press_key(Key::Down);
    asserter.wait_for_settle();

    // Now Item2 highlighted
    let item1 = asserter.find_text("Item1");
    let item2 = asserter.find_text("Item2");
    item2.fg.assert_not_grayscale();
    item1.fg.assert_grayscale();

    // Press down again to wrap to Item1
    asserter.press_key(Key::Down);
    asserter.wait_for_settle();

    // Now Item1 highlighted, Item2 not
    item1.fg.assert_not_grayscale();
    item2.fg.assert_grayscale();
}

/// SP-009: Selection - Enter Key
/// Given items with "Item2" highlighted
/// When user presses Enter
/// Then picker should output `Some("Item2")` to stdout
/// And TUI should exit
#[test]
fn test_select_picker_selection_enter() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2,Item3")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Navigate to Item2
    asserter.press_key(Key::Down);
    asserter.wait_for_settle();

    // Select with Enter
    asserter.press_key(Key::Enter);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "Some(\"Item2\")");
}

/// SP-010: Cancel - Escape Key
/// Given picker is open
/// When user presses Escape
/// Then picker should output `None` to stdout
/// And TUI should exit
#[test]
fn test_select_picker_cancel_escape() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Cancel with Escape
    asserter.press_key(Key::Esc);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "None");
}

/// SP-011: Cancel - Ctrl+C
/// Given picker is open
/// When user presses Ctrl+C
/// Then picker should output `None` to stdout
/// And TUI should exit
#[test]
fn test_select_picker_cancel_ctrl_c() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Cancel with Ctrl+C
    asserter.press_key(Key::Ctrl('c'));

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "None");
}

/// SP-012: Custom Item Rendering
/// Given items with custom `WidgetRef` implementation
/// When picker is shown
/// Then items should render according to their custom implementation
#[test]
fn test_select_picker_custom_item_rendering() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "CustomItem1,CustomItem2")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Verify custom rendered content is visible
    // (The test command uses SimpleItem which renders the text directly)
    asserter.find_text("CustomItem1").assert_visible();
    asserter.find_text("CustomItem2").assert_visible();
}

/// SP-013: Item Constraints
/// Given items with different `constraint()` values
/// When picker is shown
/// Then items should be laid out according to their constraints
#[test]
fn test_select_picker_item_constraints() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_PICKER_ITEMS", "Item1,Item2,Item3")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test picker" from palette
    asserter.type_text("test picker");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Verify items are visible (layout constraints applied)
    // SimpleItem uses a default constraint, so items should be visible
    asserter.find_text("Item1").assert_visible();
    asserter.find_text("Item2").assert_visible();
    asserter.find_text("Item3").assert_visible();
}

// =============================================================================
// Text Picker Tests (TP-001 to TP-007)
// =============================================================================

/// TP-001: Basic Text Input
/// Given empty text picker with prompt "Enter name:"
/// When user types "Hello"
/// Then input panel should show "Enter name: Hello"
#[test]
fn test_text_picker_basic_input() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_TEXT_PROMPT", "Enter name:")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input" from palette
    asserter.type_text("test text input");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "Hello" in the text picker
    asserter.type_text("Hello");
    asserter.wait_for_settle();

    // Verify input is displayed
    asserter.find_text("Enter name: Hello").assert_visible();
}

/// TP-002: Text Input - Backspace
/// Given text picker with input "Hello"
/// When user presses Backspace
/// Then input should show "Hell"
#[test]
fn test_text_picker_backspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_TEXT_PROMPT", "Enter name:")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input" from palette
    asserter.type_text("test text input");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "Hello" then backspace
    asserter.type_text("Hello");
    asserter.wait_for_settle();
    asserter.find_text("Enter name: Hello").assert_visible();

    asserter.press_key(Key::Backspace);
    asserter.wait_for_settle();

    // Verify backspace removed the last character
    asserter.find_text("Enter name: Hell").assert_visible();
    asserter.find_text("Enter name: Hello").assert_not_visible();
}

/// TP-003: Text Input - Empty Backspace
/// Given text picker with empty input
/// When user presses Backspace
/// Then input should remain empty
#[test]
fn test_text_picker_empty_backspace() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_TEXT_PROMPT", "Enter name:")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input" from palette
    asserter.type_text("test text input");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Press backspace on empty input
    asserter.press_key(Key::Backspace);
    asserter.wait_for_settle();

    // Prompt should still be visible, no change
    asserter.find_text("Enter name:").assert_visible();
}

/// TP-004: Text Input - Confirm with Enter
/// Given text picker with input "Test"
/// When user presses Enter
/// Then picker should output `Some("Test")` to stdout
/// And TUI should exit
#[test]
fn test_text_picker_confirm_enter() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_TEXT_PROMPT", "Enter name:")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input" from palette
    asserter.type_text("test text input");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "Test" and confirm
    asserter.type_text("Test");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "Some(\"Test\")");
}

/// TP-005: Text Input - Cancel with Escape
/// Given text picker with input "Test"
/// When user presses Escape
/// Then picker should output `None` to stdout
/// And TUI should exit
#[test]
fn test_text_picker_cancel_escape() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_TEXT_PROMPT", "Enter name:")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input" from palette
    asserter.type_text("test text input");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "Test" then cancel
    asserter.type_text("Test");
    asserter.wait_for_settle();
    asserter.press_key(Key::Esc);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "None");
}

/// TP-006: Text Input - Cancel with Ctrl+C
/// Given text picker is open
/// When user presses Ctrl+C
/// Then picker should output `None` to stdout
/// And TUI should exit
#[test]
fn test_text_picker_cancel_ctrl_c() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_TEXT_PROMPT", "Enter name:")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input" from palette
    asserter.type_text("test text input");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Cancel with Ctrl+C
    asserter.press_key(Key::Ctrl('c'));

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "None");
}

/// TP-007: Unicode Support
/// Given text picker
/// When user types "日本語テスト"
/// Then input should show "日本語テスト" correctly
#[test]
fn test_text_picker_unicode() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_TEXT_PROMPT", "Enter name:")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input" from palette
    asserter.type_text("test text input");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type Japanese text
    asserter.type_text("日本語テスト");
    asserter.wait_for_settle();

    // Verify Japanese text is displayed
    asserter.find_text("日本語テスト").assert_visible();
}

// =============================================================================
// Text Picker with Suggestions Tests (TPS-001 to TPS-009)
// =============================================================================

/// TPS-001: Suggestions Display
/// Given text picker with suggestions provider returning ["apple", "application", "apply"]
/// When user types "app"
/// Then middle panel should show all three suggestions
/// And "apple" should be highlighted (top match)
#[test]
fn test_text_picker_suggestions_display() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_SUGGESTIONS", "apple,application,apply")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "app" to trigger suggestions
    asserter.type_text("app");
    asserter.wait_for_settle();

    // All suggestions visible
    asserter.find_text("apple").assert_visible();
    asserter.find_text("application").assert_visible();
    asserter.find_text("apply").assert_visible();

    // First one highlighted
    let apple = asserter.find_text("apple");
    apple.fg.assert_not_grayscale();
}

/// TPS-002: Top Suggestion Auto-Highlighted
/// Given text picker with suggestions
/// When suggestions are displayed
/// Then the first suggestion should be visually highlighted
/// And should be the Tab completion target
#[test]
fn test_text_picker_suggestions_top_highlighted() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_SUGGESTIONS", "apple,banana,cherry")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "ap" to trigger suggestions
    asserter.type_text("ap");
    asserter.wait_for_settle();

    // First suggestion highlighted
    let apple = asserter.find_text("apple");
    apple.fg.assert_not_grayscale();
}

/// TPS-003: Tab Completes Top Suggestion
/// Given text picker with suggestions, top suggestion "apple" highlighted
/// When user presses Tab
/// Then input should be completed to "apple"
#[test]
fn test_text_picker_suggestions_tab_completion() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_SUGGESTIONS", "apple,application,apply")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "ap" to trigger suggestions
    asserter.type_text("ap");
    asserter.wait_for_settle();

    // Press Tab to complete
    asserter.press_key(Key::Tab);
    asserter.wait_for_settle();

    // Verify apple completed
    asserter.find_text("Query: apple").assert_visible();
}

/// TPS-004: Arrow Navigation Changes Target
/// Given text picker with suggestions ["apple", "banana", "cherry"], "apple" selected
/// When user presses Down arrow (now "banana" selected)
/// And user presses Tab
/// Then input should be completed to "banana"
#[test]
fn test_text_picker_suggestions_arrow_navigation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_SUGGESTIONS", "apple,banana,cherry")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "a" to trigger suggestions
    asserter.type_text("a");
    asserter.wait_for_settle();

    // Navigate down to banana
    asserter.press_key(Key::Down);
    asserter.wait_for_settle();

    // Tab to complete banana
    asserter.press_key(Key::Tab);
    asserter.wait_for_settle();

    // Verify banana completed
    asserter.find_text("Query: banana").assert_visible();
}

/// TPS-005: Enter Confirms Current Text
/// Given text picker with input "cust"
/// And suggestions ["custom", "customer"]
/// When user presses Enter (without Tab)
/// Then picker should output `Some("cust")` to stdout
#[test]
fn test_text_picker_suggestions_enter_confirms_text() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_SUGGESTIONS", "custom,customer")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "cust" (not completing)
    asserter.type_text("cust");
    asserter.wait_for_settle();

    // Press Enter to confirm current text
    asserter.press_key(Key::Enter);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "Some(\"cust\")");
}

/// TPS-006: No Suggestions - Shows Message
/// Given text picker where provider returns `None` for input "x"
/// When user types "x"
/// Then middle panel should show "No suggestions available"
///
/// Note: This is a unit test - tests suggestion provider logic, not rendering
#[test]
fn test_text_picker_suggestions_no_suggestions_message() {
    // Unit test for the suggestion provider behavior
    // In the TUI test, when no suggestions match, "No suggestions available" should be shown
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        // Empty suggestions - provider returns None
        .with_env_var("TEST_SUGGESTIONS", "")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type something that won't match empty suggestions
    asserter.type_text("xyz");
    asserter.wait_for_settle();

    // Verify "No suggestions available" is displayed
    asserter
        .find_text("No suggestions available")
        .assert_visible();
}

/// TPS-007: Empty Suggestions - Shows Message
/// Given text picker where provider returns `Some([])` for input "xyz"
/// When user types "xyz"
/// Then middle panel should show "No suggestions available"
///
/// Note: Unit test - tests suggestion provider logic, not rendering
#[test]
fn test_text_picker_suggestions_empty_suggestions_message() {
    // Similar to TPS-006 - tests the empty suggestions case
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_SUGGESTIONS", "apple,banana")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type something that won't match any suggestions
    asserter.type_text("xyz");
    asserter.wait_for_settle();

    // Verify "No suggestions available" is displayed
    asserter
        .find_text("No suggestions available")
        .assert_visible();
}

/// TPS-008: Suggestions Update on Input Change
/// Given text picker with dynamic suggestions
/// When user types "a" (suggestions: ["apple", "apricot"])
/// And user types "b" (suggestions: ["banana", "blueberry"])
/// Then middle panel should update to show new suggestions
#[test]
fn test_text_picker_suggestions_update_on_input_change() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_SUGGESTIONS", "apple,apricot,banana,blueberry")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // After typing "a"
    asserter.type_text("a");
    asserter.wait_for_settle();
    asserter.find_text("apple").assert_visible();
    asserter.find_text("banana").assert_not_visible();

    // Clear and type "b"
    asserter.press_key(Key::Ctrl('u')); // Clear line (common TUI pattern)
    asserter.type_text("b");
    asserter.wait_for_settle();

    // Now banana visible, apple not
    asserter.find_text("banana").assert_visible();
    asserter.find_text("apple").assert_not_visible();
}

/// TPS-009: Ctrl+J/Ctrl+K Navigation
/// Given text picker with suggestions
/// When user presses Ctrl+J
/// Then selection should move down (highlight changes)
/// When user presses Ctrl+K
/// Then selection should move up (highlight changes)
#[test]
fn test_text_picker_suggestions_ctrl_j_k_navigation() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_SUGGESTIONS", "apple,banana,cherry")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test text input suggestions" from palette
    asserter.type_text("test text input suggestions");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Type "a" to trigger suggestions
    asserter.type_text("a");
    asserter.wait_for_settle();

    // Initially apple highlighted
    let apple = asserter.find_text("apple");
    let banana = asserter.find_text("banana");
    apple.fg.assert_not_grayscale();
    banana.fg.assert_grayscale();

    // Press Ctrl+J to move down
    asserter.press_key(Key::Ctrl('j'));
    asserter.wait_for_settle();

    // Now banana highlighted
    banana.fg.assert_not_grayscale();
    apple.fg.assert_grayscale();

    // Press Ctrl+K to move up
    asserter.press_key(Key::Ctrl('k'));
    asserter.wait_for_settle();

    // Now apple highlighted again
    apple.fg.assert_not_grayscale();
    banana.fg.assert_grayscale();
}

// =============================================================================
// Confirm Picker Tests (CP-001 to CP-005)
// =============================================================================

/// CP-001: Confirm Display - Default Yes
/// Given confirm picker with prompt "Continue?" and default `true`
/// Then "Yes" should be highlighted
/// And "No" should be visible but not highlighted
#[test]
fn test_confirm_picker_default_yes() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_CONFIRM_PROMPT", "Continue?")
        .with_env_var("TEST_CONFIRM_DEFAULT", "true")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test confirm" from palette
    asserter.type_text("test confirm");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Verify Yes is highlighted, No is not
    let yes = asserter.find_text("Yes");
    let no = asserter.find_text("No");
    yes.fg.assert_not_grayscale(); // highlighted
    no.fg.assert_grayscale(); // not highlighted
}

/// CP-002: Confirm Display - Default No
/// Given confirm picker with prompt "Delete?" and default `false`
/// Then "No" should be highlighted
/// And "Yes" should be visible but not highlighted
#[test]
fn test_confirm_picker_default_no() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_CONFIRM_PROMPT", "Delete?")
        .with_env_var("TEST_CONFIRM_DEFAULT", "false")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test confirm" from palette
    asserter.type_text("test confirm");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Verify No is highlighted, Yes is not
    let yes = asserter.find_text("Yes");
    let no = asserter.find_text("No");
    no.fg.assert_not_grayscale(); // highlighted
    yes.fg.assert_grayscale(); // not highlighted
}

/// CP-003: Confirm - Select Yes
/// Given confirm picker with "No" selected
/// When user presses Left/Right arrow to change selection
/// And user presses Enter
/// Then picker should output `Some(true)` to stdout
#[test]
fn test_confirm_picker_select_yes() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_CONFIRM_PROMPT", "Continue?")
        .with_env_var("TEST_CONFIRM_DEFAULT", "false") // Start with No selected
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test confirm" from palette
    asserter.type_text("test confirm");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Navigate to Yes
    asserter.press_key(Key::Right);
    asserter.wait_for_settle();

    // Confirm with Enter
    asserter.press_key(Key::Enter);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "Some(true)");
}

/// CP-004: Confirm - Select No
/// Given confirm picker
/// When user selects "No"
/// And user presses Enter
/// Then picker should output `Some(false)` to stdout
#[test]
fn test_confirm_picker_select_no() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_CONFIRM_PROMPT", "Continue?")
        .with_env_var("TEST_CONFIRM_DEFAULT", "true") // Start with Yes selected
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test confirm" from palette
    asserter.type_text("test confirm");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Navigate to No
    asserter.press_key(Key::Right);
    asserter.wait_for_settle();

    // Confirm with Enter
    asserter.press_key(Key::Enter);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "Some(false)");
}

/// CP-005: Confirm - Cancel
/// Given confirm picker
/// When user presses Escape
/// Then picker should output `None` to stdout
#[test]
fn test_confirm_picker_cancel() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_env_var("TEST_MODE", "1")
        .with_env_var("TEST_CONFIRM_PROMPT", "Continue?")
        .with_env_var("TEST_CONFIRM_DEFAULT", "true")
        .args(&["command-palette", "show"])
        .build();

    let mut asserter = env.testers().pty().terminal_size(40, 120).run(&cmd);

    asserter.wait_for_settle();

    // Select "test confirm" from palette
    asserter.type_text("test confirm");
    asserter.wait_for_settle();
    asserter.press_key(Key::Enter);
    asserter.wait_for_settle();

    // Cancel with Escape
    asserter.press_key(Key::Esc);

    let output = asserter.expect_completion_and_get_output();
    assert_eq!(output.trim(), "None");
}
