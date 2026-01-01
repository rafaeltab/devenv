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
    pub fn matches(&self, _r: u8, _g: u8, _b: u8) -> bool {
        todo!()
    }
}

pub struct ColorAssertion {
    // Will be implemented
}

impl ColorAssertion {
    pub fn assert(&self, _matcher: ColorMatcher) {
        todo!()
    }

    pub fn exact(&self, _r: u8, _g: u8, _b: u8) {
        todo!()
    }
}
