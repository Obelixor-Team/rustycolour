use std::f32::consts::PI;

use crate::{
    method::{GenerationRequest, MethodCategory, PaletteMethod},
    metrics::{delta_e76, relative_luminance},
    palette::{Color, Palette},
};

pub struct Monochrome;

impl PaletteMethod for Monochrome {
    fn id(&self) -> &'static str {
        "monochrome"
    }

    fn name(&self) -> &'static str {
        "Monochrome"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Classical
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, _) = request.seed.to_hsl();
        let size = request.size.max(2);

        let colors = (0..size)
            .map(|i| {
                let t = i as f32 / (size as f32 - 1.0);
                let lightness = 0.1 + t * 0.8;
                Color::from_hsl(h, s, lightness)
            })
            .collect();

        Palette { colors }
    }
}

pub struct Complementary;

impl PaletteMethod for Complementary {
    fn id(&self) -> &'static str {
        "complementary"
    }

    fn name(&self) -> &'static str {
        "Complementary"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Classical
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, l) = request.seed.to_hsl();
        let size = request.size.max(2);

        let colors = (0..size)
            .map(|i| {
                let use_complement = i % 2 == 1;
                let hue = if use_complement { h + 180.0 } else { h };
                let tone = (l + (i as f32 * 0.06)).clamp(0.15, 0.85);
                Color::from_hsl(hue, s, tone)
            })
            .collect();

        Palette { colors }
    }
}

pub struct Analogous;

impl PaletteMethod for Analogous {
    fn id(&self) -> &'static str {
        "analogous"
    }

    fn name(&self) -> &'static str {
        "Analogous"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Classical
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, l) = request.seed.to_hsl();
        let size = request.size.max(3);
        let spread = request.params.analogous_spread_deg.clamp(20.0, 180.0);

        let colors = (0..size)
            .map(|i| {
                let position = i as f32 / (size as f32 - 1.0);
                let offset = (position - 0.5) * spread;
                Color::from_hsl(h + offset, s, l)
            })
            .collect();

        Palette { colors }
    }
}

pub struct SplitComplementary;

impl PaletteMethod for SplitComplementary {
    fn id(&self) -> &'static str {
        "split-complementary"
    }

    fn name(&self) -> &'static str {
        "Split Complementary"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Classical
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, l) = request.seed.to_hsl();
        let size = request.size.max(3);
        let split = request.params.split_complement_deg.clamp(10.0, 80.0);
        let wheel = [h, h + (180.0 - split), h + (180.0 + split)];

        let colors = (0..size)
            .map(|i| {
                let hue = wheel[i % wheel.len()];
                let tone = (l + (i as f32 * 0.04)).clamp(0.2, 0.85);
                Color::from_hsl(hue, s, tone)
            })
            .collect();

        Palette { colors }
    }
}

pub struct Triadic;

impl PaletteMethod for Triadic {
    fn id(&self) -> &'static str {
        "triadic"
    }

    fn name(&self) -> &'static str {
        "Triadic"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Classical
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, l) = request.seed.to_hsl();
        let size = request.size.max(3);

        let colors = (0..size)
            .map(|i| {
                let base = match i % 3 {
                    0 => h,
                    1 => h + 120.0,
                    _ => h + 240.0,
                };
                let tone = (l + (i as f32 * 0.05)).clamp(0.2, 0.85);
                Color::from_hsl(base, s, tone)
            })
            .collect();

        Palette { colors }
    }
}

pub struct Tetradic;

impl PaletteMethod for Tetradic {
    fn id(&self) -> &'static str {
        "tetradic"
    }

    fn name(&self) -> &'static str {
        "Tetradic"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Classical
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, l) = request.seed.to_hsl();
        let size = request.size.max(4);
        let offset = request.params.split_complement_deg.clamp(20.0, 80.0);
        let wheel = [h, h + offset, h + 180.0, h + (180.0 + offset)];

        let colors = (0..size)
            .map(|i| {
                let tone = (l + ((i % 2) as f32 * 0.08) - 0.04).clamp(0.18, 0.85);
                Color::from_hsl(wheel[i % wheel.len()], s, tone)
            })
            .collect();

        Palette { colors }
    }
}

