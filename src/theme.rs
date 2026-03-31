//! Colour theme derived from the ilseon / myeon palette.
//!
//! Palette:
//!   MutedTeal   #5A9B80 — accent / focus
//!   QuietAmber  #C08A3E — medium priority / secondary accent
//!   MutedRed    #B35F5F — errors / urgency
//!   DarkGrey    #121212 — OLED-friendly deep background

use eframe::egui;

// ── Palette constants ────────────────────────────────────────────────────────

pub const DARK_BG: egui::Color32 = egui::Color32::from_rgb(0x12, 0x12, 0x12);
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0x1A, 0x1A, 0x1A);
pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(0x22, 0x22, 0x22);
pub const SURFACE_HOVER: egui::Color32 = egui::Color32::from_rgb(0x2C, 0x2C, 0x2C);

pub const MUTED_TEAL: egui::Color32 = egui::Color32::from_rgb(0x5A, 0x9B, 0x80);
pub const QUIET_AMBER: egui::Color32 = egui::Color32::from_rgb(0xC0, 0x8A, 0x3E);
pub const MUTED_RED: egui::Color32 = egui::Color32::from_rgb(0xB3, 0x5F, 0x5F);

pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(0xE0, 0xE0, 0xE0);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x90, 0x90, 0x90);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(0x60, 0x60, 0x60);

// ── Apply theme ──────────────────────────────────────────────────────────────

/// Overwrite the egui visuals to match the ilseon palette.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Overall background
    visuals.panel_fill = PANEL_BG;
    visuals.window_fill = PANEL_BG;
    visuals.extreme_bg_color = DARK_BG;
    visuals.faint_bg_color = SURFACE;

    // Widget base colours
    let corner = egui::CornerRadius::same(4);

    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.noninteractive.corner_radius = corner;

    visuals.widgets.inactive.bg_fill = SURFACE;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = corner;

    visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, MUTED_TEAL);
    visuals.widgets.hovered.corner_radius = corner;

    visuals.widgets.active.bg_fill = MUTED_TEAL;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, DARK_BG);
    visuals.widgets.active.corner_radius = corner;

    visuals.widgets.open.bg_fill = SURFACE_HOVER;
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, MUTED_TEAL);
    visuals.widgets.open.corner_radius = corner;

    // Selection highlight — teal accent
    visuals.selection.bg_fill = MUTED_TEAL.gamma_multiply(0.35);
    visuals.selection.stroke = egui::Stroke::new(1.0, MUTED_TEAL);

    // Hyperlinks
    visuals.hyperlink_color = MUTED_TEAL;

    // Window shadow (subtle)
    visuals.window_shadow = egui::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };

    // Separator
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, TEXT_MUTED);

    ctx.set_visuals(visuals);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_dark_bg_matches_spec() {
        assert_eq!(DARK_BG, egui::Color32::from_rgb(0x12, 0x12, 0x12));
    }

    #[test]
    fn palette_muted_teal_matches_spec() {
        assert_eq!(MUTED_TEAL, egui::Color32::from_rgb(0x5A, 0x9B, 0x80));
    }

    #[test]
    fn palette_quiet_amber_matches_spec() {
        assert_eq!(QUIET_AMBER, egui::Color32::from_rgb(0xC0, 0x8A, 0x3E));
    }

    #[test]
    fn palette_muted_red_matches_spec() {
        assert_eq!(MUTED_RED, egui::Color32::from_rgb(0xB3, 0x5F, 0x5F));
    }

    #[test]
    fn surface_tones_are_ordered() {
        // DARK_BG < PANEL_BG < SURFACE < SURFACE_HOVER (by lightness)
        assert!(DARK_BG.r() < PANEL_BG.r());
        assert!(PANEL_BG.r() < SURFACE.r());
        assert!(SURFACE.r() < SURFACE_HOVER.r());
    }

    #[test]
    fn text_tiers_are_ordered() {
        // TEXT_MUTED < TEXT_SECONDARY < TEXT_PRIMARY
        assert!(TEXT_MUTED.r() < TEXT_SECONDARY.r());
        assert!(TEXT_SECONDARY.r() < TEXT_PRIMARY.r());
    }
}
