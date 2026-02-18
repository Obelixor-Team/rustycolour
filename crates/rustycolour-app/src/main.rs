use eframe::{NativeOptions, egui};
use rustycolour_core::{
    Color, ExportFormat, GenerationRequest, MethodParams, MethodRegistry, Palette,
    apca_contrast_lc, export_palette, wcag_contrast_ratio,
};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1260.0, 820.0]),
        ..Default::default()
    };
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
    export_format_index: usize,
    export_css_prefix: String,
    contrast_fg_index: usize,
    contrast_bg_index: usize,
}

impl Default for RustyColourApp {
    fn default() -> Self {
        Self {
            registry: MethodRegistry::with_builtins(),
            selected_method_index: 0,
            seed_hex: "#4F46E5".to_owned(),
            palette_size: 6,
            params: MethodParams::default(),
            palette: Palette::empty(),
            status: "Ready".to_owned(),
            export_format_index: 0,
            export_css_prefix: "rc".to_owned(),
            contrast_fg_index: 0,
            contrast_bg_index: 1,
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

    fn selected_export_format(&self) -> ExportFormat {
        ExportFormat::ALL
            .get(self.export_format_index)
            .copied()
            .unwrap_or(ExportFormat::HexList)
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
            self.contrast_fg_index = 0;
            self.contrast_bg_index = 1.min(self.palette.colors.len().saturating_sub(1));
            self.status = format!(
                "Generated {} colors via {}",
                self.palette.colors.len(),
                method_id
            );
        } else {
            self.status = "Generation failed.".to_owned();
        }
    }

    fn show_dynamic_params(&mut self, ui: &mut egui::Ui, method_id: &str) -> bool {
        let mut changed = false;
        ui.heading("Parameters");
        ui.separator();

        match method_id {
            "analogous" => {
                ui.label("Hue spread (degrees)");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.analogous_spread_deg,
                        20.0..=180.0,
                    ))
                    .changed();
            }
            "split-complementary" | "tetradic" => {
                ui.label("Split angle (degrees)");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.split_complement_deg,
                        10.0..=80.0,
                    ))
                    .changed();
            }
            "luminance-ramp" | "contrast-first" => {
                ui.label("Min luminance");
                changed |= ui
                    .add(egui::Slider::new(&mut self.params.luminance_min, 0.0..=1.0))
                    .changed();
                ui.label("Max luminance");
                changed |= ui
                    .add(egui::Slider::new(&mut self.params.luminance_max, 0.0..=1.0))
                    .changed();
            }
            "oklch-ramp" => {
                ui.label("Min luminance");
                changed |= ui
                    .add(egui::Slider::new(&mut self.params.luminance_min, 0.0..=1.0))
                    .changed();
                ui.label("Max luminance");
                changed |= ui
                    .add(egui::Slider::new(&mut self.params.luminance_max, 0.0..=1.0))
                    .changed();
                ui.label("Chroma scale");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.oklch_chroma_scale,
                        0.2..=2.0,
                    ))
                    .changed();
            }
            "lab-deltae-spaced" => {
                ui.label("Target DeltaE (Lab)");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.deltae_target,
                        5.0..=60.0,
                    ))
                    .changed();
            }
            "golden-angle" => {
                ui.label("Step (degrees)");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.golden_angle_step_deg,
                        10.0..=179.0,
                    ))
                    .changed();
            }
            "cubehelix" => {
                ui.label("Rotations");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.cubehelix_rotations,
                        -4.0..=4.0,
                    ))
                    .changed();
                ui.label("Hue strength");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.cubehelix_hue_strength,
                        0.0..=2.0,
                    ))
                    .changed();
            }
            _ => {
                ui.label("No extra parameters for this method.");
            }
        }

        changed
    }

    fn show_export_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Export");
        ui.separator();

        egui::ComboBox::from_label("Format")
            .selected_text(self.selected_export_format().label())
            .show_ui(ui, |ui| {
                for (index, format) in ExportFormat::ALL.iter().enumerate() {
                    ui.selectable_value(&mut self.export_format_index, index, format.label());
                }
            });

        if self.selected_export_format() == ExportFormat::CssVariables {
            ui.horizontal(|ui| {
                ui.label("CSS prefix");
                ui.text_edit_singleline(&mut self.export_css_prefix);
            });
        }

        let output = export_palette(
            &self.palette,
            self.selected_export_format(),
            &self.export_css_prefix,
        );
        if ui.button("Copy Export").clicked() {
            ui.ctx().copy_text(output.clone());
        }
        let mut preview = output;

        ui.add(
            egui::TextEdit::multiline(&mut preview)
                .font(egui::TextStyle::Monospace)
                .desired_rows(10)
                .desired_width(320.0),
        );
    }

    fn show_contrast_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Contrast");
        ui.separator();

        if self.palette.colors.len() < 2 {
            ui.label("Generate at least 2 colors to score contrast.");
            return;
        }

        let max_index = self.palette.colors.len() - 1;
        self.contrast_fg_index = self.contrast_fg_index.min(max_index);
        self.contrast_bg_index = self.contrast_bg_index.min(max_index);

        ui.label("Foreground index");
        ui.add(egui::Slider::new(
            &mut self.contrast_fg_index,
            0..=max_index,
        ));
        ui.label("Background index");
        ui.add(egui::Slider::new(
            &mut self.contrast_bg_index,
            0..=max_index,
        ));

        let fg = self.palette.colors[self.contrast_fg_index];
        let bg = self.palette.colors[self.contrast_bg_index];
        let wcag = wcag_contrast_ratio(fg, bg);
        let apca = apca_contrast_lc(fg, bg);

        ui.monospace(format!("FG: {}", fg.to_hex_rgb()));
        ui.monospace(format!("BG: {}", bg.to_hex_rgb()));
        ui.label(format!("WCAG ratio: {wcag:.2}:1"));
        ui.label(format!("APCA Lc (approx): {apca:.1}"));

        let wcag_aa = if wcag >= 4.5 { "PASS" } else { "FAIL" };
        let wcag_large = if wcag >= 3.0 { "PASS" } else { "FAIL" };
        let apca_body = if apca.abs() >= 60.0 { "PASS" } else { "FAIL" };

        ui.label(format!("WCAG AA normal (>=4.5): {wcag_aa}"));
        ui.label(format!("WCAG AA large (>=3.0): {wcag_large}"));
        ui.label(format!("APCA body text (|Lc|>=60): {apca_body}"));
    }
}