pub struct Square;

impl PaletteMethod for Square {
    fn id(&self) -> &'static str {
        "square"
    }

    fn name(&self) -> &'static str {
        "Square"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Classical
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, l) = request.seed.to_hsl();
        let size = request.size.max(4);
        let wheel = [h, h + 90.0, h + 180.0, h + 270.0];

        let colors = (0..size)
            .map(|i| {
                let tone = (l + (i as f32 * 0.03)).clamp(0.2, 0.86);
                Color::from_hsl(wheel[i % wheel.len()], s, tone)
            })
            .collect();

        Palette { colors }
    }
}

pub struct LuminanceRamp;

impl PaletteMethod for LuminanceRamp {
    fn id(&self) -> &'static str {
        "luminance-ramp"
    }

    fn name(&self) -> &'static str {
        "Perceptual Luminance Ramp"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Perceptual
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, _) = request.seed.to_hsl();
        let size = request.size.max(2);
        let l_min = request.params.luminance_min.clamp(0.0, 1.0);
        let l_max = request
            .params
            .luminance_max
            .clamp(0.0, 1.0)
            .max(l_min + 0.05);

        let colors = (0..size)
            .map(|i| {
                let t = i as f32 / (size as f32 - 1.0);
                let target = l_min + (l_max - l_min) * t;
                color_with_target_luminance(h, s, target)
            })
            .collect();

        Palette { colors }
    }
}

pub struct OklchRamp;

impl PaletteMethod for OklchRamp {
    fn id(&self) -> &'static str {
        "oklch-ramp"
    }

    fn name(&self) -> &'static str {
        "OKLCH Ramp"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Perceptual
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (seed_l, seed_c, seed_h) = request.seed.to_oklch();
        let size = request.size.max(2);
        let l_min = request.params.luminance_min.clamp(0.0, 1.0);
        let l_max = request
            .params
            .luminance_max
            .clamp(0.0, 1.0)
            .max(l_min + 0.05);
        let chroma = (seed_c * request.params.oklch_chroma_scale.clamp(0.2, 2.0)).clamp(0.01, 0.37);

        let colors = (0..size)
            .map(|i| {
                let t = i as f32 / (size as f32 - 1.0);
                let l = l_min + (l_max - l_min) * t;
                let hue = seed_h + (t - 0.5) * 12.0;
                Color::from_oklch(l, chroma, hue)
            })
            .collect::<Vec<_>>();

        if colors.is_empty() {
            Palette {
                colors: vec![Color::from_oklch(seed_l, chroma, seed_h)],
            }
        } else {
            Palette { colors }
        }
    }
}

pub struct LabDeltaESpaced;

impl PaletteMethod for LabDeltaESpaced {
    fn id(&self) -> &'static str {
        "lab-deltae-spaced"
    }

    fn name(&self) -> &'static str {
        "Lab DeltaE Spaced"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Perceptual
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let size = request.size.max(2);
        let (seed_h, seed_s, seed_l) = request.seed.to_hsl();
        let mut selected = vec![request.seed];

        let candidates = (0..72)
            .map(|i| {
                let hue = seed_h + i as f32 * 5.0;
                let sat = (seed_s * 0.85 + 0.15).clamp(0.35, 0.95);
                let light = (seed_l * 0.8 + 0.2).clamp(0.22, 0.78);
                Color::from_hsl(hue, sat, light)
            })
            .collect::<Vec<_>>();

        let target = request.params.deltae_target.max(5.0);
        while selected.len() < size {
            let mut best_index = None;
            let mut best_score = -1.0f32;

            for (idx, candidate) in candidates.iter().enumerate() {
                if selected
                    .iter()
                    .any(|existing| candidate.to_hex_rgb() == existing.to_hex_rgb())
                {
                    continue;
                }

                let min_delta = selected
                    .iter()
                    .map(|existing| delta_e76(*candidate, *existing))
                    .fold(f32::INFINITY, f32::min);

                let score = if min_delta < target {
                    min_delta * 0.5
                } else {
                    min_delta
                };

                if score > best_score {
                    best_score = score;
                    best_index = Some(idx);
                }
            }

            match best_index {
                Some(idx) => selected.push(candidates[idx]),
                None => break,
            }
        }

        Palette { colors: selected }
    }
}

