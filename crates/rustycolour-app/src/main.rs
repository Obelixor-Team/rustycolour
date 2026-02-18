use eframe::{NativeOptions, egui};
use rustycolour_core::{Color, GenerationRequest, MethodParams, MethodRegistry, Palette};

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
    params: MethodParams,
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
            params: MethodParams::default(),
            palette,
            status: "Ready".to_owned(),
        }
    }
}

impl RustyColourApp {
    fn selected_method(&self) -> Option<&dyn rustycolour_core::PaletteMethod> {
        self.registry
            .methods()
            .get(self.selected_method_index)
            .map(std::ops::Deref::deref)
    }

    fn generate_palette(&mut self) {
        let Some(seed) = Color::from_hex_rgb(&self.seed_hex) else {
            self.status = "Invalid HEX. Use #RRGGBB.".to_owned();
            return;
        };

        let Some(method_id) = self.selected_method().map(|method| method.id()) else {
            self.status = "No method selected.".to_owned();
            return;
        };

        let request = GenerationRequest {
            seed,
            size: self.palette_size,
            params: self.params,
        };

        if let Some(palette) = self.registry.generate_by_id(method_id, &request) {
            self.palette = palette;
            self.status = format!(
                "Generated {} colors via {}",
                self.palette.colors.len(),
                method_id
            );
        } else {
            self.status = "Generation failed.".to_owned();
        }
    }

    fn show_dynamic_params(&mut self, ui: &mut egui::Ui, method_id: &str) {
        ui.heading("Parameters");
        ui.separator();

        match method_id {
            "analogous" => {
                ui.label("Hue spread (degrees)");
                ui.add(egui::Slider::new(
                    &mut self.params.analogous_spread_deg,
                    20.0..=180.0,
                ));
            }
            "split-complementary" | "tetradic" => {
                ui.label("Split angle (degrees)");
                ui.add(egui::Slider::new(
                    &mut self.params.split_complement_deg,
                    10.0..=80.0,
                ));
            }
            "luminance-ramp" | "contrast-first" => {
                ui.label("Min luminance");
                ui.add(egui::Slider::new(&mut self.params.luminance_min, 0.0..=1.0));
                ui.label("Max luminance");
                ui.add(egui::Slider::new(&mut self.params.luminance_max, 0.0..=1.0));
            }
            "golden-angle" => {
                ui.label("Step (degrees)");
                ui.add(egui::Slider::new(
                    &mut self.params.golden_angle_step_deg,
                    10.0..=179.0,
                ));
            }
            "cubehelix" => {
                ui.label("Rotations");
                ui.add(egui::Slider::new(
                    &mut self.params.cubehelix_rotations,
                    -4.0..=4.0,
                ));
                ui.label("Hue strength");
                ui.add(egui::Slider::new(
                    &mut self.params.cubehelix_hue_strength,
                    0.0..=2.0,
                ));
            }
            _ => {
                ui.label("No extra parameters for this method.");
            }
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

                ui.separator();
                let selected_id = self.selected_method().map_or("", |method| method.id());
                self.show_dynamic_params(ui, selected_id);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("rustycolour");
            ui.label("Colour theory tool for developers");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Seed HEX:");
                ui.text_edit_singleline(&mut self.seed_hex);

                ui.label("Size:");
                ui.add(egui::Slider::new(&mut self.palette_size, 2..=24));

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
                        let size = egui::vec2(220.0, 30.0);
                        let (rect, _response) =
                            ui.allocate_exact_size(size, egui::Sense::focusable_noninteractive());
                        ui.painter().rect_filled(rect, 6.0, swatch);

                        ui.monospace(&hex);
                        if ui.button("Copy HEX").clicked() {
                            ui.ctx().copy_text(hex);
                        }
                    });
                }
            });
        });
    }
}
