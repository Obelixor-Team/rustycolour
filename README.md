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

## Status

Project scaffolding is in place. Core algorithms and GUI features are next.

## Development

```bash
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## License

MIT. See `LICENSE`.
