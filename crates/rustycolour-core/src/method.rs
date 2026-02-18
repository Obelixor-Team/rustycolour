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
}

pub trait PaletteMethod: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> MethodCategory;
    fn generate(&self, request: &GenerationRequest) -> Palette;
}
