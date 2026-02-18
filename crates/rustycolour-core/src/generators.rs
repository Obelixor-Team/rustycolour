use std::f32::consts::PI;

use crate::{
    method::{GenerationRequest, MethodCategory, PaletteMethod},
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
        let spread = 60.0;

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
        let step = 137.50776;

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
        let rotations = -1.3;
        let hue_strength = 1.2;

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
