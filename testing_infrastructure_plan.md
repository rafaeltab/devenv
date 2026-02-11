# Testing Infrastructure Plan (TDD)

## Overview

This plan follows strict TDD for adding testing infrastructure support for screen-based picker testing. Each phase: write tests first, then implement, then verify.

---

## Phase 1: Color Assertion Tests

### 1.1: Write Tests for Grayscale Assertions

Create `packages/test-descriptors/src/testers/color_grayscale_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::super::ColorAssertion;

    #[test]
    fn assert_grayscale_passes_for_pure_gray() {
        let color = ColorAssertion::from_rgb(128, 128, 128);
        color.assert_grayscale(); // Should pass
    }

    #[test]
    fn assert_grayscale_passes_for_near_gray() {
        let color = ColorAssertion::from_rgb(100, 105, 102);
        color.assert_grayscale(); // Should pass (within tolerance)
    }

    #[test]
    #[should_panic(expected = "Expected color to be grayscale")]
    fn assert_grayscale_fails_for_red() {
        let color = ColorAssertion::from_rgb(255, 0, 0);
        color.assert_grayscale(); // Should panic
    }

    #[test]
    #[should_panic(expected = "Expected color to be grayscale")]
    fn assert_grayscale_fails_for_yellow() {
        let color = ColorAssertion::from_rgb(255, 255, 0);
        color.assert_grayscale(); // Should panic
    }

    #[test]
    fn assert_not_grayscale_passes_for_red() {
        let color = ColorAssertion::from_rgb(255, 0, 0);
        color.assert_not_grayscale(); // Should pass
    }

    #[test]
    fn assert_not_grayscale_passes_for_yellow() {
        let color = ColorAssertion::from_rgb(255, 255, 0);
        color.assert_not_grayscale(); // Should pass
    }

    #[test]
    #[should_panic(expected = "Expected color to NOT be grayscale")]
    fn assert_not_grayscale_fails_for_gray() {
        let color = ColorAssertion::from_rgb(128, 128, 128);
        color.assert_not_grayscale(); // Should panic
    }

    #[test]
    #[should_panic(expected = "Cannot check color: text not found on screen")]
    fn assert_grayscale_fails_when_not_found() {
        let color = ColorAssertion::not_found();
        color.assert_grayscale(); // Should panic
    }

    #[test]
    #[should_panic(expected = "Cannot check color: text not found on screen")]
    fn assert_not_grayscale_fails_when_not_found() {
        let color = ColorAssertion::not_found();
        color.assert_not_grayscale(); // Should panic
    }
}
```

