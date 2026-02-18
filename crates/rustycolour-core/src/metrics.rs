use crate::palette::Color;

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

pub fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

pub fn relative_luminance(color: Color) -> f32 {
    let r = srgb_to_linear(color.r.clamp(0.0, 1.0));
    let g = srgb_to_linear(color.g.clamp(0.0, 1.0));
    let b = srgb_to_linear(color.b.clamp(0.0, 1.0));
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

pub fn wcag_contrast_ratio(foreground: Color, background: Color) -> f32 {
    let l1 = relative_luminance(foreground);
    let l2 = relative_luminance(background);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

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

pub fn delta_e76(a: Color, b: Color) -> f32 {
    let (l1, a1, b1) = a.to_lab();
    let (l2, a2, b2) = b.to_lab();
    ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
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

#[cfg(test)]
mod tests {
    use super::{apca_contrast_lc, delta_e76, wcag_contrast_ratio};
    use crate::palette::Color;

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
}
