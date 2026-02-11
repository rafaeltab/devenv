/// Color assertion helper for verifying text colors in TUI output.
#[derive(Debug, Clone)]
pub struct ColorAssertion {
    r: Option<u8>,
    g: Option<u8>,
    b: Option<u8>,
    found: bool,
}

impl ColorAssertion {
    pub(crate) fn new(r: Option<u8>, g: Option<u8>, b: Option<u8>, found: bool) -> Self {
        Self { r, g, b, found }
    }

    pub(crate) fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: Some(r),
            g: Some(g),
            b: Some(b),
            found: true,
        }
    }

    pub(crate) fn not_found() -> Self {
        Self {
            r: None,
            g: None,
            b: None,
            found: false,
        }
    }

    /// Assert that the color matches the given matcher.
    pub fn assert(&self, matcher: ColorMatcher) {
        if !self.found {
            panic!("Cannot check color: text not found on screen");
        }

        let r = self.r.expect("Color RGB values should be set when found");
        let g = self.g.expect("Color RGB values should be set when found");
        let b = self.b.expect("Color RGB values should be set when found");

        if !matcher.matches(r, g, b) {
            panic!("Color ({}, {}, {}) does not match {:?}", r, g, b, matcher);
        }
    }

    /// Assert that the color matches the given matcher (alias for assert).
    pub fn assert_matches(&self, matcher: ColorMatcher) {
        self.assert(matcher);
    }

    /// Assert an exact RGB color match.
    pub fn exact(&self, expected_r: u8, expected_g: u8, expected_b: u8) {
        if !self.found {
            panic!("Cannot check color: text not found on screen");
        }

        let r = self.r.expect("Color RGB values should be set when found");
        let g = self.g.expect("Color RGB values should be set when found");
        let b = self.b.expect("Color RGB values should be set when found");

        if r != expected_r || g != expected_g || b != expected_b {
            panic!(
                "Expected color ({}, {}, {}), got ({}, {}, {})",
                expected_r, expected_g, expected_b, r, g, b
            );
        }
    }

    /// Assert an exact RGB color match (alias for exact).
    pub fn assert_rgb(&self, r: u8, g: u8, b: u8) {
        self.exact(r, g, b);
    }

    /// Get the RGB values if available.
    pub fn rgb(&self) -> Option<(u8, u8, u8)> {
        match (self.r, self.g, self.b) {
            (Some(r), Some(g), Some(b)) => Some((r, g, b)),
            _ => None,
        }
    }

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
                r,
                g,
                b,
                max - min
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
}

/// Color matcher for fuzzy color matching in TUI output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMatcher {
    /// Matches grayscale colors (equal or near-equal R, G, B)
    Grayscale,
    /// Matches colors with dominant red component
    RedIsh,
    /// Matches colors with dominant green component
    GreenIsh,
    /// Matches colors with dominant blue component
    BlueIsh,
    /// Matches colors with dominant red and green (yellow)
    YellowIsh,
    /// Matches colors with dominant green and blue (cyan)
    CyanIsh,
    /// Matches colors with dominant red and blue (magenta)
    MagentaIsh,
    /// Matches colors within a hue range (0-360 degrees)
    Hue { min: f32, max: f32 },
}

impl ColorMatcher {
    /// Check if the given RGB color matches this matcher.
    pub fn matches(&self, r: u8, g: u8, b: u8) -> bool {
        match self {
            ColorMatcher::Grayscale => {
                // Allow some tolerance for grayscale
                let max = r.max(g).max(b);
                let min = r.min(g).min(b);
                max - min <= 20 // Within 20 units tolerance
            }
            ColorMatcher::RedIsh => r > g.saturating_add(30) && r > b.saturating_add(30),
            ColorMatcher::GreenIsh => g > r.saturating_add(30) && g > b.saturating_add(30),
            ColorMatcher::BlueIsh => b > r.saturating_add(30) && b > g.saturating_add(30),
            ColorMatcher::YellowIsh => {
                r > b.saturating_add(30)
                    && g > b.saturating_add(30)
                    && (r as i16 - g as i16).abs() < 60
            }
            ColorMatcher::CyanIsh => {
                g > r.saturating_add(30)
                    && b > r.saturating_add(30)
                    && (g as i16 - b as i16).abs() < 60
            }
            ColorMatcher::MagentaIsh => {
                r > g.saturating_add(30)
                    && b > g.saturating_add(30)
                    && (r as i16 - b as i16).abs() < 60
            }
            ColorMatcher::Hue { min, max } => {
                let hue = Self::rgb_to_hue(r, g, b);
                if let Some(h) = hue {
                    if min <= max {
                        h >= *min && h <= *max
                    } else {
                        // Wrapping case (e.g., 350-10 for red at 0)
                        h >= *min || h <= *max
                    }
                } else {
                    false // Grayscale has no hue
                }
            }
        }
    }

    /// Convert RGB to HSL hue (0-360 degrees), returns None for grayscale.
    fn rgb_to_hue(r: u8, g: u8, b: u8) -> Option<f32> {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        if delta < 0.01 {
            return None; // Grayscale
        }

        let hue = if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let hue = if hue < 0.0 { hue + 360.0 } else { hue };
        Some(hue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grayscale_matcher() {
        assert!(ColorMatcher::Grayscale.matches(128, 128, 128));
        assert!(ColorMatcher::Grayscale.matches(100, 110, 105));
        assert!(!ColorMatcher::Grayscale.matches(255, 0, 0));
    }

    #[test]
    fn test_color_matchers() {
        assert!(ColorMatcher::RedIsh.matches(255, 0, 0));
        assert!(ColorMatcher::RedIsh.matches(200, 50, 50));
        assert!(!ColorMatcher::RedIsh.matches(0, 255, 0));

        assert!(ColorMatcher::GreenIsh.matches(0, 255, 0));
        assert!(!ColorMatcher::GreenIsh.matches(255, 0, 0));

        assert!(ColorMatcher::BlueIsh.matches(0, 0, 255));
        assert!(!ColorMatcher::BlueIsh.matches(255, 0, 0));

        assert!(ColorMatcher::YellowIsh.matches(255, 255, 0));
        assert!(ColorMatcher::CyanIsh.matches(0, 255, 255));
        assert!(ColorMatcher::MagentaIsh.matches(255, 0, 255));
    }

    #[test]
    fn test_color_assertion_exact() {
        let assertion = ColorAssertion::from_rgb(255, 128, 64);
        assertion.exact(255, 128, 64);
    }

    #[test]
    #[should_panic(expected = "Expected color")]
    fn test_color_assertion_exact_fails() {
        let assertion = ColorAssertion::from_rgb(255, 128, 64);
        assertion.exact(0, 0, 0);
    }

    #[test]
    fn test_color_assertion_matcher() {
        let assertion = ColorAssertion::from_rgb(255, 0, 0);
        assertion.assert(ColorMatcher::RedIsh);
    }
}

// New grayscale assertion tests (TDD Phase 1)
#[cfg(test)]
mod color_grayscale_tests {
    use super::ColorAssertion;

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
