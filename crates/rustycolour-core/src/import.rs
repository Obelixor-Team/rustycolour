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

pub fn import_ase(bytes: &[u8]) -> Result<Palette, String> {
    let mut reader = ByteReader::new(bytes);

    let signature = reader.read_exact(4)?;
    if signature != b"ASEF" {
        return Err("Invalid ASE header: expected 'ASEF'".to_owned());
    }

    let _major = reader.read_u16_be()?;
    let _minor = reader.read_u16_be()?;
    let block_count = reader.read_u32_be()?;

    let mut colors = Vec::new();

    for _ in 0..block_count {
        let block_type = reader.read_u16_be()?;
        let block_len = reader.read_u32_be()? as usize;
        let block_data = reader.read_exact(block_len)?;

        // 0x0001 = color entry; 0xC001/0xC002 group markers.
        if block_type == 0x0001 {
            if let Some(color) = parse_ase_color_block(block_data)? {
                colors.push(color);
            }
        }
    }

    if colors.is_empty() {
        return Err("No importable colors found in ASE file".to_owned());
    }

    Ok(Palette { colors })
}

fn parse_ase_color_block(data: &[u8]) -> Result<Option<Color>, String> {
    let mut reader = ByteReader::new(data);

    let name_len = reader.read_u16_be()? as usize;
    let _name = if name_len > 0 {
        let utf16 = (0..name_len)
            .map(|_| reader.read_u16_be())
            .collect::<Result<Vec<_>, _>>()?;
        String::from_utf16_lossy(&utf16)
    } else {
        String::new()
    };

    let model_bytes = reader.read_exact(4)?;
    let model = std::str::from_utf8(model_bytes)
        .map_err(|_| "Invalid ASE color model bytes".to_owned())?
        .trim();

    let color = match model {
        "RGB" => {
            let r = reader.read_f32_be()?.clamp(0.0, 1.0);
            let g = reader.read_f32_be()?.clamp(0.0, 1.0);
            let b = reader.read_f32_be()?.clamp(0.0, 1.0);
            Some(Color::from_rgba(r, g, b, 1.0))
        }
        "Gray" => {
            let v = reader.read_f32_be()?.clamp(0.0, 1.0);
            Some(Color::from_rgba(v, v, v, 1.0))
        }
        "CMYK" => {
            let c = reader.read_f32_be()?.clamp(0.0, 1.0);
            let m = reader.read_f32_be()?.clamp(0.0, 1.0);
            let y = reader.read_f32_be()?.clamp(0.0, 1.0);
            let k = reader.read_f32_be()?.clamp(0.0, 1.0);
            let r = (1.0 - c) * (1.0 - k);
            let g = (1.0 - m) * (1.0 - k);
            let b = (1.0 - y) * (1.0 - k);
            Some(Color::from_rgba(r, g, b, 1.0))
        }
        _ => None,
    };

    let _color_type = reader.read_u16_be()?;

    Ok(color)
}

struct ByteReader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> ByteReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], String> {
        if self.cursor + len > self.data.len() {
            return Err("Unexpected end of file".to_owned());
        }
        let start = self.cursor;
        self.cursor += len;
        Ok(&self.data[start..start + len])
    }

    fn read_u16_be(&mut self) -> Result<u16, String> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32_be(&mut self) -> Result<u32, String> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_f32_be(&mut self) -> Result<f32, String> {
        let bits = self.read_u32_be()?;
        Ok(f32::from_bits(bits))
    }
}

#[cfg(test)]
mod tests {
    use super::{import_ase, import_gpl};

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
    fn rejects_wrong_gpl_header() {
        let bad = "NOT A GPL\n255 0 0 Red\n";
        let err = import_gpl(bad).expect_err("invalid header should fail");
        assert!(err.contains("Invalid GPL header"));
    }

    #[test]
    fn rejects_empty_gpl_color_section() {
        let empty = "GIMP Palette\nName: Empty\n#\n";
        let err = import_gpl(empty).expect_err("empty palette should fail");
        assert!(err.contains("No colors found"));
    }

    #[test]
    fn parses_minimal_ase_rgb_color() {
        let mut ase = Vec::new();
        ase.extend_from_slice(b"ASEF");
        ase.extend_from_slice(&1u16.to_be_bytes());
        ase.extend_from_slice(&0u16.to_be_bytes());
        ase.extend_from_slice(&1u32.to_be_bytes());

        // Block: color entry
        ase.extend_from_slice(&0x0001u16.to_be_bytes());
        ase.extend_from_slice(&28u32.to_be_bytes());

        // Name length (UTF-16 code units, incl null terminator): 4 for "Red\0"
        ase.extend_from_slice(&4u16.to_be_bytes());
        ase.extend_from_slice(&0x0052u16.to_be_bytes()); // R
        ase.extend_from_slice(&0x0065u16.to_be_bytes()); // e
        ase.extend_from_slice(&0x0064u16.to_be_bytes()); // d
        ase.extend_from_slice(&0x0000u16.to_be_bytes());

        ase.extend_from_slice(b"RGB ");
        ase.extend_from_slice(&1.0f32.to_bits().to_be_bytes());
        ase.extend_from_slice(&0.0f32.to_bits().to_be_bytes());
        ase.extend_from_slice(&0.0f32.to_bits().to_be_bytes());
        ase.extend_from_slice(&0u16.to_be_bytes());

        let palette = import_ase(&ase).expect("valid ase should parse");
        assert_eq!(palette.colors.len(), 1);
        assert_eq!(palette.colors[0].to_hex_rgb(), "#FF0000");
    }

    #[test]
    fn rejects_invalid_ase_header() {
        let bad = b"NOPE";
        let err = import_ase(bad).expect_err("bad header should fail");
        assert!(err.contains("Invalid ASE header"));
    }
}
