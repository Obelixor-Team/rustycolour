use crate::palette::{Color, Palette};

pub fn import_gpl(content: &str) -> Result<Palette, String> {
    let mut lines = content.lines();
    let Some(first) = lines.next() else {
        return Err("GPL file is empty".to_owned());
    };

    if first.trim() != "GIMP Palette" {
        return Err("Invalid GPL header: expected 'GIMP Palette'".to_owned());
    }

    let mut colors = Vec::new();

    for raw in lines {
        let line = raw.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("Name:")
            || line.starts_with("Columns:")
        {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(r_str) = parts.next() else { continue };
        let Some(g_str) = parts.next() else { continue };
        let Some(b_str) = parts.next() else { continue };

        let r: u8 = r_str
            .parse::<u16>()
            .ok()
            .filter(|v| *v <= 255)
            .map(|v| v as u8)
            .ok_or_else(|| format!("Invalid red channel in line: {line}"))?;
        let g: u8 = g_str
            .parse::<u16>()
            .ok()
            .filter(|v| *v <= 255)
            .map(|v| v as u8)
            .ok_or_else(|| format!("Invalid green channel in line: {line}"))?;
        let b: u8 = b_str
            .parse::<u16>()
            .ok()
            .filter(|v| *v <= 255)
            .map(|v| v as u8)
            .ok_or_else(|| format!("Invalid blue channel in line: {line}"))?;

        colors.push(Color::from_rgb_u8(r, g, b));
    }

    if colors.is_empty() {
        return Err("No colors found in GPL file".to_owned());
    }

    Ok(Palette { colors })
}

#[cfg(test)]
mod tests {
    use super::import_gpl;

    #[test]
    fn parses_basic_gpl_palette() {
        let gpl = "GIMP Palette\nName: Test\n#\n255 0 0 Red\n0 255 0 Green\n0 0 255 Blue\n";
        let palette = import_gpl(gpl).expect("should parse valid gpl");
        assert_eq!(palette.colors.len(), 3);
        assert_eq!(palette.colors[0].to_hex_rgb(), "#FF0000");
        assert_eq!(palette.colors[1].to_hex_rgb(), "#00FF00");
        assert_eq!(palette.colors[2].to_hex_rgb(), "#0000FF");
    }

    #[test]
    fn rejects_wrong_header() {
        let bad = "NOT A GPL\n255 0 0 Red\n";
        let err = import_gpl(bad).expect_err("invalid header should fail");
        assert!(err.contains("Invalid GPL header"));
    }

    #[test]
    fn rejects_empty_color_section() {
        let empty = "GIMP Palette\nName: Empty\n#\n";
        let err = import_gpl(empty).expect_err("empty palette should fail");
        assert!(err.contains("No colors found"));
    }
}
