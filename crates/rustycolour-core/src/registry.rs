use std::sync::Arc;

use crate::{
    generators::{Analogous, Complementary, Cubehelix, GoldenAngle, Monochrome, Triadic},
    method::{GenerationRequest, PaletteMethod},
    palette::Palette,
};

pub struct MethodRegistry {
    methods: Vec<Arc<dyn PaletteMethod>>,
}

impl MethodRegistry {
    pub fn with_builtins() -> Self {
        let methods: Vec<Arc<dyn PaletteMethod>> = vec![
            Arc::new(Monochrome),
            Arc::new(Complementary),
            Arc::new(Analogous),
            Arc::new(Triadic),
            Arc::new(GoldenAngle),
            Arc::new(Cubehelix),
        ];
        Self { methods }
    }

    pub fn methods(&self) -> &[Arc<dyn PaletteMethod>] {
        &self.methods
    }

    pub fn generate_by_id(&self, id: &str, request: &GenerationRequest) -> Option<Palette> {
        self.methods
            .iter()
            .find(|method| method.id() == id)
            .map(|method| method.generate(request))
    }
}
