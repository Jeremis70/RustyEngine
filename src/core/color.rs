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

//Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_string() {
        let color = Color::from(String::from("hsla(11, 100%, 32%, 0.76)"));
        // Values should be normalized; alpha should match input
        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);
        assert!((color.a - 0.76).abs() < 1e-6);
    }
    #[test]
    fn test_color_from_str() {
        let color = Color::from("#FF5733");
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.34117648);
        assert_eq!(color.b, 0.2);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_hsl_str() {
        let color = Color::from("hsl(11, 100%, 32%)");
        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_hsla_str() {
        let color = Color::from("hsla(11, 100%, 32%, 0.76)");
        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);
        assert!((color.a - 0.76).abs() < 1e-6);
    }
    #[test]
    fn test_color_from_rgb_str() {
        let color = Color::from("rgb(255, 87, 51)");
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.34117648);
        assert_eq!(color.b, 0.2);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_rgba_str() {
        let color = Color::from("rgba(255, 87, 51, 0.76)");
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.34117648);
        assert_eq!(color.b, 0.2);
        assert_eq!(color.a, 0.75686276);
    }
    #[test]
    fn test_color_from_hex_hash_str() {
        let color = Color::from("#FF5733");
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.34117648);
        assert_eq!(color.b, 0.2);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_hex_0x_str() {
        let color = Color::from("0xFF5733");
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.34117648);
        assert_eq!(color.b, 0.2);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_three_f32() {
        let color = Color::from((0.5, 0.5, 0.5));
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.5);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_four_f32() {
        let color = Color::from((0.5, 0.5, 0.5, 0.5));
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.5);
        assert_eq!(color.a, 0.5);
    }
    #[test]
    fn test_color_from_three_f32_array() {
        let color = Color::from([0.5, 0.5, 0.5]);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.5);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_four_f32_array() {
        let color = Color::from([0.5, 0.5, 0.5, 0.5]);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.5);
        assert_eq!(color.a, 0.5);
    }
    #[test]
    fn test_color_from_three_u8() {
        let color = Color::from((128u8, 128u8, 128u8));
        assert_eq!(color.r, 0.5019608);
        assert_eq!(color.g, 0.5019608);
        assert_eq!(color.b, 0.5019608);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_four_u8() {
        let color = Color::from((128u8, 128u8, 128u8, 128u8));
        assert_eq!(color.r, 0.5019608);
        assert_eq!(color.g, 0.5019608);
        assert_eq!(color.b, 0.5019608);
        assert_eq!(color.a, 0.5019608);
    }
    #[test]
    fn test_color_from_three_u8_array() {
        let color = Color::from([128u8, 128u8, 128u8]);
        assert_eq!(color.r, 0.5019608);
        assert_eq!(color.g, 0.5019608);
        assert_eq!(color.b, 0.5019608);
        assert_eq!(color.a, 1.0);
    }
    #[test]
    fn test_color_from_four_u8_array() {
        let color = Color::from([128u8, 128u8, 128u8, 128u8]);
        assert_eq!(color.r, 0.5019608);
        assert_eq!(color.g, 0.5019608);
        assert_eq!(color.b, 0.5019608);
        assert_eq!(color.a, 0.5019608);
    }

    #[test]
    fn test_color_add_operator() {
        let a = Color::from((0.3, 0.4, 0.5, 0.6));
        let b = Color::from((0.5, 0.6, 0.7, 0.5));
        let c = a + b;
        assert_eq!(c.r, 0.8);
        assert_eq!(c.g, 1.0); // clamped
        assert_eq!(c.b, 1.0); // clamped
        assert_eq!(c.a, 1.0); // clamped
    }

    #[test]
    fn test_color_sub_operator() {
        let a = Color::from((0.3, 0.4, 0.5, 0.6));
        let b = Color::from((0.5, 0.1, 0.7, 0.7));
        let c = a - b;
        assert_eq!(c.r, 0.0); // clamped
        assert_eq!(c.g, 0.3);
        assert_eq!(c.b, 0.0); // clamped
        assert_eq!(c.a, 0.0); // clamped
    }

    #[test]
    fn test_color_add_assign_operator() {
        let mut c = Color::from((0.2, 0.2, 0.2, 0.9));
        c += Color::from((0.9, 0.5, 0.1, 0.2));
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.7);
        assert_eq!(c.b, 0.3);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_sub_assign_operator() {
        let mut c = Color::from((0.9, 0.5, 0.6, 0.4));
        c -= Color::from((0.3, 0.6, 0.2, 0.6));
        assert!((c.r - 0.6).abs() < 1e-6);
        assert_eq!(c.g, 0.0);
        assert!((c.b - 0.4).abs() < 1e-6);
        assert_eq!(c.a, 0.0);
    }

    #[test]
    fn test_color_mix_factor_edges() {
        let a = Color::from((0.2, 0.4, 0.6, 0.8));
        let b = Color::from((0.8, 0.6, 0.4, 0.2));
        let c0 = a.mix(b, 0.0);
        let c1 = a.mix(b, 1.0);
        assert!((c0.r - a.r).abs() < 1e-6);
        assert!((c0.g - a.g).abs() < 1e-6);
        assert!((c0.b - a.b).abs() < 1e-6);
        assert!((c0.a - a.a).abs() < 1e-6);
        assert!((c1.r - b.r).abs() < 1e-6);
        assert!((c1.g - b.g).abs() < 1e-6);
        assert!((c1.b - b.b).abs() < 1e-6);
        assert!((c1.a - b.a).abs() < 1e-6);
    }

    #[test]
    fn test_color_mix_mid_factor() {
        let a = Color::from((0.0, 0.5, 1.0, 0.0));
        let b = Color::from((1.0, 0.5, 0.0, 1.0));
        let c = a.mix(b, 0.5);
        assert!((c.r - 0.5).abs() < 1e-6);
        assert!((c.g - 0.5).abs() < 1e-6);
        assert!((c.b - 0.5).abs() < 1e-6);
        assert!((c.a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_color_complementary() {
        let c = Color::from((0.25, 0.5, 0.75, 0.6));
        let comp = c.complementary();
        assert!((comp.r - 0.75).abs() < 1e-6);
        assert!((comp.g - 0.5).abs() < 1e-6);
        assert!((comp.b - 0.25).abs() < 1e-6);
        assert!((comp.a - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_color_to_linear_rgba_roundtrip() {
        let color = Color::from("#accac1");
        let linear = color.to_linear_rgba();
        assert!((linear_to_srgb(linear[0]) - color.r).abs() < 1e-6);
        assert!((linear_to_srgb(linear[1]) - color.g).abs() < 1e-6);
        assert!((linear_to_srgb(linear[2]) - color.b).abs() < 1e-6);
        assert!((linear[3] - color.a).abs() < 1e-6);
    }

    fn linear_to_srgb(component: f32) -> f32 {
        if component <= 0.0031308 {
            component * 12.92
        } else {
            1.055 * component.powf(1.0 / 2.4) - 0.055
        }
    }
}
