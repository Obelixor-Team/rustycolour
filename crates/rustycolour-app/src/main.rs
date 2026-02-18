use eframe::{NativeOptions, egui};
use rustycolour_core::{
    Color, CvdMode, DeltaEMetric, ExportFormat, GenerationRequest, MethodParams, MethodRegistry,
    Palette, apca_contrast_lc, export_palette, import_ase, import_gpl, simulate_cvd,
    wcag_contrast_ratio,
};
use serde::{Deserialize, Serialize};
use std::fs;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1090.0, 550.0]),
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
    session_path: String,
    import_path: String,
    preset_path: String,
    preset_name_input: String,
    preset_selection_index: usize,
    presets: Vec<MethodPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSession {
    seed_hex: String,
    palette_size: usize,
    method_id: String,
    params: MethodParams,
    export_format_index: usize,
    export_css_prefix: String,
    swatches_hex: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MethodPreset {
    name: String,
    method_id: String,
    params: MethodParams,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PresetStore {
    presets: Vec<MethodPreset>,
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
            session_path: "rustycolour-session.json".to_owned(),
            import_path: "palette.gpl".to_owned(),
            preset_path: "rustycolour-presets.json".to_owned(),
            preset_name_input: String::new(),
            preset_selection_index: 0,
            presets: Vec::new(),
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
                egui::ComboBox::from_label("DeltaE metric")
                    .selected_text(match self.params.deltae_metric {
                        DeltaEMetric::E76 => "CIE76",
                        DeltaEMetric::E00 => "CIEDE2000",
                    })
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(
                                &mut self.params.deltae_metric,
                                DeltaEMetric::E76,
                                "CIE76",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut self.params.deltae_metric,
                                DeltaEMetric::E00,
                                "CIEDE2000",
                            )
                            .changed();
                    });
            }
            "ryb-complementary" => {
                ui.label("RYB wheel mix");
                changed |= ui
                    .add(egui::Slider::new(&mut self.params.ryb_mix, 0.0..=1.0))
                    .changed();
            }
            "cvd-safe-categorical" => {
                egui::ComboBox::from_label("CVD mode")
                    .selected_text(match self.params.cvd_mode {
                        CvdMode::Deuteranopia => "Deuteranopia",
                        CvdMode::Protanopia => "Protanopia",
                        CvdMode::Tritanopia => "Tritanopia",
                    })
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(
                                &mut self.params.cvd_mode,
                                CvdMode::Deuteranopia,
                                "Deuteranopia",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut self.params.cvd_mode,
                                CvdMode::Protanopia,
                                "Protanopia",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut self.params.cvd_mode,
                                CvdMode::Tritanopia,
                                "Tritanopia",
                            )
                            .changed();
                    });
                ui.label("CVD severity");
                changed |= ui
                    .add(egui::Slider::new(&mut self.params.cvd_severity, 0.0..=1.0))
                    .changed();
            }
            "annealed-deltae" => {
                ui.label("Target DeltaE (Lab)");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.deltae_target,
                        5.0..=60.0,
                    ))
                    .changed();
                egui::ComboBox::from_label("DeltaE metric")
                    .selected_text(match self.params.deltae_metric {
                        DeltaEMetric::E76 => "CIE76",
                        DeltaEMetric::E00 => "CIEDE2000",
                    })
                    .show_ui(ui, |ui| {
                        changed |= ui
                            .selectable_value(
                                &mut self.params.deltae_metric,
                                DeltaEMetric::E76,
                                "CIE76",
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut self.params.deltae_metric,
                                DeltaEMetric::E00,
                                "CIEDE2000",
                            )
                            .changed();
                    });
                ui.label("Anneal iterations");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.anneal_iterations,
                        20..=1200,
                    ))
                    .changed();
                ui.label("Anneal temperature");
                changed |= ui
                    .add(egui::Slider::new(
                        &mut self.params.anneal_temperature,
                        0.1..=3.0,
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
                .desired_width(ui.available_width()),
        );
    }

    fn import_gpl_from_path(&mut self) {
        let content = match fs::read_to_string(&self.import_path) {
            Ok(content) => content,
            Err(error) => {
                self.status = format!("Failed to read GPL file: {error}");
                return;
            }
        };

        let palette = match import_gpl(&content) {
            Ok(palette) => palette,
            Err(error) => {
                self.status = format!("Failed to parse GPL: {error}");
                return;
            }
        };

        let imported_len = palette.colors.len();
        if let Some(first) = palette.colors.first() {
            self.seed_hex = first.to_hex_rgb();
        }
        self.palette_size = imported_len.clamp(2, 24);
        self.palette = palette;
        self.contrast_fg_index = 0;
        self.contrast_bg_index = 1.min(self.palette.colors.len().saturating_sub(1));
        self.status = format!("Imported {imported_len} colors from {}", self.import_path);
    }

    fn import_ase_from_path(&mut self) {
        let bytes = match fs::read(&self.import_path) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.status = format!("Failed to read ASE file: {error}");
                return;
            }
        };

        let palette = match import_ase(&bytes) {
            Ok(palette) => palette,
            Err(error) => {
                self.status = format!("Failed to parse ASE: {error}");
                return;
            }
        };

        let imported_len = palette.colors.len();
        if let Some(first) = palette.colors.first() {
            self.seed_hex = first.to_hex_rgb();
        }
        self.palette_size = imported_len.clamp(2, 24);
        self.palette = palette;
        self.contrast_fg_index = 0;
        self.contrast_bg_index = 1.min(self.palette.colors.len().saturating_sub(1));
        self.status = format!("Imported {imported_len} colors from {}", self.import_path);
    }

    fn show_import_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Import");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("GPL path");
            ui.text_edit_singleline(&mut self.import_path);
        });
        if ui.button("Import GPL").clicked() {
            self.import_gpl_from_path();
        }
        if ui.button("Import ASE").clicked() {
            self.import_ase_from_path();
        }
    }

    fn save_presets_to_disk(&mut self) {
        let store = PresetStore {
            presets: self.presets.clone(),
        };
        match serde_json::to_string_pretty(&store) {
            Ok(json) => match fs::write(&self.preset_path, json) {
                Ok(()) => {
                    self.status = format!("Presets saved to {}", self.preset_path);
                }
                Err(error) => {
                    self.status = format!("Failed to save presets: {error}");
                }
            },
            Err(error) => {
                self.status = format!("Failed to serialize presets: {error}");
            }
        }
    }

    fn load_presets_from_disk(&mut self) {
        let content = match fs::read_to_string(&self.preset_path) {
            Ok(content) => content,
            Err(error) => {
                self.status = format!("Failed to read presets: {error}");
                return;
            }
        };

        let store: PresetStore = match serde_json::from_str(&content) {
            Ok(store) => store,
            Err(error) => {
                self.status = format!("Invalid preset JSON: {error}");
                return;
            }
        };

        self.presets = store.presets;
        self.preset_selection_index = 0;
        self.status = format!("Loaded {} presets", self.presets.len());
    }

    fn show_preset_panel(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        ui.heading("Presets");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("File");
            ui.text_edit_singleline(&mut self.preset_path);
        });
        ui.horizontal(|ui| {
            if ui.button("Save Preset File").clicked() {
                self.save_presets_to_disk();
            }
            if ui.button("Load Preset File").clicked() {
                self.load_presets_from_disk();
            }
        });
        ui.separator();

        let Some(method_id) = self.selected_method().map(|method| method.id().to_owned()) else {
            ui.label("Select a method to manage presets.");
            return changed;
        };

        let filtered_indices = self
            .presets
            .iter()
            .enumerate()
            .filter_map(|(index, preset)| (preset.method_id == method_id).then_some(index))
            .collect::<Vec<_>>();

        if self.preset_selection_index >= filtered_indices.len() {
            self.preset_selection_index = 0;
        }

        let selected_label = filtered_indices
            .get(self.preset_selection_index)
            .map(|idx| self.presets[*idx].name.clone())
            .unwrap_or_else(|| "No presets".to_owned());

        egui::ComboBox::from_label("Method presets")
            .selected_text(selected_label)
            .show_ui(ui, |ui| {
                for (pos, idx) in filtered_indices.iter().enumerate() {
                    ui.selectable_value(
                        &mut self.preset_selection_index,
                        pos,
                        &self.presets[*idx].name,
                    );
                }
            });

        ui.horizontal(|ui| {
            if ui.button("Load Selected").clicked() {
                if let Some(idx) = filtered_indices.get(self.preset_selection_index) {
                    self.params = self.presets[*idx].params;
                    self.status = format!("Loaded preset '{}'", self.presets[*idx].name);
                    changed = true;
                }
            }
            if ui.button("Delete Selected").clicked() {
                if let Some(idx) = filtered_indices.get(self.preset_selection_index) {
                    let removed = self.presets.remove(*idx);
                    self.preset_selection_index = 0;
                    self.status = format!("Deleted preset '{}'", removed.name);
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut self.preset_name_input);
        });
        if ui.button("Save Current As Preset").clicked() {
            let name = self.preset_name_input.trim();
            if name.is_empty() {
                self.status = "Preset name cannot be empty".to_owned();
            } else if let Some(existing_index) = self
                .presets
                .iter()
                .position(|preset| preset.method_id == method_id && preset.name == name)
            {
                self.presets[existing_index].params = self.params;
                self.status = format!("Updated preset '{}'", name);
            } else {
                self.presets.push(MethodPreset {
                    name: name.to_owned(),
                    method_id,
                    params: self.params,
                });
                self.status = format!("Saved preset '{}'", name);
            }
        }

        changed
    }

    fn save_session(&mut self) {
        let method_id = self
            .selected_method()
            .map_or_else(String::new, |method| method.id().to_owned());
        let session = AppSession {
            seed_hex: self.seed_hex.clone(),
            palette_size: self.palette_size,
            method_id,
            params: self.params,
            export_format_index: self.export_format_index,
            export_css_prefix: self.export_css_prefix.clone(),
            swatches_hex: self.palette.colors.iter().map(|c| c.to_hex_rgb()).collect(),
        };

        match serde_json::to_string_pretty(&session) {
            Ok(json) => match fs::write(&self.session_path, json) {
                Ok(()) => {
                    self.status = format!("Session saved to {}", self.session_path);
                }
                Err(error) => {
                    self.status = format!("Failed to save session: {error}");
                }
            },
            Err(error) => {
                self.status = format!("Failed to serialize session: {error}");
            }
        }
    }

    fn load_session(&mut self) {
        let content = match fs::read_to_string(&self.session_path) {
            Ok(content) => content,
            Err(error) => {
                self.status = format!("Failed to read session: {error}");
                return;
            }
        };

        let session: AppSession = match serde_json::from_str(&content) {
            Ok(session) => session,
            Err(error) => {
                self.status = format!("Invalid session JSON: {error}");
                return;
            }
        };

        self.seed_hex = session.seed_hex;
        self.palette_size = session.palette_size;
        self.params = session.params;
        self.export_format_index = session.export_format_index.min(ExportFormat::ALL.len() - 1);
        self.export_css_prefix = session.export_css_prefix;

        if let Some(index) = self
            .registry
            .methods()
            .iter()
            .position(|method| method.id() == session.method_id)
        {
            self.selected_method_index = index;
        }

        let mut loaded = Vec::with_capacity(session.swatches_hex.len());
        for swatch in &session.swatches_hex {
            if let Some(color) = Color::from_hex_rgb(swatch) {
                loaded.push(color);
            }
        }

        if loaded.is_empty() {
            self.generate_palette();
            self.status = format!("Session loaded from {} (regenerated)", self.session_path);
        } else {
            self.palette = Palette { colors: loaded };
            self.contrast_fg_index = 0;
            self.contrast_bg_index = 1.min(self.palette.colors.len().saturating_sub(1));
            self.status = format!("Session loaded from {}", self.session_path);
        }
    }

    fn show_session_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Session");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Path");
            ui.text_edit_singleline(&mut self.session_path);
        });
        ui.horizontal(|ui| {
            if ui.button("Save Session").clicked() {
                self.save_session();
            }
            if ui.button("Load Session").clicked() {
                self.load_session();
            }
        });
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
        let sim_fg = simulate_cvd(fg, self.params.cvd_mode, self.params.cvd_severity);
        let sim_bg = simulate_cvd(bg, self.params.cvd_mode, self.params.cvd_severity);
        let sim_wcag = wcag_contrast_ratio(sim_fg, sim_bg);
        let sim_apca = apca_contrast_lc(sim_fg, sim_bg);

        ui.monospace(format!("FG: {}", fg.to_hex_rgb()));
        ui.monospace(format!("BG: {}", bg.to_hex_rgb()));
        ui.label(format!("WCAG ratio: {wcag:.2}:1"));
        ui.label(format!("APCA Lc: {apca:.1}"));

        let wcag_aa = if wcag >= 4.5 { "PASS" } else { "FAIL" };
        let wcag_large = if wcag >= 3.0 { "PASS" } else { "FAIL" };
        let apca_body = if apca.abs() >= 60.0 { "PASS" } else { "FAIL" };
        let sim_wcag_aa = if sim_wcag >= 4.5 { "PASS" } else { "FAIL" };
        let sim_apca_body = if sim_apca.abs() >= 60.0 {
            "PASS"
        } else {
            "FAIL"
        };

        ui.label(format!("WCAG AA normal (>=4.5): {wcag_aa}"));
        ui.label(format!("WCAG AA large (>=3.0): {wcag_large}"));
        ui.label(format!("APCA body text (|Lc|>=60): {apca_body}"));
        ui.separator();
        ui.label("CVD simulation");
        egui::ComboBox::from_label("Mode")
            .selected_text(match self.params.cvd_mode {
                CvdMode::Deuteranopia => "Deuteranopia",
                CvdMode::Protanopia => "Protanopia",
                CvdMode::Tritanopia => "Tritanopia",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.params.cvd_mode,
                    CvdMode::Deuteranopia,
                    "Deuteranopia",
                );
                ui.selectable_value(&mut self.params.cvd_mode, CvdMode::Protanopia, "Protanopia");
                ui.selectable_value(&mut self.params.cvd_mode, CvdMode::Tritanopia, "Tritanopia");
            });
        ui.add(egui::Slider::new(&mut self.params.cvd_severity, 0.0..=1.0).text("Severity"));
        ui.label(format!("Simulated WCAG ratio: {sim_wcag:.2}:1"));
        ui.label(format!("Simulated APCA Lc: {sim_apca:.1}"));
        ui.label(format!("Simulated WCAG AA normal: {sim_wcag_aa}"));
        ui.label(format!("Simulated APCA body text: {sim_apca_body}"));
    }
}

