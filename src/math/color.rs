#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn clamp(self) -> Self {
        Self::new(self.r, self.g, self.b, self.a)
    }

    // Mix two colors with a factor in [0,1]
    // factor = 0 -> self, factor = 1 -> other
    pub fn mix(self, other: Color, factor: f32) -> Self {
        let t = factor.clamp(0.0, 1.0);
        let lerp = |a: f32, b: f32| a + (b - a) * t;
        Color::new(
            lerp(self.r, other.r),
            lerp(self.g, other.g),
            lerp(self.b, other.b),
            lerp(self.a, other.a),
        )
    }

    // Compute the complementary color (preserving alpha)
    pub fn complementary(self) -> Self {
        Color::new(1.0 - self.r, 1.0 - self.g, 1.0 - self.b, self.a)
    }

    // Common color constants (normalized sRGB values)
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const GRAY: Color = Color {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub fn to_rgba(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn to_linear_rgba(self) -> [f32; 4] {
        [
            Self::srgb_to_linear(self.r),
            Self::srgb_to_linear(self.g),
            Self::srgb_to_linear(self.b),
            self.a,
        ]
    }

    fn srgb_to_linear(component: f32) -> f32 {
        if component <= 0.04045 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    }
}

impl From<String> for Color {
    fn from(value: String) -> Self {
        Color::from(value.as_str())
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        let s = value.trim().to_ascii_lowercase();
        if s.starts_with('#') || s.starts_with("0x") {
            // Hex format
            let hex = s.trim_start_matches('#').trim_start_matches("0x");
            let (r, g, b, a) = match hex.len() {
                6 => (
                    u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
                    u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
                    u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
                    255,
                ),
                8 => (
                    u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
                    u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
                    u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
                    u8::from_str_radix(&hex[6..8], 16).unwrap_or(255),
                ),
                _ => (0, 0, 0, 255),
            };
            Color::from((r, g, b, a))
        } else if s.starts_with("rgb") {
            // RGB or RGBA format
            let nums: Vec<&str> = s
                .trim_start_matches("rgba")
                .trim_start_matches("rgb")
                .trim_start_matches('(')
                .trim_end_matches(')')
                .split(',')
                .map(|x| x.trim())
                .collect();
            let (r, g, b, a) = match nums.len() {
                3 => (
                    nums[0].parse::<u8>().unwrap_or(0),
                    nums[1].parse::<u8>().unwrap_or(0),
                    nums[2].parse::<u8>().unwrap_or(0),
                    255,
                ),
                4 => (
                    nums[0].parse::<u8>().unwrap_or(0),
                    nums[1].parse::<u8>().unwrap_or(0),
                    nums[2].parse::<u8>().unwrap_or(0),
                    (nums[3].parse::<f32>().unwrap_or(1.0) * 255.0) as u8,
                ),
                _ => (0, 0, 0, 255),
            };
            Color::from((r, g, b, a))
        } else if s.starts_with("hsl") {
            // HSL or HSLA format
            let nums: Vec<&str> = s
                .trim_start_matches("hsla")
                .trim_start_matches("hsl")
                .trim_start_matches('(')
                .trim_end_matches(')')
                .split(',')
                .map(|x| x.trim().trim_end_matches('%'))
                .collect();
            let (h, s, l, a) = match nums.len() {
                3 => (
                    nums[0].parse::<f32>().unwrap_or(0.0),
                    nums[1].parse::<f32>().unwrap_or(0.0) / 100.0,
                    nums[2].parse::<f32>().unwrap_or(0.0) / 100.0,
                    1.0,
                ),
                4 => (
                    nums[0].parse::<f32>().unwrap_or(0.0),
                    nums[1].parse::<f32>().unwrap_or(0.0) / 100.0,
                    nums[2].parse::<f32>().unwrap_or(0.0) / 100.0,
                    nums[3].parse::<f32>().unwrap_or(1.0),
                ),
                _ => (0.0, 0.0, 0.0, 1.0),
            };
            // Convert HSL to RGB
            let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
            let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
            let m = l - c / 2.0;

            let (r1, g1, b1) = if (0.0..60.0).contains(&h) {
                (c, x, 0.0)
            } else if (60.0..120.0).contains(&h) {
                (x, c, 0.0)
            } else if (120.0..180.0).contains(&h) {
                (0.0, c, x)
            } else if (180.0..240.0).contains(&h) {
                (0.0, x, c)
            } else if (240.0..300.0).contains(&h) {
                (x, 0.0, c)
            } else if (300.0..360.0).contains(&h) {
                (c, 0.0, x)
            } else {
                (0.0, 0.0, 0.0)
            };
            Color::new(r1 + m, g1 + m, b1 + m, a)
        } else {
            // Default to black
            Color::from((0u8, 0u8, 0u8, 255u8))
        }
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Color::new(r, g, b, 1.0)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Color::new(r, g, b, a)
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Color::new(r, g, b, 1.0)
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Color::new(r, g, b, a)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }
}

impl From<(u8, u8, u8, f32)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, f32)) -> Self {
        Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
    }
}

impl From<[u8; 3]> for Color {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
    }
}

impl From<[u8; 4]> for Color {
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Color::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }
}

use core::ops::{Add, AddAssign, Sub, SubAssign};

impl Add for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Self::Output {
        Color::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Sub for Color {
    type Output = Color;
    fn sub(self, rhs: Color) -> Self::Output {
        Color::new(
            self.r - rhs.r,
            self.g - rhs.g,
            self.b - rhs.b,
            self.a - rhs.a,
        )
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r = (self.r + rhs.r).clamp(0.0, 1.0);
        self.g = (self.g + rhs.g).clamp(0.0, 1.0);
        self.b = (self.b + rhs.b).clamp(0.0, 1.0);
        self.a = (self.a + rhs.a).clamp(0.0, 1.0);
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Color) {
        self.r = (self.r - rhs.r).clamp(0.0, 1.0);
        self.g = (self.g - rhs.g).clamp(0.0, 1.0);
        self.b = (self.b - rhs.b).clamp(0.0, 1.0);
        self.a = (self.a - rhs.a).clamp(0.0, 1.0);
    }
}
