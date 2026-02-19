//! Palette method contracts and shared generation parameters.

use crate::palette::{Color, Palette};
use serde::{Deserialize, Serialize};

/// High-level grouping for palette generation strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodCategory {
    /// Traditional harmony rules based on hue wheel relationships.
    Classical,
    /// Perceptual color-space-aware approaches (e.g. OKLCH/Lab).
    Perceptual,
    /// Experimental or niche methods outside common harmony sets.
    Advanced,
    /// Accessibility-oriented generation focused on robust distinction/contrast.
    Accessibility,
}

/// Request payload passed to palette generators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenerationRequest {
    /// Seed color for generation.
    pub seed: Color,
    /// Desired output size. Generators may clamp to method-specific minimums.
    pub size: usize,
    /// Tunable parameters shared across methods.
    pub params: MethodParams,
}

/// DeltaE metric options for spacing/scoring methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeltaEMetric {
    /// CIE76 Euclidean Lab distance.
    #[default]
    E76,
    /// CIEDE2000 perceptual distance.
    E00,
}

/// Color vision deficiency simulation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CvdMode {
    /// Deuteranopia simulation.
    #[default]
    Deuteranopia,
    /// Protanopia simulation.
    Protanopia,
    /// Tritanopia simulation.
    Tritanopia,
}

/// Shared method parameters used by generator implementations.
///
/// Parameters may be ignored by methods that do not use them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MethodParams {
    pub analogous_spread_deg: f32,
    pub split_complement_deg: f32,
    pub golden_angle_step_deg: f32,
    pub cubehelix_rotations: f32,
    pub cubehelix_hue_strength: f32,
    pub luminance_min: f32,
    pub luminance_max: f32,
    pub oklch_chroma_scale: f32,
    pub deltae_target: f32,
    pub deltae_metric: DeltaEMetric,
    pub ryb_mix: f32,
    pub cvd_mode: CvdMode,
    pub cvd_severity: f32,
    pub anneal_iterations: usize,
    pub anneal_temperature: f32,
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
            oklch_chroma_scale: 1.0,
            deltae_target: 22.0,
            deltae_metric: DeltaEMetric::E76,
            ryb_mix: 1.0,
            cvd_mode: CvdMode::Deuteranopia,
            cvd_severity: 1.0,
            anneal_iterations: 120,
            anneal_temperature: 1.0,
        }
    }
}

/// Interface implemented by all palette generation methods.
pub trait PaletteMethod: Send + Sync {
    /// Stable programmatic identifier (used by UI/session serialization).
    fn id(&self) -> &'static str;
    /// Human-readable method name.
    fn name(&self) -> &'static str;
    /// Category displayed in UI and used for grouping.
    fn category(&self) -> MethodCategory;
    /// Generate a palette for the given request.
    fn generate(&self, request: &GenerationRequest) -> Palette;
}
