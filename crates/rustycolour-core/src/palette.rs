#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: 1.0,
        }
    }

    pub fn to_rgb_u8(self) -> (u8, u8, u8) {
        (
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    pub fn to_hex_rgb(self) -> String {
        let (r, g, b) = self.to_rgb_u8();
        format!("#{r:02X}{g:02X}{b:02X}")
    }

    pub fn from_hex_rgb(value: &str) -> Option<Self> {
        let hex = value.trim().trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self::from_rgb_u8(r, g, b))
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let r = self.r;
        let g = self.g;
        let b = self.b;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        let delta = max - min;

        let l = (max + min) / 2.0;
        if delta.abs() < f32::EPSILON {
            return (0.0, 0.0, l);
        }

        let s = delta / (1.0 - (2.0 * l - 1.0).abs());

        let mut h = if (max - r).abs() < f32::EPSILON {
            ((g - b) / delta) % 6.0
        } else if (max - g).abs() < f32::EPSILON {
            ((b - r) / delta) + 2.0
        } else {
            ((r - g) / delta) + 4.0
        };

        h *= 60.0;
        if h < 0.0 {
            h += 360.0;
        }

        (h, s.clamp(0.0, 1.0), l.clamp(0.0, 1.0))
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let hue = h.rem_euclid(360.0);
        let saturation = s.clamp(0.0, 1.0);
        let lightness = l.clamp(0.0, 1.0);

        let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
        let x = c * (1.0 - (((hue / 60.0) % 2.0) - 1.0).abs());
        let m = lightness - c / 2.0;

        let (r1, g1, b1) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: (r1 + m).clamp(0.0, 1.0),
            g: (g1 + m).clamp(0.0, 1.0),
            b: (b1 + m).clamp(0.0, 1.0),
            a: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub colors: Vec<Color>,
}

impl Palette {
    pub fn empty() -> Self {
        Self { colors: Vec::new() }
    }
}
