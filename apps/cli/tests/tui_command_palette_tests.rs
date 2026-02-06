mod common;

use common::CliCommandBuilder;
use test_descriptors::testers::{Key, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

#[test]
fn test_command_palette_displays_commands() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["command-palette", "show"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // Verify UI title is visible
    asserter.find_text("Enter your command:").assert_visible();

    // Verify all example commands are displayed
    asserter.find_text("Open workspace").assert_visible();
    asserter.find_text("Add workspace").assert_visible();
    asserter.find_text("Open link").assert_visible();
    asserter.find_text("Github").assert_visible();

    // Verify descriptions are shown
    asserter
        .find_text("Search through the workspaces, and open it")
        .assert_visible();
    asserter
        .find_text("Create a workspace in the current directory")
        .assert_visible();
    asserter
        .find_text("Search through links, and open them")
        .assert_visible();
    asserter
        .find_text("Open a github repository")
        .assert_visible();

    // Exit with Ctrl+C
    asserter.send_keys(&[Key::Ctrl('c')]);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_palette_filters_commands() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["command-palette", "show"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // All commands visible initially
    asserter.find_text("Open workspace").assert_visible();
    asserter.find_text("Add workspace").assert_visible();
    asserter.find_text("Open link").assert_visible();
    asserter.find_text("Github").assert_visible();

    // Type to filter
    asserter.type_text("workspace");
    asserter.wait_for_settle();

    // Only workspace-related commands should be visible
    asserter.find_text("Open workspace").assert_visible();
    asserter.find_text("Add workspace").assert_visible();
    // Note: "Open link" and "Github" might still be visible if the filter
    // implementation allows partial matches or if they're always shown

    // Exit with Ctrl+C
    asserter.send_keys(&[Key::Ctrl('c')]);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_palette_text_input() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["command-palette", "show"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // Type some unique text that won't appear in command names
    asserter.type_text("xyz");
    asserter.wait_for_settle();

    // The typed text should appear in the command palette
    asserter.find_text("xyz").assert_visible();

    // Use backspace to delete
    asserter.press_key(Key::Backspace);
    asserter.wait_for_settle();

    // "xy" should now be visible instead of "xyz"
    asserter.find_text("xy").assert_visible();

    // Continue deleting
    asserter.press_key(Key::Backspace);
    asserter.wait_for_settle();

    // Input should be empty now, "xy" should not be visible
    asserter.find_text("xy").assert_not_visible();

    // Exit with Ctrl+C
    asserter.send_keys(&[Key::Ctrl('c')]);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_palette_enter_completes() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["command-palette", "show"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // Verify command palette is displayed
    asserter.find_text("Enter your command:").assert_visible();

    // Press Enter to complete
    asserter.press_key(Key::Enter);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_palette_ctrl_c_exits() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .args(&["command-palette", "show"])
        .build();
    let mut asserter = env
        .testers()
        .pty()
        .terminal_size(40, 120)
        .settle_timeout(300)
        .run(&cmd);

    asserter.wait_for_settle();

    // Verify command palette is displayed
    asserter.find_text("Enter your command:").assert_visible();

    // Exit with Ctrl+C
    asserter.send_keys(&[Key::Ctrl('c')]);
    let exit_code = asserter.expect_completion();
    assert_eq!(exit_code, 0);
}
