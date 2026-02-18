use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodCategory {
    Classical,
    Perceptual,
    Advanced,
    Accessibility,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenerationRequest {
    pub seed: crate::palette::Color,
    pub size: usize,
}

pub trait PaletteMethod {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> MethodCategory;
    fn generate(&self, request: &GenerationRequest) -> Palette;
}
