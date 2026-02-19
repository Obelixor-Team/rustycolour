# rustycolour

A colour theory toolkit for developers, artists, and design systems.

## Status

Active beta on `dev`. Core generation, import/export, accessibility scoring, presets, and session workflows are implemented and tested.

## Repository layout

- `crates/rustycolour-core`: colour spaces, conversions, palette generation algorithms, scoring.
- `crates/rustycolour-app`: desktop GUI.
- `crates/rustycolour-cli`: command-line tooling for scripting and export automation.
- `docs/`: architecture, theory references, roadmap, and design decisions.
- `examples/`: reproducible examples and presets.
- `examples/palettes/`: sample GPL/ASE files for import/export testing.

## Features

- Built-in palette method registry.
- Generation methods:
  - Classical: monochrome, complementary, analogous, split-complementary, triadic, tetradic, square.
  - Perceptual: luminance ramp, OKLCH ramp, Lab DeltaE-spaced.
  - Advanced/uncommon: RYB complementary, annealed DeltaE spacing, golden-angle, cubehelix.
  - Accessibility-oriented: contrast-first, CVD-safe categorical.
- DeltaE support:
  - `CIE76` and `CIEDE2000` selectable for DeltaE-based generators.
- Accessibility support:
  - WCAG contrast ratio.
  - APCA 0.0.98G-style `Lc` scoring.
  - CVD simulation modes: deuteranopia, protanopia, tritanopia (with severity control).
- Import:
  - GPL (`.gpl`) and ASE (`.ase`) palettes.
- Export:
  - HEX, RGB, HSL, OKLCH, GPL, ASE, CSS variables, JSON.
  - Copy-to-clipboard and export-to-file workflows.
  - Auto file-extension sync based on selected export format.
- Workflow tools:
  - Session save/load (`.json`) for full working state.
  - Per-method preset save/load/delete with preset file persistence (`.json`).
- Testing:
  - Unit coverage for color metrics, uncommon generators, and import/export round-trips.

## Development

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p rustycolour-cli
cargo run -p rustycolour-app
```

## Quick start

1. Launch app: `cargo run -p rustycolour-app`
2. Try sample imports:
  - `examples/palettes/vivid-sample.gpl`
  - `examples/palettes/single-red.ase`
3. Generate/adjust palettes, then export to file from the Export panel.

## License

MIT. See `LICENSE`.
