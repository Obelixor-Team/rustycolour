use rustycolour_core::{Color, GenerationRequest, MethodRegistry};

fn main() {
    let registry = MethodRegistry::with_builtins();
    let request = GenerationRequest {
        seed: Color::from_rgb_u8(79, 70, 229),
        size: 6,
    };

    match registry.generate_by_id("golden-angle", &request) {
        Some(palette) => {
            for color in palette.colors {
                println!("{}", color.to_hex_rgb());
            }
        }
        None => {
            eprintln!("method not found");
            std::process::exit(1);
        }
    }
}