impl eframe::App for RustyColourApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut controls_changed = self.palette.colors.is_empty();
        let mut manual_generate = false;

        egui::SidePanel::left("method_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Methods");
                ui.separator();

                for (index, method) in self.registry.methods().iter().enumerate() {
                    let label = format!("{} ({:?})", method.name(), method.category());
                    controls_changed |= ui
                        .selectable_value(&mut self.selected_method_index, index, label)
                        .changed();
                }

                ui.separator();
                let selected_id = self
                    .selected_method()
                    .map_or_else(String::new, |method| method.id().to_owned());
                controls_changed |= self.show_dynamic_params(ui, &selected_id);
            });

        egui::SidePanel::right("tools_panel")
            .resizable(true)
            .show(ctx, |ui| {
                self.show_export_panel(ui);
                ui.separator();
                self.show_contrast_panel(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("rustycolour");
            ui.label("Colour theory tool for developers");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Seed HEX:");
                controls_changed |= ui.text_edit_singleline(&mut self.seed_hex).changed();

                ui.label("Size:");
                controls_changed |= ui
                    .add(egui::Slider::new(&mut self.palette_size, 2..=24))
                    .changed();

                if ui.button("Generate").clicked() {
                    manual_generate = true;
                }
            });

            ui.label(&self.status);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for color in &self.palette.colors {
                    let (r, g, b) = color.to_rgb_u8();
                    let swatch = egui::Color32::from_rgb(r, g, b);
                    let hex = color.to_hex_rgb();
                    let (h, s, l) = color.to_hsl();
                    let rgb = format!("rgb({r}, {g}, {b})");
                    let hsl = format!("hsl({h:.0}deg, {:.1}%, {:.1}%)", s * 100.0, l * 100.0);

                    ui.horizontal(|ui| {
                        let size = egui::vec2(190.0, 30.0);
                        let (rect, _response) =
                            ui.allocate_exact_size(size, egui::Sense::focusable_noninteractive());
                        ui.painter().rect_filled(rect, 6.0, swatch);

                        ui.monospace(&hex);
                        if ui.button("HEX").clicked() {
                            ui.ctx().copy_text(hex.clone());
                        }
                        if ui.button("RGB").clicked() {
                            ui.ctx().copy_text(rgb.clone());
                        }
                        if ui.button("HSL").clicked() {
                            ui.ctx().copy_text(hsl.clone());
                        }
                    });
                }
            });
        });

        if controls_changed || manual_generate {
            self.generate_palette();
        }
    }
}
