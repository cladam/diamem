//! Rendering pipeline: Mermaid syntax → SVG → PNG.
//!
//! Wraps `mermaid-rs-renderer` with a dark theme matching the ilseon palette,
//! and provides helpers for both in-app SVG preview and PNG file export.
//! Also supports injecting DSL comments as a visible footer in the SVG/PNG
//! so that OCR tools like Shotext can index them.

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

/// Render Mermaid syntax to a PNG file, including a visible comment footer.
///
/// The footer embeds DSL comments as OCR-readable text below the diagram so
/// that screenshot-indexing tools (e.g. Shotext) can search for them later.
pub fn mermaid_to_png_with_comments(
    mermaid: &str,
    comments: &[String],
    output_path: &Path,
) -> Result<(), String> {
    let options = dark_render_options();
    let svg = render_with_options(mermaid, options).map_err(|e| format!("SVG render: {e}"))?;
    let svg = inject_svg_footer(&svg, comments);
    svg_to_png(&svg, output_path)
}

// ── SVG comment footer ──────────────────────────────────────────────────────

/// Inject a visible text footer into an SVG string.
///
/// Each comment becomes a monospace line below the diagram, separated by a
/// thin rule.  The footer uses the ilseon dark-background palette so it
/// blends with the rest of the rendered diagram.
///
/// If `comments` is empty the SVG is returned unchanged.
pub fn inject_svg_footer(svg: &str, comments: &[String]) -> String {
    if comments.is_empty() {
        return svg.to_string();
    }

    let font_size: f64 = 16.0;
    let line_height: f64 = 24.0;
    let pad_top: f64 = 16.0;
    let pad_left: f64 = 20.0;
    let pad_right: f64 = 20.0;
    let pad_bottom: f64 = 16.0;
    let gap: f64 = 12.0; // space between diagram and footer
    let footer_h = gap + pad_top + (comments.len() as f64 * line_height) + pad_bottom;

    // Approximate character width for Helvetica/Arial at this font size.
    // 0.62 × font_size is a conservative average (caps + lowercase mix).
    let char_w: f64 = font_size * 0.62;

    // Parse the original viewBox – bail out unchanged if it's missing.
    let (vb_x, vb_y, vb_w, vb_h) = match parse_viewbox(svg) {
        Some(dims) => dims,
        None => return svg.to_string(),
    };

    // The footer may be wider than the diagram — pick the bigger value.
    let longest_text_w = comments
        .iter()
        .map(|c| pad_left + (c.len() as f64 * char_w) + pad_right)
        .fold(0.0_f64, f64::max);
    let new_vb_w = vb_w.max(longest_text_w);
    let new_vb_h = vb_h + footer_h;

    // Build footer SVG elements
    let mut footer = String::new();

    let footer_y = vb_y + vb_h + gap;
    let footer_content_h = footer_h - gap;

    // White background rect — high contrast for OCR, spans full width
    footer.push_str(&format!(
        "<rect x=\"{vb_x}\" y=\"{footer_y}\" width=\"{new_vb_w}\" \
         height=\"{footer_content_h}\" fill=\"#FFFFFF\" rx=\"4\"/>\n",
    ));

    // Thin top border to separate from diagram
    footer.push_str(&format!(
        "<line x1=\"{vb_x}\" y1=\"{footer_y}\" x2=\"{}\" y2=\"{footer_y}\" \
         stroke=\"#CCCCCC\" stroke-width=\"1\"/>\n",
        vb_x + new_vb_w,
    ));

    // One <text> per comment line — dark text on white for max OCR contrast
    for (i, line) in comments.iter().enumerate() {
        let y = footer_y + pad_top + (i as f64 * line_height) + font_size;
        let x = vb_x + pad_left;
        let escaped = xml_escape(line);
        footer.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" fill=\"#1A1A1A\" \
             font-family=\"Helvetica, Arial, sans-serif\" \
             font-size=\"{font_size}\">{escaped}</text>\n",
        ));
    }

    // Patch dimensions & splice in the footer
    let mut result = replace_viewbox(svg, vb_x, vb_y, new_vb_w, new_vb_h);
    result = replace_svg_attr(&result, "width", new_vb_w);
    result = replace_svg_attr(&result, "height", new_vb_h);
    result.replace("</svg>", &format!("{footer}</svg>"))
}

// ── Direct SVG → PNG rasterisation ──────────────────────────────────────────

/// Rasterize an SVG string and write a PNG file.
///
/// Uses `usvg` + `resvg` with the same dark background fill as the in-app
/// preview so the exported PNG looks identical.
pub fn svg_to_png(svg: &str, output_path: &Path) -> Result<(), String> {
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_str(svg, &opt).map_err(|e| format!("SVG parse: {e}"))?;
    let size = tree.size().to_int_size();

    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| "Failed to allocate pixmap".to_string())?;

    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(0x12, 0x12, 0x12, 0xFF));
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    pixmap
        .save_png(output_path)
        .map_err(|e| format!("PNG save: {e}"))
}

// ── Internal helpers ────────────────────────────────────────────────────────

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

