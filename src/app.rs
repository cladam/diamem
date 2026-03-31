use crate::dsl::dsl_to_mermaid;
use crate::render;
use crate::theme;
use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct DiamemApp {
    /// Raw DSL text entered by the user.
    dsl_source: String,
    /// Export directory path.
    export_path: String,
    /// Last status message shown in the footer.
    #[serde(skip)]
    status_message: String,
    /// Cached Mermaid output (recomputed when DSL changes).
    #[serde(skip)]
    mermaid_output: String,
    /// Cached SVG string rendered from Mermaid.
    #[serde(skip)]
    svg_output: String,
    /// Previous DSL source used for change detection.
    #[serde(skip)]
    prev_dsl_source: String,
    /// Whether the DSL currently parses without errors.
    #[serde(skip)]
    dsl_valid: bool,
    /// Whether the SVG rendered successfully.
    #[serde(skip)]
    svg_valid: bool,
    /// Error message from the SVG renderer (if any).
    #[serde(skip)]
    svg_error: String,
    /// Cached egui texture for the rendered SVG.
    #[serde(skip)]
    diagram_texture: Option<egui::TextureHandle>,
    /// Whether the theme has been applied.
    #[serde(skip)]
    theme_applied: bool,
}

impl Default for DiamemApp {
    fn default() -> Self {
        Self {
            dsl_source: "# Type your diagram DSL here\n\
                          # Examples:\n\
                          #   A -> B           (connection)\n\
                          #   A -> B -> C      (chain)\n\
                          #   A -(sends)> B    (labeled)\n\
                          #   A -[sends]-> B   (labeled, classic)\n\
                          #   User > App : Req (sequence)\n\
                          #   @ Group: A, B    (grouping)\n\
                          #   [Group] { A, B } (grouping, classic)\n\
                          \n\
                          Start -> Process -> Check -> Done\n"
                .to_string(),
            export_path: "~/Desktop".to_string(),
            status_message: String::new(),
            mermaid_output: String::new(),
            svg_output: String::new(),
            prev_dsl_source: String::new(),
            dsl_valid: true,
            svg_valid: false,
            svg_error: String::new(),
            diagram_texture: None,
            theme_applied: false,
        }
    }
}

impl DiamemApp {
    /// Create the app, restoring persisted state when available.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Re-render the diagram if the DSL source changed.
    fn update_diagram(&mut self, ctx: &egui::Context) {
        if self.dsl_source == self.prev_dsl_source {
            return;
        }
        self.prev_dsl_source = self.dsl_source.clone();

        // Step 1: DSL → Mermaid
        match dsl_to_mermaid(&self.dsl_source) {
            Ok(mermaid) => {
                self.mermaid_output = mermaid;
                self.dsl_valid = true;
            }
            Err(err) => {
                self.mermaid_output = err;
                self.dsl_valid = false;
                self.svg_valid = false;
                self.svg_error = "DSL parse error".into();
                self.diagram_texture = None;
                return;
            }
        }

        // Step 2: Mermaid → SVG
        match render::mermaid_to_svg(&self.mermaid_output) {
            Ok(svg) => {
                self.svg_output = svg;
                self.svg_valid = true;
                self.svg_error.clear();
                // Step 3: SVG → egui texture
                self.rasterize_svg(ctx);
            }
            Err(err) => {
                self.svg_valid = false;
                self.svg_error = err;
                self.diagram_texture = None;
            }
        }
    }

    /// Rasterize the current SVG into an egui texture for display.
    fn rasterize_svg(&mut self, ctx: &egui::Context) {
        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();

        let tree = match usvg::Tree::from_str(&self.svg_output, &opt) {
            Ok(t) => t,
            Err(e) => {
                self.svg_error = format!("SVG parse: {e}");
                self.diagram_texture = None;
                return;
            }
        };

        let size = tree.size().to_int_size();
        let Some(mut pixmap) = resvg::tiny_skia::Pixmap::new(size.width(), size.height()) else {
            self.svg_error = "Failed to allocate pixmap".into();
            self.diagram_texture = None;
            return;
        };

        // Fill with dark background
        pixmap.fill(resvg::tiny_skia::Color::from_rgba8(0x12, 0x12, 0x12, 0xFF));
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );

        let image = egui::ColorImage::from_rgba_unmultiplied(
            [size.width() as usize, size.height() as usize],
            pixmap.data(),
        );

        self.diagram_texture =
            Some(ctx.load_texture("diagram_preview", image, egui::TextureOptions::LINEAR));
    }

    /// Export the current diagram as a PNG file to the configured path.
    fn export_png(&mut self) {
        if !self.dsl_valid || !self.svg_valid {
            self.status_message = "✗ Cannot export — DSL or render has errors".into();
            return;
        }

        let dir = expand_tilde(&self.export_path);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            self.status_message = format!("✗ Cannot create directory: {e}");
            return;
        }

        let filename = format!(
            "diagram_{}.png",
            chrono::Local::now().format("%Y-%m-%d_%H%M")
        );
        let path = std::path::Path::new(&dir).join(&filename);

        match render::mermaid_to_png(&self.mermaid_output, &path) {
            Ok(()) => {
                self.status_message = format!("✓ Exported to {}", path.display());
            }
            Err(e) => {
                self.status_message = format!("✗ Export failed: {e}");
            }
        }
    }
}

