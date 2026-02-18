pub mod generators;
pub mod method;
pub mod palette;
pub mod registry;

pub use method::{GenerationRequest, MethodCategory, MethodParams, PaletteMethod};
pub use palette::{Color, Palette};
pub use registry::MethodRegistry;
