# rustycolour

A colour theory toolkit for developers, artists, and design systems.

## Vision

`rustycolour` combines classical and advanced colour theory models in one maintainable Rust codebase with both GUI and CLI workflows.

## Repository layout

- `crates/rustycolour-core`: colour spaces, conversions, palette generation algorithms, scoring.
- `crates/rustycolour-app`: desktop GUI.
- `crates/rustycolour-cli`: command-line tooling for scripting and export automation.
- `docs/`: architecture, theory references, roadmap, and design decisions.
- `examples/`: reproducible examples and presets.

## Current capabilities

- Built-in palette method registry.
- Initial generation methods:
  - Monochrome
  - Complementary
  - Analogous
  - Split-complementary
  - Triadic
  - Tetradic
  - Square
  - RYB complementary variant
  - Perceptual luminance ramp
  - OKLCH ramp
  - Lab DeltaE-spaced palette
  - Annealed DeltaE spacing
  - Contrast-first accessibility ramp
  - CVD-safe categorical set
  - Golden-angle sequence
  - Cubehelix
- Desktop GUI with method selection, dynamic parameter controls, swatch rendering, and one-click copy for HEX/RGB/HSL.
- Export panel for HEX/RGB/HSL/OKLCH, CSS variables, and JSON.
- Contrast scoring panel with WCAG ratios and APCA 0.0.98G-style Lc scoring.
- Session save/load to JSON for seed, method, parameters, and generated swatches.
- CLI scaffold with generation example output.

## Development

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p rustycolour-cli
cargo run -p rustycolour-app
```

## License

MIT. See `LICENSE`.