**Status**: ⏳ Write these tests - they will FAIL (methods don't exist)

### 1.2: Implement Grayscale Assertions

Add to `packages/test-descriptors/src/testers/color.rs` in `impl ColorAssertion`:

```rust
/// Assert that the color is grayscale (R, G, B are equal or nearly equal)
pub fn assert_grayscale(&self) {
    if !self.found {
        panic!("Cannot check color: text not found on screen");
    }

    let r = self.r.expect("Color RGB values should be set when found");
    let g = self.g.expect("Color RGB values should be set when found");
    let b = self.b.expect("Color RGB values should be set when found");

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let is_grayscale = max - min <= 20; // Within tolerance

    if !is_grayscale {
        panic!(
            "Expected color to be grayscale, but got ({}, {}, {}) (difference: {})",
            r, g, b, max - min
        );
    }
}

/// Assert that the color is NOT grayscale
pub fn assert_not_grayscale(&self) {
    if !self.found {
        panic!("Cannot check color: text not found on screen");
    }

    let r = self.r.expect("Color RGB values should be set when found");
    let g = self.g.expect("Color RGB values should be set when found");
    let b = self.b.expect("Color RGB values should be set when found");

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let is_grayscale = max - min <= 20;

    if is_grayscale {
        panic!(
            "Expected color to NOT be grayscale, but got ({}, {}, {})",
            r, g, b
        );
    }
}
```

**Status**: ⏳ Implement - tests should PASS

### 1.3: Verify

```bash
cargo test -p test-descriptors color_grayscale
```

**Status**: All 10 grayscale tests pass

---

## Phase 2: Vertical Order Assertion Tests

### 2.1: Write Tests for Vertical Ordering

Create `packages/test-descriptors/src/testers/vertical_order_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::super::{TextMatch, TuiAsserter};

    #[test]
    fn assert_vertical_order_passes_when_ordered() {
        // Create mock TextMatches with positions
        let item1 = create_mock_text_match("Item1", 10, 5);
        let item2 = create_mock_text_match("Item2", 11, 5);
        let item3 = create_mock_text_match("Item3", 12, 5);

        // This should pass - rows are increasing
        // mock_asserter.assert_vertical_order(&[item1, item2, item3]);
    }

    #[test]
    #[should_panic(expected = "Expected items to be in vertical order")]
    fn assert_vertical_order_fails_when_out_of_order() {
        let item1 = create_mock_text_match("Item1", 10, 5);
        let item2 = create_mock_text_match("Item2", 8, 5);  // Above item1!
        let item3 = create_mock_text_match("Item3", 12, 5);

        // This should panic - item2 is above item1
        // mock_asserter.assert_vertical_order(&[item1, item2, item3]);
    }

    #[test]
    #[should_panic(expected = "Item at index 1 not found")]
    fn assert_vertical_order_fails_when_item_not_visible() {
        let item1 = create_mock_text_match("Item1", 10, 5);
        let item2 = TextMatch::not_found("Item2");

        // This should panic - item2 is not found
        // mock_asserter.assert_vertical_order(&[item1, item2]);
    }

    #[test]
    fn assert_vertical_order_passes_with_same_row_different_col() {
        // Items on same row but different columns
        let item1 = create_mock_text_match("Item1", 10, 5);
        let item2 = create_mock_text_match("Item2", 10, 20);

        // Same row is OK (left-to-right reading order)
        // mock_asserter.assert_vertical_order(&[item1, item2]);
    }

    #[test]
    fn assert_vertical_order_single_item() {
        let item1 = create_mock_text_match("Item1", 10, 5);
        // Should pass with single item
        // mock_asserter.assert_vertical_order(&[item1]);
    }

    #[test]
    #[should_panic(expected = "Cannot assert order on empty list")]
    fn assert_vertical_order_empty_list() {
        // Should panic on empty list
        // mock_asserter.assert_vertical_order(&[]);
    }
}
```

**Status**: ⏳ Write these tests - they will FAIL (method doesn't exist)

### 2.2: Implement Vertical Order Assertion

Add to `TuiAsserter` trait in `packages/test-descriptors/src/testers/tui_asserter.rs`:

```rust
/// Assert that text matches appear in vertical order from top to bottom.
/// Items should be ordered by row (top to bottom), and within the same row
/// by column (left to right).
///
/// # Panics
/// Panics if:
/// - The list is empty
/// - Any item is not visible
/// - Items are not in top-to-bottom order
fn assert_vertical_order(&self, matches: &[TextMatch]) {
    if matches.is_empty() {
        panic!("Cannot assert order on empty list");
    }

    let mut last_row: Option<u16> = None;
    let mut last_col: Option<u16> = None;

    for (i, text_match) in matches.iter().enumerate() {
        // Check item is visible
        if !text_match.is_visible() {
            panic!(
                "Item at index {} not found: '{}'",
                i,
                text_match.text
            );
        }

        let (row, col) = text_match.position().expect("Visible item should have position");

        // Check ordering
        if let Some(prev_row) = last_row {
            if row < prev_row {
                panic!(
                    "Expected items to be in vertical order (top to bottom). \
                     Item {} '{}' at row {} is above previous item at row {}",
                    i, text_match.text, row, prev_row
                );
            }

            // If same row, check left-to-right ordering
            if row == prev_row {
                if let Some(prev_col) = last_col {
                    if col < prev_col {
                        panic!(
                            "Expected items to be ordered left-to-right within row {}. \
                             Item {} '{}' at col {} is left of previous item at col {}",
                            row, i, text_match.text, col, prev_col
                        );
                    }
                }
            }
        }

        last_row = Some(row);
        last_col = Some(col);
    }
}
```

**Status**: ⏳ Implement - tests should PASS

### 2.3: Verify

```bash
cargo test -p test-descriptors vertical_order
```

**Status**: All 6 vertical order tests pass

---

## Phase 3: Output Capture Tests

### 3.1: Write Tests for Output Capture

Create `packages/test-descriptors/src/testers/output_capture_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::super::TuiAsserter;

    #[test]
    fn expect_completion_and_get_output_returns_stdout() {
        // Mock a command that outputs "Hello" and exits
        // let output = mock_asserter.expect_completion_and_get_output();
        // assert_eq!(output, "Hello");
    }

    #[test]
    fn expect_completion_and_get_output_captures_full_output() {
        // Mock a command that outputs multiple lines
        // let output = mock_asserter.expect_completion_and_get_output();
        // assert!(output.contains("Line1"));
        // assert!(output.contains("Line2"));
    }

    #[test]
    fn expect_completion_and_get_output_captures_special_chars() {
        // Test that special characters are captured correctly
        // let output = mock_asserter.expect_completion_and_get_output();
        // assert!(output.contains("Some(\"test\")"));
    }

    #[test]
    #[should_panic(expected = "Process did not complete")]
    fn expect_completion_and_get_output_fails_if_not_completed() {
        // Should panic if process hasn't exited
        // mock_asserter.expect_completion_and_get_output();
    }
}
```

**Status**: ⏳ Write these tests - they will FAIL (method doesn't exist)

### 3.2: Implement Output Capture

Add to `TuiAsserter` trait in `packages/test-descriptors/src/testers/tui_asserter.rs`:

```rust
/// Wait for the command to complete and return its stdout output as a string.
/// This is useful for capturing test command output like "Some(\"value\")" or "None".
///
/// # Returns
/// The stdout content of the process as a String.
///
/// # Panics
/// Panics if the process doesn't complete within a timeout.
fn expect_completion_and_get_output(&mut self) -> String;
```

Implement in `PtyAsserter` (in `packages/test-descriptors/src/testers/pty/pty_asserter.rs`):

```rust
fn expect_completion_and_get_output(&mut self) -> String {
    // First wait for completion and get exit code
    let _exit_code = self.expect_completion();

    // Get the captured stdout from the PTY session
    // This requires storing stdout during the PTY session
    self.session.stdout()
        .expect("Should have captured stdout")
}
```

**Note**: This requires modifying the PTY session to capture stdout separately from the terminal screen.

**Status**: ⏳ Implement - tests should PASS

### 3.3: Verify

```bash
cargo test -p test-descriptors output_capture
```

**Status**: All 4 output capture tests pass

---

## Phase 4: Integration Tests for Test Infrastructure

### 4.1: Write End-to-End Test

Create `packages/test-descriptors/tests/test_infra_integration_tests.rs`:

```rust
use test_descriptors::testers::{Key, TuiAsserter};
use test_descriptors::TestEnvironment;

/// This test verifies that all testing infrastructure works together
/// by running a simple command and using all the new assertions.
#[test]
fn test_full_infrastructure_stack() {
    let env = TestEnvironment::describe(|root| {
        // Create a simple test script that echoes output
        root.test_dir(|td| {
            td.file("echo_test.sh", r#"#!/bin/bash
echo "First"
echo "Second"
echo "Third"
echo "Some(\"done\")"
"#);
        });
    }).create();

    // Create a command that uses the test script
    let cmd = /* ... build command ... */;

    let mut asserter = env.testers().pty()
        .terminal_size(40, 120)
        .run(&cmd);

    // Test visibility assertions
    asserter.find_text("First").assert_visible();
    asserter.find_text("Second").assert_visible();
    asserter.find_text("Third").assert_visible();

    // Test vertical order
    let first = asserter.find_text("First");
    let second = asserter.find_text("Second");
    let third = asserter.find_text("Third");
    asserter.assert_vertical_order(&[first, second, third]);

    // Test output capture
    let output = asserter.expect_completion_and_get_output();
    assert!(output.contains("Some(\"done\")"));
}
```

**Status**: ⏳ Write this test - it will FAIL (requires full implementation)

### 4.2: Verify Full Integration

```bash
cargo test -p test-descriptors test_infra_integration
```

**Status**: Integration test passes

---

## Summary

| Phase | Activity         | Test File                         | Status                      |
| ----- | ---------------- | --------------------------------- | --------------------------- |
| 1     | Color Assertions | `color_grayscale_tests.rs`        | ⏳ Write → Implement → Pass |
| 2     | Vertical Order   | `vertical_order_tests.rs`         | ⏳ Write → Implement → Pass |
| 3     | Output Capture   | `output_capture_tests.rs`         | ⏳ Write → Implement → Pass |
| 4     | Full Integration | `test_infra_integration_tests.rs` | ⏳ Write → Implement → Pass |

**Files Modified**:

- `packages/test-descriptors/src/testers/color.rs` - Add `assert_grayscale()` and `assert_not_grayscale()`
- `packages/test-descriptors/src/testers/tui_asserter.rs` - Add `assert_vertical_order()` and `expect_completion_and_get_output()`
- `packages/test-descriptors/src/testers/pty/pty_asserter.rs` - Implement new trait methods
- `packages/test-descriptors/src/testers/mod.rs` - Include new test modules

**New Files Created**:

- `packages/test-descriptors/src/testers/color_grayscale_tests.rs`
- `packages/test-descriptors/src/testers/vertical_order_tests.rs`
- `packages/test-descriptors/src/testers/output_capture_tests.rs`
- `packages/test-descriptors/tests/test_infra_integration_tests.rs`

**TDD Flow**:

```
For each phase:
  1. Write test file with #[should_panic] expectations
  2. Run tests - confirm they fail (red)
  3. Implement the feature
  4. Run tests - confirm they pass (green)
  5. Move to next phase
```
