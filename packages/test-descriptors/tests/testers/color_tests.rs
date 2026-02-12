//! 1.7 Color Tests
//!
//! Tests for color detection and matching in terminal output.

use test_descriptors::testers::{ColorMatcher, Command, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// Grayscale detection works.
#[test]
fn color_matcher_grayscale() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Output gray text
    let cmd = Command::new("printf").args(&["\x1b[90mGray text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Gray text");
    text_match.fg.assert_matches(ColorMatcher::Grayscale);
}

/// Yellow hue detection works.
#[test]
fn color_matcher_yellowish() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[33mYellow text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Yellow text");
    text_match.fg.assert_matches(ColorMatcher::YellowIsh);
}

/// Red hue detection works.
#[test]
fn color_matcher_redish() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[31mRed text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Red text");
    text_match.fg.assert_matches(ColorMatcher::RedIsh);
}

/// Green hue detection works.
#[test]
fn color_matcher_greenish() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[32mGreen text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Green text");
    text_match.fg.assert_matches(ColorMatcher::GreenIsh);
}

/// Blue hue detection works.
#[test]
fn color_matcher_blueish() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[34mBlue text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Blue text");
    text_match.fg.assert_matches(ColorMatcher::BlueIsh);
}

/// Cyan hue detection works.
#[test]
fn color_matcher_cyanish() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[36mCyan text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Cyan text");
    text_match.fg.assert_matches(ColorMatcher::CyanIsh);
}

/// Magenta hue detection works.
#[test]
fn color_matcher_magentaish() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[35mMagenta text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Magenta text");
    text_match.fg.assert_matches(ColorMatcher::MagentaIsh);
}

/// Custom hue range works.
#[test]
fn color_matcher_custom_hue_range() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Orange is around hue 30
    let cmd = Command::new("printf").args(&["\x1b[38;2;255;128;0mOrange text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Orange text");
    text_match.fg.assert_matches(ColorMatcher::Hue {
        min: 20.0,
        max: 40.0,
    });
}

/// Exact foreground RGB matching.
#[test]
fn color_assertion_fg_exact() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[38;2;255;0;0mExact red\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Exact red");
    text_match.fg.assert_rgb(255, 0, 0);
}

/// Background color matcher.
#[test]
fn color_assertion_bg_matcher() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[41mRed bg text\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Red bg text");
    text_match.bg.assert_matches(ColorMatcher::RedIsh);
}