pub struct ContrastFirst;

impl PaletteMethod for ContrastFirst {
    fn id(&self) -> &'static str {
        "contrast-first"
    }

    fn name(&self) -> &'static str {
        "Contrast-First"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Accessibility
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (h, s, _) = request.seed.to_hsl();
        let size = request.size.max(2);
        let dark =
            color_with_target_luminance(h, s, request.params.luminance_min.clamp(0.02, 0.35));
        let light =
            color_with_target_luminance(h, s, request.params.luminance_max.clamp(0.65, 0.98));

        let colors = (0..size)
            .map(|i| if i % 2 == 0 { dark } else { light })
            .collect();

        Palette { colors }
    }
}

pub struct GoldenAngle;

impl PaletteMethod for GoldenAngle {
    fn id(&self) -> &'static str {
        "golden-angle"
    }

    fn name(&self) -> &'static str {
        "Golden Angle Sequence"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Advanced
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (seed_h, seed_s, seed_l) = request.seed.to_hsl();
        let size = request.size.max(2);
        let step = request.params.golden_angle_step_deg.clamp(10.0, 179.0);

        let colors = (0..size)
            .map(|i| {
                let hue = seed_h + step * i as f32;
                let sat = (seed_s * (0.8 + (i as f32 * 0.02))).clamp(0.35, 0.95);
                let light = (seed_l * (0.9 + (i as f32 * 0.01))).clamp(0.2, 0.82);
                Color::from_hsl(hue, sat, light)
            })
            .collect();

        Palette { colors }
    }
}

pub struct Cubehelix;

impl PaletteMethod for Cubehelix {
    fn id(&self) -> &'static str {
        "cubehelix"
    }

    fn name(&self) -> &'static str {
        "Cubehelix"
    }

    fn category(&self) -> MethodCategory {
        MethodCategory::Advanced
    }

    fn generate(&self, request: &GenerationRequest) -> Palette {
        let (seed_h, _, _) = request.seed.to_hsl();
        let size = request.size.max(2);
        let start = (seed_h / 360.0) * 3.0;
        let rotations = request.params.cubehelix_rotations.clamp(-4.0, 4.0);
        let hue_strength = request.params.cubehelix_hue_strength.clamp(0.0, 2.0);

        let colors = (0..size)
            .map(|i| {
                let t = i as f32 / (size as f32 - 1.0);
                let angle = 2.0 * PI * (start / 3.0 + rotations * t);
                let amp = hue_strength * t * (1.0 - t) / 2.0;
                let r = t + amp * (-0.14861 * angle.cos() + 1.78277 * angle.sin());
                let g = t + amp * (-0.29227 * angle.cos() - 0.90649 * angle.sin());
                let b = t + amp * (1.97294 * angle.cos());

                Color::from_rgba(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0)
            })
            .collect();

        Palette { colors }
    }
}

fn color_with_target_luminance(h: f32, s: f32, target_luminance: f32) -> Color {
    let target = target_luminance.clamp(0.0, 1.0);
    let mut low = 0.0;
    let mut high = 1.0;
    let mut candidate = Color::from_hsl(h, s, 0.5);

    for _ in 0..22 {
        let mid = (low + high) * 0.5;
        candidate = Color::from_hsl(h, s, mid);
        let lum = relative_luminance(candidate);

        if lum < target {
            low = mid;
        } else {
            high = mid;
        }
    }

    candidate
}
