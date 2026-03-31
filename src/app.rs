use crate::dsl::dsl_to_mermaid;
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
    /// Cached Mermaid output (recomputed every frame).
    #[serde(skip)]
    mermaid_output: String,
    /// Whether the DSL currently parses without errors.
    #[serde(skip)]
    dsl_valid: bool,
    /// Whether the theme has been applied.
    #[serde(skip)]
    theme_applied: bool,
}

impl Default for DiamemApp {
    fn default() -> Self {
        Self {
            dsl_source: "# Type your diagram DSL here\n\
                          # Examples:\n\
                          #   A -> B\n\
                          #   A -[sends]-> B\n\
                          #   User > App : Request\n\
                          #   [Group] { A, B }\n\
                          \n\
                          Start -> Process\n\
                          Process -[validate]-> Check\n\
                          Check -> Done\n"
                .to_string(),
            export_path: "~/Desktop".to_string(),
            status_message: String::new(),
            mermaid_output: String::new(),
            dsl_valid: true,
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
}

impl eframe::App for DiamemApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply ilseon theme once
        if !self.theme_applied {
            theme::apply(ctx);
            self.theme_applied = true;
        }

        // Re-parse DSL every frame
        match dsl_to_mermaid(&self.dsl_source) {
            Ok(mermaid) => {
                self.mermaid_output = mermaid;
                self.dsl_valid = true;
            }
            Err(err) => {
                self.mermaid_output = err;
                self.dsl_valid = false;
            }
        }

        // Keyboard shortcuts
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.status_message = "Export triggered (Ctrl/Cmd+S) — not yet implemented".into();
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
                        self.status_message = "Export triggered — not yet implemented".into();
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
                    let (status_text, status_color) = if self.dsl_valid {
                        ("✓ DSL Valid", theme::MUTED_TEAL)
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
                        self.status_message = "Export to Shotext — not yet implemented".into();
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
                    egui::RichText::new("Diagram Preview")
                        .heading()
                        .color(theme::MUTED_TEAL),
                );
                ui.add_space(4.0);

                // For Phase 1: show generated Mermaid syntax.
                // Phase 2 will replace this with actual SVG rendering.
                if self.dsl_valid {
                    ui.label(
                        egui::RichText::new("Generated Mermaid syntax:")
                            .small()
                            .color(theme::TEXT_MUTED),
                    );
                    ui.add_space(4.0);
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut preview = self.mermaid_output.clone();
                            ui.add(
                                egui::TextEdit::multiline(&mut preview)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false)
                                    .code_editor(),
                            );
                        });
                } else {
                    ui.colored_label(theme::MUTED_RED, "⚠ Parse errors:");
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(&self.mermaid_output)
                            .monospace()
                            .color(theme::MUTED_RED),
                    );
                }
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
