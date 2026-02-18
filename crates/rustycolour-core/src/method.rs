use crate::palette::{Color, Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodCategory {
    Classical,
    Perceptual,
    Advanced,
    Accessibility,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenerationRequest {
    pub seed: Color,
    pub size: usize,
    pub params: MethodParams,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MethodParams {
    pub analogous_spread_deg: f32,
    pub split_complement_deg: f32,
    pub golden_angle_step_deg: f32,
    pub cubehelix_rotations: f32,
    pub cubehelix_hue_strength: f32,
    pub luminance_min: f32,
    pub luminance_max: f32,
}

impl Default for MethodParams {
    fn default() -> Self {
        Self {
            analogous_spread_deg: 60.0,
            split_complement_deg: 30.0,
            golden_angle_step_deg: 137.507_76,
            cubehelix_rotations: -1.3,
            cubehelix_hue_strength: 1.2,
            luminance_min: 0.08,
            luminance_max: 0.92,
        }
    }
}

pub trait PaletteMethod: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> MethodCategory;
    fn generate(&self, request: &GenerationRequest) -> Palette;
}