/// Parse `viewBox="x y w h"` from the root `<svg>` element.
fn parse_viewbox(svg: &str) -> Option<(f64, f64, f64, f64)> {
    let needle = "viewBox=\"";
    let start = svg.find(needle)? + needle.len();
    let end = start + svg[start..].find('"')?;
    let parts: Vec<f64> = svg[start..end]
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if parts.len() == 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

/// Rewrite the `viewBox` attribute value in an SVG string.
fn replace_viewbox(svg: &str, x: f64, y: f64, w: f64, h: f64) -> String {
    let needle = "viewBox=\"";
    if let Some(attr_start) = svg.find(needle) {
        let val_start = attr_start + needle.len();
        if let Some(val_len) = svg[val_start..].find('"') {
            let val_end = val_start + val_len;
            return format!(
                "{}viewBox=\"{x} {y} {w} {h}\"{}",
                &svg[..attr_start],
                &svg[val_end + 1..],
            );
        }
    }
    svg.to_string()
}

/// Replace a numeric attribute on the root `<svg>` tag (e.g. `height`).
fn replace_svg_attr(svg: &str, attr: &str, new_value: f64) -> String {
    let needle = format!("{attr}=\"");
    // Only look inside the opening <svg …> tag.
    let tag_end = match svg.find('>') {
        Some(pos) => pos,
        None => return svg.to_string(),
    };
    let search = &svg[..tag_end];
    if let Some(attr_start) = search.find(&needle) {
        let val_start = attr_start + needle.len();
        if let Some(val_len) = svg[val_start..tag_end].find('"') {
            let val_end = val_start + val_len;
            return format!(
                "{}{needle}{new_value}\"{}",
                &svg[..attr_start],
                &svg[val_end + 1..],
            );
        }
    }
    svg.to_string()
}

/// Minimal XML escaping for text content.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── inject_svg_footer ───────────────────────────────────────────────

    const STUB_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100" width="200" height="100"><rect width="200" height="100" fill="#121212"/></svg>"##;

    #[test]
    fn footer_empty_comments_returns_unchanged() {
        let result = inject_svg_footer(STUB_SVG, &[]);
        assert_eq!(result, STUB_SVG);
    }

    #[test]
    fn footer_adds_text_elements() {
        let comments = vec!["hello world".to_string()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        assert!(result.contains("<text"));
        assert!(result.contains("hello world"));
    }

    #[test]
    fn footer_increases_viewbox_height() {
        let comments = vec!["line one".into(), "line two".into()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        // The original height was 100; it should now be larger.
        let (_, _, _, h) = parse_viewbox(&result).unwrap();
        assert!(h > 100.0);
    }

    #[test]
    fn footer_escapes_special_xml_chars() {
        let comments = vec!["A < B & C > D".to_string()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        assert!(result.contains("A &lt; B &amp; C &gt; D"));
    }

    #[test]
    fn footer_adds_separator_line() {
        let comments = vec!["note".into()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        assert!(result.contains("<line"));
    }

    #[test]
    fn footer_adds_background_rect() {
        let comments = vec!["note".into()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        // There should be a background rect for the footer area (in addition
        // to the diagram's own rect).
        assert!(result.matches("<rect").count() >= 2);
    }

    #[test]
    fn footer_uses_high_contrast_for_ocr() {
        let comments = vec!["searchable context".into()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        // White background for max OCR contrast
        assert!(result.contains("fill=\"#FFFFFF\""));
        // Dark text on the white background
        assert!(result.contains("fill=\"#1A1A1A\""));
        // Sans-serif font for clean glyph shapes
        assert!(result.contains("sans-serif"));
        // Reasonable font size (>= 16px)
        assert!(result.contains("font-size=\"16\""));
    }

    #[test]
    fn footer_widens_viewbox_for_long_text() {
        // STUB_SVG is 200px wide; a ~60-char comment should force it wider.
        let long = "This is a long comment that should definitely exceed 200px of width easily";
        let comments = vec![long.into()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        let (_, _, w, _) = parse_viewbox(&result).unwrap();
        assert!(
            w > 200.0,
            "Expected viewBox width > 200 for long text, got {w}"
        );
    }

    #[test]
    fn footer_keeps_width_when_diagram_is_wider() {
        // Short comment should not shrink the original 200px width.
        let comments = vec!["hi".into()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        let (_, _, w, _) = parse_viewbox(&result).unwrap();
        assert!(w >= 200.0, "Expected viewBox width >= 200, got {w}");
    }

    // ── parse_viewbox ───────────────────────────────────────────────────

    #[test]
    fn parse_viewbox_valid() {
        let svg = r#"<svg viewBox="-10 -5 800 600"></svg>"#;
        assert_eq!(parse_viewbox(svg), Some((-10.0, -5.0, 800.0, 600.0)));
    }

    #[test]
    fn parse_viewbox_missing_returns_none() {
        let svg = "<svg width=\"100\" height=\"100\"></svg>";
        assert_eq!(parse_viewbox(svg), None);
    }

    // ── xml_escape ──────────────────────────────────────────────────────

    #[test]
    fn xml_escape_special_chars() {
        assert_eq!(xml_escape("<>&\"'"), "&lt;&gt;&amp;&quot;&apos;");
    }
}
