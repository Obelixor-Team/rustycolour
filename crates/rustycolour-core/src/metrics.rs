//! Color quality and accessibility metrics used by generators and UI.

use crate::{method::CvdMode, palette::Color};

const APCA_NORM_BG: f32 = 0.56;
const APCA_NORM_TXT: f32 = 0.57;
const APCA_REV_BG: f32 = 0.65;
const APCA_REV_TXT: f32 = 0.62;
const APCA_SCALE_BOW: f32 = 1.14;
const APCA_SCALE_WOB: f32 = 1.14;
const APCA_BLACK_THRESHOLD: f32 = 0.022;
const APCA_BLACK_CLAMP: f32 = 1.414;
const APCA_DELTA_Y_MIN: f32 = 0.0005;
const APCA_LOW_CLIP: f32 = 0.1;
const APCA_LOW_OFFSET_BOW: f32 = 0.027;
const APCA_LOW_OFFSET_WOB: f32 = 0.027;

/// Convert an sRGB channel value in `[0,1]` to linear light space.
pub fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

/// Compute WCAG relative luminance for an RGBA color (alpha ignored).
pub fn relative_luminance(color: Color) -> f32 {
    let r = srgb_to_linear(color.r.clamp(0.0, 1.0));
    let g = srgb_to_linear(color.g.clamp(0.0, 1.0));
    let b = srgb_to_linear(color.b.clamp(0.0, 1.0));
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Compute WCAG 2 contrast ratio between two colors.
///
/// Returns ratio in `[1, 21]`, where larger means stronger contrast.
pub fn wcag_contrast_ratio(foreground: Color, background: Color) -> f32 {
    let l1 = relative_luminance(foreground);
    let l2 = relative_luminance(background);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Compute APCA `Lc` contrast score using 0.0.98G-style constants.
///
/// Positive values indicate dark text on light background; negative indicates reverse.
pub fn apca_contrast_lc(foreground: Color, background: Color) -> f32 {
    let txt_y = apca_luminance(foreground);
    let bg_y = apca_luminance(background);

    let txt_y = apca_black_soft_clamp(txt_y);
    let bg_y = apca_black_soft_clamp(bg_y);

    if (bg_y - txt_y).abs() < APCA_DELTA_Y_MIN {
        return 0.0;
    }

    if bg_y > txt_y {
        let sapc = (bg_y.powf(APCA_NORM_BG) - txt_y.powf(APCA_NORM_TXT)) * APCA_SCALE_BOW;
        if sapc < APCA_LOW_CLIP {
            0.0
        } else {
            (sapc - APCA_LOW_OFFSET_BOW) * 100.0
        }
    } else {
        let sapc = (bg_y.powf(APCA_REV_BG) - txt_y.powf(APCA_REV_TXT)) * APCA_SCALE_WOB;
        if sapc > -APCA_LOW_CLIP {
            0.0
        } else {
            (sapc + APCA_LOW_OFFSET_WOB) * 100.0
        }
    }
}

/// Compute CIE76 DeltaE distance between two colors in Lab space.
pub fn delta_e76(a: Color, b: Color) -> f32 {
    let (l1, a1, b1) = a.to_lab();
    let (l2, a2, b2) = b.to_lab();
    ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
}

/// Compute CIEDE2000 DeltaE distance between two colors in Lab space.
pub fn delta_e00(a: Color, b: Color) -> f32 {
    let (l1, a1, b1) = a.to_lab();
    let (l2, a2, b2) = b.to_lab();

    let c1 = (a1 * a1 + b1 * b1).sqrt();
    let c2 = (a2 * a2 + b2 * b2).sqrt();
    let c_bar = (c1 + c2) * 0.5;
    let c_bar7 = c_bar.powi(7);
    let g = 0.5 * (1.0 - (c_bar7 / (c_bar7 + 25_f32.powi(7))).sqrt());

    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;
    let c1p = (a1p * a1p + b1 * b1).sqrt();
    let c2p = (a2p * a2p + b2 * b2).sqrt();

    let h1p = hue_angle_deg(b1, a1p);
    let h2p = hue_angle_deg(b2, a2p);

    let dl = l2 - l1;
    let dc = c2p - c1p;
    let dh = if c1p * c2p == 0.0 {
        0.0
    } else {
        let mut d = h2p - h1p;
        if d > 180.0 {
            d -= 360.0;
        } else if d < -180.0 {
            d += 360.0;
        }
        d
    };
    let dh_term = 2.0 * (c1p * c2p).sqrt() * (deg_to_rad(dh * 0.5)).sin();

    let l_bar = (l1 + l2) * 0.5;
    let c_bar_p = (c1p + c2p) * 0.5;
    let h_bar_p = average_hue_deg(h1p, h2p, c1p, c2p);

    let t = 1.0 - 0.17 * deg_to_rad(h_bar_p - 30.0).cos()
        + 0.24 * deg_to_rad(2.0 * h_bar_p).cos()
        + 0.32 * deg_to_rad(3.0 * h_bar_p + 6.0).cos()
        - 0.20 * deg_to_rad(4.0 * h_bar_p - 63.0).cos();

    let delta_theta = 30.0 * (-(((h_bar_p - 275.0) / 25.0).powi(2))).exp();
    let rc = 2.0 * (c_bar_p.powi(7) / (c_bar_p.powi(7) + 25_f32.powi(7))).sqrt();
    let sl = 1.0 + (0.015 * (l_bar - 50.0).powi(2)) / (20.0 + (l_bar - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * c_bar_p;
    let sh = 1.0 + 0.015 * c_bar_p * t;
    let rt = -deg_to_rad(2.0 * delta_theta).sin() * rc;

    let kl = 1.0;
    let kc = 1.0;
    let kh = 1.0;

    let dl_term = dl / (kl * sl);
    let dc_term = dc / (kc * sc);
    let dh_term = dh_term / (kh * sh);

    (dl_term * dl_term + dc_term * dc_term + dh_term * dh_term + rt * dc_term * dh_term).sqrt()
}

/// Simulate color appearance under color vision deficiency.
///
/// `severity` in `[0,1]` blends from original color (`0`) to full simulation (`1`).
pub fn simulate_cvd(color: Color, mode: CvdMode, severity: f32) -> Color {
    let s = severity.clamp(0.0, 1.0);
    let (mr, mg, mb) = match mode {
        CvdMode::Deuteranopia => (
            [0.367, 0.861, -0.228],
            [0.280, 0.673, 0.047],
            [-0.012, 0.043, 0.969],
        ),
        CvdMode::Protanopia => (
            [0.152, 1.053, -0.205],
            [0.115, 0.786, 0.099],
            [-0.004, -0.048, 1.052],
        ),
        CvdMode::Tritanopia => (
            [1.255, -0.076, -0.179],
            [-0.078, 0.931, 0.148],
            [0.005, 0.691, 0.304],
        ),
    };

    let sim_r = (mr[0] * color.r + mr[1] * color.g + mr[2] * color.b).clamp(0.0, 1.0);
    let sim_g = (mg[0] * color.r + mg[1] * color.g + mg[2] * color.b).clamp(0.0, 1.0);
    let sim_b = (mb[0] * color.r + mb[1] * color.g + mb[2] * color.b).clamp(0.0, 1.0);

    Color::from_rgba(
        color.r + (sim_r - color.r) * s,
        color.g + (sim_g - color.g) * s,
        color.b + (sim_b - color.b) * s,
        color.a,
    )
}

fn apca_luminance(color: Color) -> f32 {
    let r = color.r.clamp(0.0, 1.0).powf(2.4);
    let g = color.g.clamp(0.0, 1.0).powf(2.4);
    let b = color.b.clamp(0.0, 1.0).powf(2.4);
    0.2126729 * r + 0.7151522 * g + 0.072175 * b
}

fn apca_black_soft_clamp(y: f32) -> f32 {
    if y < APCA_BLACK_THRESHOLD {
        y + (APCA_BLACK_THRESHOLD - y).powf(APCA_BLACK_CLAMP)
    } else {
        y
    }
}

fn hue_angle_deg(b: f32, a: f32) -> f32 {
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    h
}

fn average_hue_deg(h1p: f32, h2p: f32, c1p: f32, c2p: f32) -> f32 {
    if c1p * c2p == 0.0 {
        return h1p + h2p;
    }

    let diff = (h1p - h2p).abs();
    if diff <= 180.0 {
        (h1p + h2p) * 0.5
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) * 0.5
    } else {
        (h1p + h2p - 360.0) * 0.5
    }
}

fn deg_to_rad(deg: f32) -> f32 {
    deg.to_radians()
}

#[cfg(test)]
mod tests {
    use super::{apca_contrast_lc, delta_e00, delta_e76, simulate_cvd, wcag_contrast_ratio};
    use crate::{method::CvdMode, palette::Color};

    #[test]
    fn wcag_black_on_white_is_high() {
        let black = Color::from_rgb_u8(0, 0, 0);
        let white = Color::from_rgb_u8(255, 255, 255);
        let ratio = wcag_contrast_ratio(black, white);
        assert!(ratio > 20.0);
    }

    #[test]
    fn apca_has_directional_sign() {
        let black = Color::from_rgb_u8(0, 0, 0);
        let white = Color::from_rgb_u8(255, 255, 255);
        assert!(apca_contrast_lc(black, white) > 0.0);
        assert!(apca_contrast_lc(white, black) < 0.0);
    }

    #[test]
    fn apca_same_color_is_zero() {
        let mid = Color::from_rgb_u8(120, 120, 120);
        let score = apca_contrast_lc(mid, mid);
        assert!(score.abs() < f32::EPSILON);
    }

    #[test]
    fn deltae_is_zero_for_same_color() {
        let c = Color::from_rgb_u8(33, 66, 99);
        assert!(delta_e76(c, c) < f32::EPSILON);
    }

    #[test]
    fn deltae_increases_for_farther_colors() {
        let a = Color::from_rgb_u8(20, 20, 20);
        let b = Color::from_rgb_u8(40, 40, 40);
        let c = Color::from_rgb_u8(240, 240, 240);
        assert!(delta_e76(a, c) > delta_e76(a, b));
    }

    #[test]
    fn deltae00_is_zero_for_same_color() {
        let c = Color::from_rgb_u8(50, 100, 150);
        assert!(delta_e00(c, c) < f32::EPSILON);
    }

    #[test]
    fn cvd_simulation_changes_color_at_full_severity() {
        let c = Color::from_rgb_u8(90, 170, 45);
        let sim = simulate_cvd(c, CvdMode::Tritanopia, 1.0);
        assert_ne!(c.to_hex_rgb(), sim.to_hex_rgb());
    }
}