impl eframe::App for RustyColourApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut controls_changed = self.palette.colors.is_empty();
        let mut manual_generate = false;

        egui::SidePanel::left("method_panel")
            .resizable(true)
            .default_width(205.0)
            .min_width(165.0)
            .max_width(290.0)
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
                ui.separator();
                controls_changed |= self.show_preset_panel(ui);
            });

        egui::SidePanel::right("tools_panel")
            .resizable(true)
            .default_width(230.0)
            .min_width(180.0)
            .max_width(330.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.show_session_panel(ui);
                    ui.separator();
                    self.show_import_panel(ui);
                    ui.separator();
                    self.show_export_panel(ui);
                    ui.separator();
                    self.show_contrast_panel(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
            ui.heading("rustycolour");
            ui.label("Colour theory tool for developers");
            ui.separator();

            ui.horizontal_wrapped(|ui| {
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

#[cfg(test)]
mod tests {
    use super::{AppSession, MethodPreset, PresetStore};
    use rustycolour_core::{CvdMode, DeltaEMetric, MethodParams};

    #[test]
    fn app_session_json_round_trip_preserves_key_fields() {
        let session = AppSession {
            seed_hex: "#112233".to_owned(),
            palette_size: 7,
            method_id: "annealed-deltae".to_owned(),
            params: MethodParams {
                deltae_metric: DeltaEMetric::E00,
                cvd_mode: CvdMode::Tritanopia,
                cvd_severity: 0.7,
                ..MethodParams::default()
            },
            export_format_index: 3,
            export_css_prefix: "brand".to_owned(),
            swatches_hex: vec!["#112233".to_owned(), "#445566".to_owned()],
        };

        let json = serde_json::to_string(&session).expect("session should serialize");
        let decoded: AppSession = serde_json::from_str(&json).expect("session should deserialize");

        assert_eq!(decoded.seed_hex, session.seed_hex);
        assert_eq!(decoded.palette_size, session.palette_size);
        assert_eq!(decoded.method_id, session.method_id);
        assert_eq!(decoded.params.deltae_metric, DeltaEMetric::E00);
        assert_eq!(decoded.params.cvd_mode, CvdMode::Tritanopia);
        assert_eq!(decoded.swatches_hex, session.swatches_hex);
    }

    #[test]
    fn preset_store_json_round_trip_preserves_presets() {
        let store = PresetStore {
            presets: vec![
                MethodPreset {
                    name: "Soft triadic".to_owned(),
                    method_id: "triadic".to_owned(),
                    params: MethodParams::default(),
                },
                MethodPreset {
                    name: "Strict CVD".to_owned(),
                    method_id: "cvd-safe-categorical".to_owned(),
                    params: MethodParams {
                        cvd_mode: CvdMode::Deuteranopia,
                        cvd_severity: 1.0,
                        ..MethodParams::default()
                    },
                },
            ],
        };

        let json = serde_json::to_string(&store).expect("preset store should serialize");
        let decoded: PresetStore =
            serde_json::from_str(&json).expect("preset store should deserialize");

        assert_eq!(decoded.presets.len(), 2);
        assert_eq!(decoded.presets[0].name, "Soft triadic");
        assert_eq!(decoded.presets[1].method_id, "cvd-safe-categorical");
        assert_eq!(decoded.presets[1].params.cvd_mode, CvdMode::Deuteranopia);
    }
}
