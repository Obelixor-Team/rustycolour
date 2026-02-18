# Architecture

## Goals

- Maintainable separation between colour science, palette methods, and presentation.
- Extensible algorithm registry to support well-known and uncommon palette generation models.
- Deterministic outputs for reproducibility and testability.

## Layers

1. `rustycolour-core`
   - Domain color types and conversion boundary.
   - `PaletteMethod` trait and method registry.
   - Scoring utilities (contrast, distance metrics, quality heuristics).
   - Import/export models shared by GUI and CLI.
2. `rustycolour-app`
   - Interactive desktop UI.
   - State orchestration, previews, copy/export actions.
   - No raw colour math in UI modules.
3. `rustycolour-cli`
   - Scriptable generation and export.
   - Batch use-cases and CI integrations.

## Initial Core Contracts

```rust
pub trait PaletteMethod {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> MethodCategory;
    fn generate(&self, request: &GenerationRequest) -> Palette;
}
```

## Extension Model

- New method modules implement `PaletteMethod` and register in the method registry.
- UI and CLI query registry metadata and render controls dynamically.
- Each method includes deterministic tests and documentation references.