/// Expand `~` at the start of a path to the user's home directory.
fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs_home() {
            return format!("{}/{rest}", home);
        }
    }
    path.to_string()
}

fn dirs_home() -> Option<String> {
    std::env::var("HOME").ok()
}

impl eframe::App for DiamemApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply ilseon theme once
        if !self.theme_applied {
            theme::apply(ctx);
            self.theme_applied = true;
        }

        // Re-render diagram only when DSL changes
        self.update_diagram(ctx);

        // Keyboard shortcuts
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.export_png();
        }

        // --- Top Menu Bar ---
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.label(
                    egui::RichText::new("◆ diamem")
                        .strong()
                        .color(theme::MUTED_TEAL),
                );
                ui.separator();
                ui.menu_button("File", |ui| {
                    if ui.button("Export PNG (Ctrl+S)").clicked() {
                        self.export_png();
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.status_message = "diamem v0.1.0 — Live Diagram Editor".into();
                        ui.close_menu();
                    }
                });
            });
        });

        // --- Export Footer (Bottom Panel) ---
        egui::TopBottomPanel::bottom("export_footer")
            .min_height(60.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::SURFACE)
                    .inner_margin(egui::Margin::same(8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Status indicator
                    let (status_text, status_color) = if self.dsl_valid && self.svg_valid {
                        ("✓ Diagram OK", theme::MUTED_TEAL)
                    } else if self.dsl_valid {
                        ("⚠ Render error", theme::QUIET_AMBER)
                    } else {
                        ("✗ DSL Errors", theme::MUTED_RED)
                    };
                    ui.colored_label(status_color, status_text);

                    ui.separator();

                    // Export path
                    ui.label(egui::RichText::new("Path:").color(theme::TEXT_SECONDARY));
                    ui.add(egui::TextEdit::singleline(&mut self.export_path).desired_width(180.0));

                    ui.separator();

                    // The big export button — Quiet Amber accent
                    let export_btn = egui::Button::new(
                        egui::RichText::new("⬆ Commit to shotext")
                            .size(16.0)
                            .strong()
                            .color(theme::DARK_BG),
                    )
                    .fill(theme::QUIET_AMBER)
                    .corner_radius(egui::CornerRadius::same(6));

                    if ui.add(export_btn).clicked() {
                        self.export_png();
                    }
                });

                // Status message
                if !self.status_message.is_empty() {
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .small()
                            .color(theme::TEXT_MUTED),
                    );
                }
            });

        // --- Left Panel: "The Engine" (DSL Editor) ---
        egui::SidePanel::left("engine_panel")
            .default_width(500.0)
            .min_width(300.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(theme::DARK_BG)
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("✏ DSL Editor")
                        .heading()
                        .color(theme::MUTED_TEAL),
                );
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.dsl_source)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(30)
                                .lock_focus(true)
                                .code_editor(),
                        );
                    });
            });

        // --- Right Panel: "The Memory" (Preview) ---
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL_BG)
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("🔍 Diagram Preview")
                        .heading()
                        .color(theme::MUTED_TEAL),
                );
                ui.add_space(4.0);

                if let Some(texture) = &self.diagram_texture {
                    // Show the rendered diagram with scroll/zoom
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let size = texture.size_vec2();
                            // Fit to panel width, maintain aspect ratio
                            let available_width = ui.available_width();
                            let scale = (available_width / size.x).min(1.0);
                            let display_size = egui::vec2(size.x * scale, size.y * scale);
                            ui.image(egui::load::SizedTexture::new(texture.id(), display_size));
                        });
                } else if !self.dsl_valid {
                    ui.colored_label(theme::MUTED_RED, "⚠ Parse errors:");
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(&self.mermaid_output)
                            .monospace()
                            .color(theme::MUTED_RED),
                    );
                } else if !self.svg_error.is_empty() {
                    ui.colored_label(theme::QUIET_AMBER, "⚠ Render error:");
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(&self.svg_error)
                            .monospace()
                            .color(theme::QUIET_AMBER),
                    );
                    // Fallback: show Mermaid source
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Generated Mermaid syntax:")
                            .small()
                            .color(theme::TEXT_MUTED),
                    );
                    let mut preview = self.mermaid_output.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut preview)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false)
                            .code_editor(),
                    );
                } else {
                    ui.colored_label(theme::TEXT_MUTED, "Type DSL on the left to see a preview…");
                }
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_valid_dsl() {
        let app = DiamemApp::default();
        let result = dsl_to_mermaid(&app.dsl_source);
        assert!(result.is_ok(), "Default DSL should parse: {result:?}");
    }

    #[test]
    fn default_export_path_is_set() {
        let app = DiamemApp::default();
        assert!(!app.export_path.is_empty());
    }

    #[test]
    fn default_starts_valid() {
        let app = DiamemApp::default();
        assert!(app.dsl_valid);
    }

    #[test]
    fn default_theme_not_applied() {
        let app = DiamemApp::default();
        assert!(!app.theme_applied);
    }

    #[test]
    fn default_status_message_empty() {
        let app = DiamemApp::default();
        assert!(app.status_message.is_empty());
    }

    #[test]
    fn expand_tilde_expands_home() {
        let expanded = expand_tilde("~/Documents");
        assert!(!expanded.starts_with("~/"));
    }

    #[test]
    fn expand_tilde_leaves_absolute_unchanged() {
        let path = "/tmp/output";
        assert_eq!(expand_tilde(path), path);
    }
}
