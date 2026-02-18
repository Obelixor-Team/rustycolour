use serde_json::json;

use crate::palette::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    HexList,
    RgbList,
    HslList,
    OklchList,
    Gpl,
    CssVariables,
    Json,
}

impl ExportFormat {
    pub const ALL: [Self; 7] = [
        Self::HexList,
        Self::RgbList,
        Self::HslList,
        Self::OklchList,
        Self::Gpl,
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

#[cfg(test)]
mod tests {
    use super::{ExportFormat, export_palette};
    use crate::{import::import_gpl, palette::Color, palette::Palette};

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
}
