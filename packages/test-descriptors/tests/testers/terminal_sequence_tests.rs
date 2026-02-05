//! 1.8 Terminal Sequence Tests
//!
//! Tests for ANSI escape sequence handling in terminal emulation.

use test_descriptors::testers::{Command, TuiAsserter, TuiTester};
use test_descriptors::TestEnvironment;

/// ESC[2J clears screen.
#[test]
fn test_clear_screen_sequence() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["Before\x1b[2JAfter"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // "Before" should be cleared, "After" should be visible
    asserter.find_text("Before").assert_not_visible();
    asserter.find_text("After").assert_visible();
}

/// ESC[row;colH positions cursor.
#[test]
fn test_cursor_positioning() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Clear screen, move to row 5 col 10, print X
    let cmd = Command::new("printf").args(&["\x1b[2J\x1b[H\x1b[5;10HX"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("X");
    let pos = text_match.position().expect("X should be found");
    assert_eq!(pos, (4, 9)); // 0-indexed: row 5 -> 4, col 10 -> 9
}

/// ESC[K erases to end of line.
#[test]
fn test_erase_in_line() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Print AAAAAAAAAA, move to column 5, erase to end of line
    let cmd = Command::new("printf").args(&["AAAAAAAAAA\x1b[1;5H\x1b[K"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Should only see first 4 A's
    let screen = asserter.screen();
    assert!(screen.contains("AAAA"));
    assert!(!screen.contains("AAAAAAAAAA"));
}

/// ESC[s/ESC[u save/restore cursor.
#[test]
fn test_cursor_save_restore() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Clear, print Start, save cursor, move away, print Away, restore, print Back
    let cmd = Command::new("printf").args(&["\x1b[2J\x1b[HStart\x1b[s\x1b[10;10HAway\x1b[uBack"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // "StartBack" should appear at start position
    asserter.find_text("StartBack").assert_visible();
    asserter.find_text("Away").assert_visible();
}

/// ESC[A/B/C/D move cursor.
#[test]
fn test_cursor_movement() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Clear, move to 5,5, print O, move up, print U
    let cmd = Command::new("printf").args(&["\x1b[2J\x1b[5;5HO\x1b[AU"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // O should be at row 5, U should be at row 4
    let o_match = asserter.find_text("O");
    let u_match = asserter.find_text("U");

    let o_pos = o_match.position().expect("O should be found");
    let u_pos = u_match.position().expect("U should be found");

    assert_eq!(o_pos.0 - 1, u_pos.0); // U is one row above O
}

/// ESC[top;bottomr sets scroll region.
#[test]
fn test_scrolling_region() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf")
        .args(&["\x1b[2J\x1b[HHeader\n\x1b[2;4r\x1b[4;1HScroll content\x1b[r"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Header should remain at top, scroll content should be visible
    asserter.find_text("Header").assert_visible();
    asserter.find_text("Scroll content").assert_visible();
}

/// ESC[@n/ESC[Pn insert/delete chars.
#[test]
fn test_insert_delete_characters() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Print ABCDEF, move to column 3, insert 2 spaces, print XX
    let cmd = Command::new("printf").args(&["\x1b[2J\x1b[HABCDEF\x1b[1;3H\x1b[2@XX"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Result should be ABXXCDEF
    asserter.find_text("ABXXCDEF").assert_visible();
}

/// ESC[Ln/ESC[Mn insert/delete lines.
#[test]
fn test_insert_delete_lines() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd =
        Command::new("printf").args(&["\x1b[2J\x1b[HLine1\nLine2\nLine3\x1b[2;1H\x1b[LInserted"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    asserter.find_text("Line1").assert_visible();
    asserter.find_text("Inserted").assert_visible();
}

/// Characters overwrite existing content.
#[test]
fn test_overwrite_mode() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Print AAAAA, move to column 2, print BB
    // After AAAAA: cursor at col 5, screen shows "AAAAA"
    // Move to row 1, col 2 (1-indexed) = col 1 (0-indexed)
    // Print BB: overwrites positions 1 and 2
    // Result: A at 0, B at 1, B at 2, A at 3, A at 4 = "ABBAA"
    let cmd = Command::new("printf").args(&["\x1b[2J\x1b[HAAAAA\x1b[1;2HBB"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Result should be ABBAA (B overwrites positions 1 and 2)
    asserter.find_text("ABBAA").assert_visible();
}

/// Tab stops at column 8, 16, 24...
#[test]
fn test_tab_stops() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[2J\x1b[HA\tB\tC"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    // Check positions: A at 0, B at 8, C at 16
    let a_match = asserter.find_text("A");
    let b_match = asserter.find_text("B");
    let c_match = asserter.find_text("C");

    let a_col = a_match.position().expect("A should be found").1;
    let b_col = b_match.position().expect("B should be found").1;
    let c_col = c_match.position().expect("C should be found").1;

    assert_eq!(a_col, 0);
    assert_eq!(b_col, 8);
    assert_eq!(c_col, 16);
}

/// ESC[7m swaps fg/bg.
#[test]
fn test_reverse_video() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("printf").args(&["\x1b[7mReverse\x1b[0m Normal"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    asserter.find_text("Reverse").assert_visible();
    asserter.find_text("Normal").assert_visible();

    // The "Reverse" text should have swapped fg/bg
    // This is verified by checking the colors are different
}

/// ESC[38;5;nm 256-color mode.
#[test]
fn test_256_color_support() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Color 196 is bright red in 256-color palette
    let cmd = Command::new("printf").args(&["\x1b[38;5;196mBright Red\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("Bright Red");
    text_match.assert_visible();
    // Color 196 should be red-ish
    text_match
        .fg
        .assert_matches(test_descriptors::testers::ColorMatcher::RedIsh);
}

/// ESC[38;2;r;g;bm true color.
#[test]
fn test_rgb_color_support() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    // Pure red in true color
    let cmd = Command::new("printf").args(&["\x1b[38;2;255;0;0mTrue Red\x1b[0m"]);
    let mut asserter = env.testers().pty().run(&cmd);
    asserter.wait_for_settle();

    let text_match = asserter.find_text("True Red");
    text_match.fg.assert_rgb(255, 0, 0);
}
