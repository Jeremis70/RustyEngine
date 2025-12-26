/// Represents a color in RGBA format with values normalized to [0.0, 1.0].
///
/// # VSCode Color Picker Compatibility
///
/// For VSCode color extensions to work, use string literals or direct constructors:
/// - `Color::from("rgb(255, 128, 0)")` or `Color::rgb(255, 128, 0)`
/// - `Color::from("rgba(255, 128, 0, 0.5)")` or `Color::rgba(255, 128, 0, 0.5)`
/// - `Color::from("hsl(120, 100%, 50%)")` or `Color::hsl(120.0, 1.0, 0.5)`
/// - `Color::from("#FF8000")`
///
/// # Examples
///
/// ```
/// // RGB/RGBA with u8 values (0-255) - VSCode compatible!
/// let orange = Color::rgb(255, 128, 0);
/// let semi_red = Color::rgba(255, 0, 0, 0.5);
///
/// // String formats also work
/// let green = Color::from("rgb(0, 255, 0)");
/// let blue_hex = Color::from("#0080FF");
///
/// // For normalized f32 values (0.0-1.0), use _f32 variants
/// let color_f32 = Color::rgb_f32(1.0, 0.5, 0.0);
/// let color_alpha_f32 = Color::rgba_f32(1.0, 0.5, 0.0, 0.5);
///
/// // Using constants
/// let white = Color::WHITE;
/// let red = Color::RED;
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Creates a new color with RGBA components clamped to [0.0, 1.0].
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Creates an RGB color from u8 values (0-255) with full opacity.
    /// This is the CSS-standard format, compatible with VSCode color extensions.
    /// # Example
    /// ```
    /// let orange = Color::rgb(255, 128, 0);
    /// ```
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
    }

    /// Creates an RGBA color from u8 RGB values (0-255) and f32 alpha (0.0-1.0).
    /// This is the CSS-standard format, compatible with VSCode color extensions.
    /// # Example
    /// ```
    /// let semi_red = Color::rgba(255, 0, 0, 0.5);
    /// ```
    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
    }

    /// Creates an RGB color from normalized f32 values (0.0-1.0).
    /// Use this when working with normalized color values.
    /// # Example
    /// ```
    /// let orange = Color::rgb_f32(1.0, 0.5, 0.0);
    /// ```
    pub fn rgb_f32(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Creates an RGBA color from normalized f32 values (0.0-1.0).
    /// Use this when working with normalized color values.
    /// # Example
    /// ```
    /// let semi_orange = Color::rgba_f32(1.0, 0.5, 0.0, 0.5);
    /// ```
    pub fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(r, g, b, a)
    }

    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#').trim_start_matches("0x");
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
        Self::rgba(r, g, b, a as f32 / 255.0)
    }

    pub fn from_rgb_str(rgb_str: &str) -> Self {
        let nums: Vec<&str> = rgb_str
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
        Self::rgba(r, g, b, a as f32 / 255.0)
    }

    pub fn hsl(h: f32, s: f32, l: f32) -> Self {
        Self::hsla(h, s, l, 1.0)
    }

    pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
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
        Self::new(r1 + m, g1 + m, b1 + m, a)
    }

    pub fn from_hsl_str(hsl_str: &str) -> Self {
        let nums: Vec<&str> = hsl_str
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
        Self::hsla(h, s, l, a)
    }

    /// Parses a color from a string. Supports multiple formats:
    /// - Hex: "#FF8000", "#FF8000AA", "0xFF8000"
    /// - RGB: "rgb(255, 128, 0)", "rgba(255, 128, 0, 0.5)"
    /// - HSL: "hsl(30, 100%, 50%)", "hsla(30, 100%, 50%, 0.5)"
    /// - CSS color names: "red", "blue", "green", etc.
    pub fn from_string(value: &str) -> Self {
        let s = value.trim().to_ascii_lowercase();
        if s.starts_with('#') || s.starts_with("0x") {
            Self::from_hex(&s)
        } else if s.starts_with("rgb") {
            Self::from_rgb_str(&s)
        } else if s.starts_with("hsl") {
            Self::from_hsl_str(&s)
        } else {
            // Try to parse as CSS color name
            Self::from_css_name(&s)
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
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const ORANGE: Color = Color {
        r: 1.0,
        g: 0.647,
        b: 0.0,
        a: 1.0,
    };
    pub const PURPLE: Color = Color {
        r: 0.502,
        g: 0.0,
        b: 0.502,
        a: 1.0,
    };
    pub const PINK: Color = Color {
        r: 1.0,
        g: 0.753,
        b: 0.796,
        a: 1.0,
    };
    pub const BROWN: Color = Color {
        r: 0.647,
        g: 0.165,
        b: 0.165,
        a: 1.0,
    };

    /// Parses a CSS color name. Returns black if the name is not recognized.
    fn from_css_name(name: &str) -> Self {
        match name {
            "white" => Self::WHITE,
            "black" => Self::BLACK,
            "red" => Self::RED,
            "green" => Self::GREEN,
            "blue" => Self::BLUE,
            "yellow" => Self::YELLOW,
            "cyan" => Self::CYAN,
            "magenta" => Self::MAGENTA,
            "orange" => Self::ORANGE,
            "purple" => Self::PURPLE,
            "pink" => Self::PINK,
            "brown" => Self::BROWN,
            "gray" | "grey" => Self::GRAY,
            "transparent" => Self::TRANSPARENT,
            _ => Self::BLACK, // Default to black for unrecognized names
        }
    }

    /// Converts the color to a hex string (e.g., "#FF8000" or "#FF8000AA" if alpha < 1.0).
    pub fn to_hex(self) -> String {
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        let a = (self.a * 255.0).round() as u8;

        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }

    /// Converts the color to an RGB string (e.g., "rgb(255, 128, 0)").
    pub fn to_rgb_string(self) -> String {
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        format!("rgb({}, {}, {})", r, g, b)
    }

    /// Converts the color to an RGBA string (e.g., "rgba(255, 128, 0, 0.5)").
    pub fn to_rgba_string(self) -> String {
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        format!("rgba({}, {}, {}, {})", r, g, b, self.a)
    }

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
        Self::from_string(&value)
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        Self::from_string(value)
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Self::rgb_f32(r, g, b)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Self::rgba_f32(r, g, b, a)
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Self::rgb_f32(r, g, b)
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self::rgba_f32(r, g, b, a)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Self::rgba(r, g, b, a as f32 / 255.0)
    }
}

impl From<(u8, u8, u8, f32)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, f32)) -> Self {
        Self::rgba(r, g, b, a)
    }
}

impl From<[u8; 3]> for Color {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<[u8; 4]> for Color {
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Self::rgba(r, g, b, a as f32 / 255.0)
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
