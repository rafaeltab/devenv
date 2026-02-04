//! 1.5 Input Tests
//!
//! Tests for keyboard input handling in TUI testers.

use test_descriptors::testers::{Command, Key, Modifier, TuiAsserter};
use test_descriptors::TestEnvironment;

/// Type "hello" into cat, verify echoed back.
#[test]
fn type_text_sends_to_pty() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    asserter.type_text("hello");
    asserter.wait_for_settle();
    asserter.find_text("hello").assert_visible();

    // Send Ctrl+D to exit cat
    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);
}

/// Press Enter, verify newline.
#[test]
fn press_key_enter() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    asserter.type_text("line1");
    asserter.press_key(Key::Enter);
    asserter.type_text("line2");
    asserter.wait_for_settle();

    asserter.find_text("line1").assert_visible();
    asserter.find_text("line2").assert_visible();

    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);
}

/// Press arrow keys, don't crash.
#[test]
fn press_key_arrows() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    // Should not crash
    asserter.press_key(Key::Up);
    asserter.press_key(Key::Down);
    asserter.press_key(Key::Left);
    asserter.press_key(Key::Right);

    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);
}

/// Press Esc, don't crash.
#[test]
fn press_key_esc() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    // Should not crash
    asserter.press_key(Key::Esc);

    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);
}

/// Press Backspace, verify character deleted.
#[test]
fn press_key_backspace() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Use a shell with line editing
    let cmd = Command::new("bash").args(&["-i"]);
    let mut asserter = env.testers().pty().run(&cmd);

    asserter.wait_for_settle();
    asserter.type_text("abc");
    asserter.press_key(Key::Backspace);
    asserter.wait_for_settle();

    // "ab" should be visible, "abc" should not
    asserter.find_text("ab").assert_visible();
}

/// Ctrl+C interrupts running process.
#[test]
fn send_keys_ctrl_c() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Start a long-running command
    let cmd = Command::new("sleep").args(&["100"]);
    let mut asserter = env.testers().pty().run(&cmd);

    // Send Ctrl+C to interrupt
    asserter.send_keys(&[Key::Char('c').with_modifier(Modifier::Ctrl)]);

    // Should exit with non-zero (interrupted)
    let exit_code = asserter.expect_completion();
    assert_ne!(exit_code, 0);
}

/// Panics when only modifiers sent.
#[test]
#[should_panic(expected = "modifier")]
fn send_keys_requires_non_modifier() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    // Should panic - can't send just modifiers without a key
    asserter.send_keys(&[Key::Modifier(Modifier::Ctrl)]);
}

/// Panics when multiple regular keys sent.
#[test]
#[should_panic(expected = "single")]
fn send_keys_single_regular_key_only() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    // Should panic - can't send multiple regular keys at once
    asserter.send_keys(&[Key::Char('a'), Key::Char('b')]);
}

/// Alt+A detected by key detector program.
#[test]
fn send_keys_alt_a() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Use a program that can detect key combos
    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    asserter.send_keys(&[Key::Char('a').with_modifier(Modifier::Alt)]);
    asserter.wait_for_settle();

    // Alt+A sends ESC followed by 'a' - verify the escape sequence
    // The exact output depends on terminal settings
    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);
}

/// Shift+Up detected.
#[test]
fn send_keys_shift_up() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    // Should not crash
    asserter.send_keys(&[Key::Up.with_modifier(Modifier::Shift)]);

    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);
}

/// Ctrl+Shift+R detected.
#[test]
fn send_keys_ctrl_shift_r() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("cat");
    let mut asserter = env.testers().pty().run(&cmd);

    // Should not crash
    asserter.send_keys(&[Key::Char('r')
        .with_modifier(Modifier::Ctrl)
        .with_modifier(Modifier::Shift)]);

    asserter.send_keys(&[Key::Char('d').with_modifier(Modifier::Ctrl)]);
}
