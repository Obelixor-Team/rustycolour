use eframe::{NativeOptions, egui};
use rustycolour_core::{Color, GenerationRequest, MethodRegistry, Palette};

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "rustycolour",
        options,
        Box::new(|_cc| Ok(Box::new(RustyColourApp::default()))),
    )
}

struct RustyColourApp {
    registry: MethodRegistry,
    selected_method_index: usize,
    seed_hex: String,
    palette_size: usize,
    palette: Palette,
    status: String,
}

impl Default for RustyColourApp {
    fn default() -> Self {
        let registry = MethodRegistry::with_builtins();
        let seed_hex = "#4F46E5".to_owned();
        let palette = Palette::empty();

        Self {
            registry,
            selected_method_index: 0,
            seed_hex,
            palette_size: 6,
            palette,
            status: "Ready".to_owned(),
        }
    }
}

impl RustyColourApp {
    fn selected_method_id(&self) -> Option<&'static str> {
        self.registry
            .methods()
            .get(self.selected_method_index)
            .map(|method| method.id())
    }

    fn generate_palette(&mut self) {
        let Some(seed) = Color::from_hex_rgb(&self.seed_hex) else {
            self.status = "Invalid HEX. Use #RRGGBB.".to_owned();
            return;
        };

        let Some(method_id) = self.selected_method_id() else {
            self.status = "No method selected.".to_owned();
            return;
        };

        let request = GenerationRequest {
            seed,
            size: self.palette_size,
        };

        if let Some(palette) = self.registry.generate_by_id(method_id, &request) {
            self.palette = palette;
            self.status = format!(
                "Generated {} colors via {method_id}",
                self.palette.colors.len()
            );
        } else {
            self.status = "Generation failed.".to_owned();
        }
    }
}

impl eframe::App for RustyColourApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("method_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Methods");
                ui.separator();

                for (index, method) in self.registry.methods().iter().enumerate() {
                    let label = format!("{} ({:?})", method.name(), method.category());
                    ui.selectable_value(&mut self.selected_method_index, index, label);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("rustycolour");
            ui.label("Colour theory toolkit scaffold");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Seed HEX:");
                ui.text_edit_singleline(&mut self.seed_hex);

                ui.label("Size:");
                ui.add(egui::Slider::new(&mut self.palette_size, 2..=16));

                if ui.button("Generate").clicked() {
                    self.generate_palette();
                }
            });

            ui.label(&self.status);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for color in &self.palette.colors {
                    let (r, g, b) = color.to_rgb_u8();
                    let swatch = egui::Color32::from_rgb(r, g, b);
                    let hex = color.to_hex_rgb();

                    ui.horizontal(|ui| {
                        let size = egui::vec2(180.0, 28.0);
                        let (rect, _response) =
                            ui.allocate_exact_size(size, egui::Sense::focusable_noninteractive());
                        ui.painter().rect_filled(rect, 4.0, swatch);

                        ui.monospace(&hex);
                        if ui.button("Copy").clicked() {
                            ui.ctx().copy_text(hex);
                        }
                    });
                }
            });
        });
    }
}
