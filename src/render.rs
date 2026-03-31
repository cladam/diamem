//! Rendering pipeline: Mermaid syntax → SVG → PNG.
//!
//! Wraps `mermaid-rs-renderer` with a dark theme matching the ilseon palette,
//! and provides helpers for both in-app SVG preview and PNG file export.

use std::path::Path;

use mermaid_rs_renderer::{RenderConfig, RenderOptions, Theme, render_with_options};

/// Render Mermaid syntax to an SVG string using our dark theme.
pub fn mermaid_to_svg(mermaid: &str) -> Result<String, String> {
    let options = dark_render_options();
    render_with_options(mermaid, options).map_err(|e| format!("{e}"))
}

/// Render Mermaid syntax to a PNG file on disk.
pub fn mermaid_to_png(mermaid: &str, output_path: &Path) -> Result<(), String> {
    let options = dark_render_options();
    let svg = render_with_options(mermaid, options).map_err(|e| format!("SVG render: {e}"))?;
    let theme = dark_theme();
    let cfg = RenderConfig::default();
    mermaid_rs_renderer::write_output_png(&svg, output_path, &cfg, &theme)
        .map_err(|e| format!("PNG export: {e}"))
}

/// Build render options with a dark theme inspired by the ilseon palette.
fn dark_render_options() -> RenderOptions {
    RenderOptions {
        theme: dark_theme(),
        ..RenderOptions::default()
    }
}

/// A dark theme derived from the ilseon / myeon colour palette.
fn dark_theme() -> Theme {
    let mut theme = Theme::modern();
    theme.background = "#121212".into();
    theme.primary_color = "#5A9B80".into();
    theme.primary_text_color = "#E0E0E0".into();
    theme.primary_border_color = "#5A9B80".into();
    theme.line_color = "#909090".into();
    theme.secondary_color = "#C08A3E".into();
    theme.tertiary_color = "#B35F5F".into();
    theme.text_color = "#E0E0E0".into();
    theme.edge_label_background = "#1A1A1A".into();
    theme.cluster_background = "#1A1A1A".into();
    theme.cluster_border = "#5A9B80".into();
    theme
}
