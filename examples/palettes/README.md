# Sample Palettes

Use these files to quickly test import/export workflows in `rustycolour-app`.

## Included files

- `vivid-sample.gpl`: text palette for GIMP palette format import.
- `single-red.ase`: binary Adobe Swatch Exchange palette with one color.

## How to test

1. Run app: `cargo run -p rustycolour-app`
2. In right panel `Import`:
   - set path to `examples/palettes/vivid-sample.gpl`, click `Import GPL`
   - set path to `examples/palettes/single-red.ase`, click `Import ASE`
3. In `Export`, switch formats and use `Export to File`.
