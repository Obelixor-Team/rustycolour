use serde_json::json;

use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    HexList,
    RgbList,
    HslList,
    OklchList,
    Gpl,
    Ase,
    CssVariables,
    Json,
}

impl ExportFormat {
    pub const ALL: [Self; 8] = [
        Self::HexList,
        Self::RgbList,
        Self::HslList,
        Self::OklchList,
        Self::Gpl,
        Self::Ase,
        Self::CssVariables,
        Self::Json,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::HexList => "HEX list",
            Self::RgbList => "RGB list",
            Self::HslList => "HSL list",
            Self::OklchList => "OKLCH list",
            Self::Gpl => "GPL palette",
            Self::Ase => "ASE palette",
            Self::CssVariables => "CSS variables",
            Self::Json => "JSON",
        }
    }
}

pub fn export_palette(palette: &Palette, format: ExportFormat, css_prefix: &str) -> String {
    match format {
        ExportFormat::HexList => palette
            .colors
            .iter()
            .map(|color| color.to_hex_rgb())
            .collect::<Vec<_>>()
            .join("\n"),
        ExportFormat::RgbList => palette
            .colors
            .iter()
            .map(|color| {
                let (r, g, b) = color.to_rgb_u8();
                format!("rgb({r}, {g}, {b})")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        ExportFormat::HslList => palette
            .colors
            .iter()
            .map(|color| {
                let (h, s, l) = color.to_hsl();
                format!("hsl({h:.0}deg, {:.1}%, {:.1}%)", s * 100.0, l * 100.0)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        ExportFormat::OklchList => palette
            .colors
            .iter()
            .map(|color| {
                let (l, c, h) = color.to_oklch();
                format!("oklch({:.3} {:.3} {:.1})", l, c, h)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        ExportFormat::Gpl => {
            let name = css_prefix.trim();
            let name = if name.is_empty() { "rustycolour" } else { name };
            let mut lines = vec![
                "GIMP Palette".to_owned(),
                format!("Name: {name}"),
                "Columns: 6".to_owned(),
                "#".to_owned(),
            ];
            lines.extend(palette.colors.iter().enumerate().map(|(index, color)| {
                let (r, g, b) = color.to_rgb_u8();
                format!("{r:>3} {g:>3} {b:>3}\tColor {}", index + 1)
            }));
            lines.join("\n")
        }
        ExportFormat::Ase => {
            "Binary ASE format selected. Use \"Export to File\" to write .ase.".to_owned()
        }
        ExportFormat::CssVariables => {
            let prefix = css_prefix.trim();
            let prefix = if prefix.is_empty() { "color" } else { prefix };
            palette
                .colors
                .iter()
                .enumerate()
                .map(|(index, color)| format!("--{prefix}-{}: {};", index + 1, color.to_hex_rgb()))
                .collect::<Vec<_>>()
                .join("\n")
        }
        ExportFormat::Json => {
            let items = palette
                .colors
                .iter()
                .enumerate()
                .map(|(index, color)| {
                    let (r, g, b) = color.to_rgb_u8();
                    let (h, s, l) = color.to_hsl();
                    let (okl, okc, okh) = color.to_oklch();
                    json!({
                        "index": index + 1,
                        "hex": color.to_hex_rgb(),
                        "rgb": {"r": r, "g": g, "b": b},
                        "hsl": {"h": h, "s": s, "l": l},
                        "oklch": {"l": okl, "c": okc, "h": okh}
                    })
                })
                .collect::<Vec<_>>();

            serde_json::to_string_pretty(&json!({ "palette": items }))
                .unwrap_or_else(|_| "{\"palette\":[]}".to_owned())
        }
    }
}

pub fn export_palette_bytes(
    palette: &Palette,
    format: ExportFormat,
    css_prefix: &str,
) -> Result<Vec<u8>, String> {
    match format {
        ExportFormat::Ase => export_ase_bytes(palette),
        _ => Ok(export_palette(palette, format, css_prefix).into_bytes()),
    }
}

fn export_ase_bytes(palette: &Palette) -> Result<Vec<u8>, String> {
    if palette.colors.is_empty() {
        return Err("Cannot export empty palette to ASE".to_owned());
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"ASEF");
    bytes.extend_from_slice(&1u16.to_be_bytes()); // major
    bytes.extend_from_slice(&0u16.to_be_bytes()); // minor
    bytes.extend_from_slice(&(palette.colors.len() as u32).to_be_bytes());

    for (index, color) in palette.colors.iter().enumerate() {
        let name = format!("Color {}", index + 1);
        let mut block = Vec::new();

        // UTF-16BE name, including null terminator.
        let mut utf16 = name.encode_utf16().collect::<Vec<_>>();
        utf16.push(0);
        block.extend_from_slice(&(utf16.len() as u16).to_be_bytes());
        for unit in utf16 {
            block.extend_from_slice(&unit.to_be_bytes());
        }

        block.extend_from_slice(b"RGB ");
        block.extend_from_slice(&color.r.clamp(0.0, 1.0).to_bits().to_be_bytes());
        block.extend_from_slice(&color.g.clamp(0.0, 1.0).to_bits().to_be_bytes());
        block.extend_from_slice(&color.b.clamp(0.0, 1.0).to_bits().to_be_bytes());
        block.extend_from_slice(&0u16.to_be_bytes()); // global color type

        bytes.extend_from_slice(&0x0001u16.to_be_bytes());
        bytes.extend_from_slice(&(block.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&block);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{ExportFormat, export_palette, export_palette_bytes};
    use crate::{import::import_ase, import::import_gpl, palette::Color, palette::Palette};

    #[test]
    fn gpl_round_trip_preserves_rgb_values() {
        let palette = Palette {
            colors: vec![
                Color::from_rgb_u8(255, 0, 0),
                Color::from_rgb_u8(12, 34, 56),
                Color::from_rgb_u8(0, 255, 200),
            ],
        };

        let gpl = export_palette(&palette, ExportFormat::Gpl, "RoundTrip");
        let imported = import_gpl(&gpl).expect("exported GPL should be importable");

        assert_eq!(imported.colors.len(), palette.colors.len());
        for (left, right) in imported.colors.iter().zip(palette.colors.iter()) {
            assert_eq!(left.to_rgb_u8(), right.to_rgb_u8());
        }
    }

    #[test]
    fn ase_round_trip_preserves_rgb_values() {
        let palette = Palette {
            colors: vec![
                Color::from_rgb_u8(200, 10, 40),
                Color::from_rgb_u8(12, 220, 100),
                Color::from_rgb_u8(77, 88, 210),
            ],
        };

        let ase = export_palette_bytes(&palette, ExportFormat::Ase, "")
            .expect("ASE export should produce bytes");
        let imported = import_ase(&ase).expect("exported ASE should be importable");

        assert_eq!(imported.colors.len(), palette.colors.len());
        for (left, right) in imported.colors.iter().zip(palette.colors.iter()) {
            assert_eq!(left.to_rgb_u8(), right.to_rgb_u8());
        }
    }
}
