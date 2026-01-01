use crate::terminal::TerminalBuffer;

#[derive(Debug, Clone, Copy)]
pub enum ColorMatcher {
    Grayscale,
    YellowIsh,
    RedIsh,
    GreenIsh,
    BlueIsh,
    CyanIsh,
    MagentaIsh,
    Hue { min: f32, max: f32 },
    Saturation { min: f32, max: f32 },
    Lightness { min: f32, max: f32 },
}

impl ColorMatcher {
    pub fn matches(&self, r: u8, g: u8, b: u8) -> bool {
        let (h, s, l) = rgb_to_hsl(r, g, b);

        match self {
            ColorMatcher::Grayscale => s < 0.1,
            ColorMatcher::YellowIsh => (h >= 30.0 && h <= 90.0) && s > 0.3,
            ColorMatcher::RedIsh => (h >= 330.0 || h <= 30.0) && s > 0.3,
            ColorMatcher::GreenIsh => (h >= 90.0 && h <= 150.0) && s > 0.3,
            ColorMatcher::BlueIsh => (h >= 210.0 && h <= 270.0) && s > 0.3,
            ColorMatcher::CyanIsh => (h >= 150.0 && h <= 210.0) && s > 0.3,
            ColorMatcher::MagentaIsh => (h >= 270.0 && h <= 330.0) && s > 0.3,
            ColorMatcher::Hue { min, max } => h >= *min && h <= *max,
            ColorMatcher::Saturation { min, max } => s >= *min && s <= *max,
            ColorMatcher::Lightness { min, max } => l >= *min && l <= *max,
        }
    }
}

pub struct ColorAssertion {
    color: Option<(u8, u8, u8)>,
    context: String,
    dump_on_fail: bool,
    screen_snapshot: TerminalBuffer,
}

impl ColorAssertion {
    pub fn new(
        color: Option<(u8, u8, u8)>,
        context: String,
        dump_on_fail: bool,
        screen_snapshot: &TerminalBuffer,
    ) -> Self {
        Self {
            color,
            context,
            dump_on_fail,
            screen_snapshot: screen_snapshot.clone(),
        }
    }

    pub fn assert(&self, matcher: ColorMatcher) {
        match self.color {
            None => panic!("{}: no color set (using terminal default)", self.context),
            Some((r, g, b)) => {
                if !matcher.matches(r, g, b) {
                    if self.dump_on_fail {
                        eprintln!("\n=== Screen Dump ===");
                        eprintln!("{}", self.screen_snapshot.render());
                        eprintln!("===================\n");
                    }
                    panic!(
                        "{}: expected color to match {:?}, but got RGB({}, {}, {})",
                        self.context, matcher, r, g, b
                    );
                }
            }
        }
    }

    pub fn exact(&self, r: u8, g: u8, b: u8) {
        match self.color {
            None => panic!("{}: no color set (using terminal default)", self.context),
            Some((actual_r, actual_g, actual_b)) => {
                if (actual_r, actual_g, actual_b) != (r, g, b) {
                    if self.dump_on_fail {
                        eprintln!("\n=== Screen Dump ===");
                        eprintln!("{}", self.screen_snapshot.render());
                        eprintln!("===================\n");
                    }
                    panic!(
                        "{}: expected RGB({}, {}, {}), but got RGB({}, {}, {})",
                        self.context, r, g, b, actual_r, actual_g, actual_b
                    );
                }
            }
        }
    }
}

// RGB to HSL conversion
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, l); // Grayscale
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, l)
}
