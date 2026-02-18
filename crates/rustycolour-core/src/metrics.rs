use crate::palette::Color;

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
    // Approximate APCA-like Lc score for fast interactive preview.
    let y_txt = relative_luminance(foreground);
    let y_bg = relative_luminance(background);

    if (y_bg - y_txt).abs() < 0.0005 {
        return 0.0;
    }

    let sapc = if y_bg > y_txt {
        (y_bg.powf(0.56) - y_txt.powf(0.57)) * 1.14
    } else {
        -((y_txt.powf(0.62) - y_bg.powf(0.65)) * 1.14)
    };

    sapc * 100.0
}

pub fn delta_e76(a: Color, b: Color) -> f32 {
    let (l1, a1, b1) = a.to_lab();
    let (l2, a2, b2) = b.to_lab();
    ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
}
