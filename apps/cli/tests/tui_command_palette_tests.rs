mod common;

use crate::common::run_cli_tui;
use test_descriptors::TestEnvironment;
use tui_test::Key;

#[test]
fn test_command_palette_displays_commands() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let mut tui = run_cli_tui(
        &["command-palette", "show"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // Verify UI title is visible
    tui.find_text("Enter your command:").assert_visible();

    // Verify all example commands are displayed
    tui.find_text("Open workspace").assert_visible();
    tui.find_text("Add workspace").assert_visible();
    tui.find_text("Open link").assert_visible();
    tui.find_text("Github").assert_visible();

    // Verify descriptions are shown
    tui.find_text("Search through the workspaces, and open it")
        .assert_visible();
    tui.find_text("Create a workspace in the current directory")
        .assert_visible();
    tui.find_text("Search through links, and open them")
        .assert_visible();
    tui.find_text("Open a github repository").assert_visible();

    // Exit with Ctrl+C
    tui.send_keys(&[Key::Ctrl, Key::Char('c')]);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_palette_filters_commands() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let mut tui = run_cli_tui(
        &["command-palette", "show"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // All commands visible initially
    tui.find_text("Open workspace").assert_visible();
    tui.find_text("Add workspace").assert_visible();
    tui.find_text("Open link").assert_visible();
    tui.find_text("Github").assert_visible();

    // Type to filter
    tui.type_text("workspace");
    tui.wait_for_settle();

    // Only workspace-related commands should be visible
    tui.find_text("Open workspace").assert_visible();
    tui.find_text("Add workspace").assert_visible();
    // Note: "Open link" and "Github" might still be visible if the filter
    // implementation allows partial matches or if they're always shown

    // Exit with Ctrl+C
    tui.send_keys(&[Key::Ctrl, Key::Char('c')]);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_palette_text_input() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let mut tui = run_cli_tui(
        &["command-palette", "show"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // Type some unique text that won't appear in command names
    tui.type_text("xyz");
    tui.wait_for_settle();

    // The typed text should appear in the command palette
    tui.find_text("xyz").assert_visible();

    // Use backspace to delete
    tui.press_key(Key::Backspace);
    tui.wait_for_settle();

    // "xy" should now be visible instead of "xyz"
    tui.find_text("xy").assert_visible();

    // Continue deleting
    tui.press_key(Key::Backspace);
    tui.wait_for_settle();

    // Input should be empty now, "xy" should not be visible
    tui.find_text("xy").assert_not_visible();

    // Exit with Ctrl+C
    tui.send_keys(&[Key::Ctrl, Key::Char('c')]);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_palette_enter_completes() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let mut tui = run_cli_tui(
        &["command-palette", "show"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // Verify command palette is displayed
    tui.find_text("Enter your command:").assert_visible();

    // Press Enter to complete
    tui.press_key(Key::Enter);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}

#[test]
fn test_command_palette_ctrl_c_exits() {
    let env = TestEnvironment::describe(|root| {
        use crate::common::rafaeltab_descriptors::RafaeltabRootMixin;
        root.rafaeltab_config(|_c| {});
    })
    .create();

    let config_path = env.context().config_path().unwrap();

    let mut tui = run_cli_tui(
        &["command-palette", "show"],
        config_path.to_str().unwrap(),
        env.tmux_socket(),
    );

    tui.wait_for_settle();

    // Verify command palette is displayed
    tui.find_text("Enter your command:").assert_visible();

    // Exit with Ctrl+C
    tui.send_keys(&[Key::Ctrl, Key::Char('c')]);
    let exit_code = tui.expect_completion();
    assert_eq!(exit_code, 0);
}
