//! Rendering pipeline: Mermaid syntax → SVG → PNG.
//!
//! Wraps `mermaid-rs-renderer` with a dark theme matching the ilseon palette,
//! and provides helpers for both in-app SVG preview and PNG file export.
//!
//! DSL comments are rendered as a **separate** high-contrast SVG and
//! composited below the diagram so OCR tools like Shotext can reliably
//! index them — even when the Mermaid SVG clips or restricts text rendering.

use std::path::Path;

use mermaid_rs_renderer::{LayoutConfig, RenderOptions, Theme, render_with_options};

// ── Public data types ───────────────────────────────────────────────────────

/// Rasterised diagram image (premultiplied RGBA, row-major).
pub struct RenderedDiagram {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel data — 4 bytes per pixel, premultiplied RGBA.
    pub rgba_data: Vec<u8>,
}

// ── Primary public API ──────────────────────────────────────────────────────

/// Render Mermaid syntax to an SVG string using our dark theme.
pub fn mermaid_to_svg(mermaid: &str) -> Result<String, String> {
    let is_mindmap = mermaid.starts_with("mindmap\n");
    let is_timeline = mermaid.starts_with("timeline\n");
    let options = if is_mindmap {
        dark_mindmap_render_options()
    } else {
        dark_render_options()
    };
    let svg = render_with_options(mermaid, options).map_err(|e| format!("{e}"))?;
    if is_timeline {
        Ok(recolor_timeline_svg(&svg))
    } else {
        Ok(svg)
    }
}

/// Render a Mermaid diagram (with optional comment footer) and return pixel data.
///
/// The footer is rendered as its own standalone SVG and composited below the
/// diagram, which avoids clip-path / font-resolution issues that can occur
/// when injecting `<text>` elements into the Mermaid SVG.
pub fn render_diagram(mermaid: &str, comments: &[String]) -> Result<RenderedDiagram, String> {
    let pm = render_composite(mermaid, comments)?;
    Ok(RenderedDiagram {
        width: pm.width(),
        height: pm.height(),
        rgba_data: pm.data().to_vec(),
    })
}

/// Render a Mermaid diagram with comment footer and save as PNG.
pub fn export_diagram_png(
    mermaid: &str,
    comments: &[String],
    output_path: &Path,
) -> Result<(), String> {
    let pm = render_composite(mermaid, comments)?;
    pm.save_png(output_path)
        .map_err(|e| format!("PNG save: {e}"))
}

// ── SVG footer injection (kept for backward compat / tests) ─────────────────

/// Inject a visible text footer into an SVG string.
///
/// **Note:** For PNG export prefer [`render_diagram`] / [`export_diagram_png`]
/// which composite the footer as a separate SVG — more reliable across
/// different SVG renderers and font configurations.
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
    let gap: f64 = 12.0;
    let footer_h = gap + pad_top + (comments.len() as f64 * line_height) + pad_bottom;
    let char_w: f64 = font_size * 0.62;

    let (vb_x, vb_y, vb_w, vb_h) = match parse_viewbox(svg) {
        Some(dims) => dims,
        None => return svg.to_string(),
    };

    let longest_text_w = comments
        .iter()
        .map(|c| pad_left + (c.len() as f64 * char_w) + pad_right)
        .fold(0.0_f64, f64::max);
    let new_vb_w = vb_w.max(longest_text_w);
    let new_vb_h = vb_h + footer_h;

    let mut footer = String::new();
    let footer_y = vb_y + vb_h + gap;
    let footer_content_h = footer_h - gap;

    footer.push_str(&format!(
        "<rect x=\"{vb_x}\" y=\"{footer_y}\" width=\"{new_vb_w}\" \
         height=\"{footer_content_h}\" fill=\"#FFFFFF\" rx=\"4\"/>\n",
    ));
    footer.push_str(&format!(
        "<line x1=\"{vb_x}\" y1=\"{footer_y}\" x2=\"{}\" y2=\"{footer_y}\" \
         stroke=\"#CCCCCC\" stroke-width=\"1\"/>\n",
        vb_x + new_vb_w,
    ));
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

    let mut result = replace_viewbox(svg, vb_x, vb_y, new_vb_w, new_vb_h);
    result = replace_svg_attr(&result, "width", new_vb_w);
    result = replace_svg_attr(&result, "height", new_vb_h);
    result.replace("</svg>", &format!("{footer}</svg>"))
}

// ── Internal: composite rendering ───────────────────────────────────────────

/// Render diagram + footer as separate SVGs, then composite into one pixmap.
fn render_composite(
    mermaid: &str,
    comments: &[String],
) -> Result<resvg::tiny_skia::Pixmap, String> {
    let is_mindmap = mermaid.starts_with("mindmap\n");
    let is_timeline = mermaid.starts_with("timeline\n");
    let options = if is_mindmap {
        dark_mindmap_render_options()
    } else {
        dark_render_options()
    };
    let svg = render_with_options(mermaid, options).map_err(|e| format!("SVG render: {e}"))?;
    let svg = if is_timeline {
        recolor_timeline_svg(&svg)
    } else {
        svg
    };

    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    // Rasterise the diagram
    let tree = usvg::Tree::from_str(&svg, &opt).map_err(|e| format!("SVG parse: {e}"))?;
    let size = tree.size().to_int_size();
    let mut diagram_pm = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| "Failed to allocate diagram pixmap".to_string())?;
    diagram_pm.fill(resvg::tiny_skia::Color::from_rgba8(0x12, 0x12, 0x12, 0xFF));
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut diagram_pm.as_mut(),
    );

    if comments.is_empty() {
        return Ok(diagram_pm);
    }

    // Render the footer as a completely separate SVG
    let footer_pm = render_footer_pixmap(comments, size.width(), &opt)?;

    // Stack vertically: diagram → 8px gap → footer
    composite_vertically(&diagram_pm, &footer_pm, 8)
}

/// Render comment text as a standalone high-contrast SVG and rasterise it.
///
/// The SVG is a self-contained document with its own white background and
/// dark text — no dependency on the Mermaid SVG's styles or clip regions.
fn render_footer_pixmap(
    comments: &[String],
    min_width: u32,
    opt: &usvg::Options,
) -> Result<resvg::tiny_skia::Pixmap, String> {
    let font_size = 24.0_f64;
    let line_height = 36.0_f64;
    let pad_x = 20.0_f64;
    let pad_y = 20.0_f64;
    let char_w = font_size * 0.62;

    let text_width: f64 = comments
        .iter()
        .map(|c| pad_x + (c.len() as f64 * char_w) + pad_x)
        .fold(0.0_f64, f64::max);
    let w = (min_width as f64).max(text_width).max(200.0);
    let h = pad_y + (comments.len() as f64 * line_height) + pad_y;

    // Broad font-family list so every OS finds something.
    let families = "Helvetica Neue, Helvetica, Arial, Segoe UI, \
                    DejaVu Sans, Liberation Sans, Noto Sans, sans-serif";

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" \
         width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\
         <rect width=\"{w}\" height=\"{h}\" fill=\"white\"/>\n"
    );
    for (i, line) in comments.iter().enumerate() {
        let y = pad_y + (i as f64 * line_height) + font_size;
        let escaped = xml_escape(line);
        svg.push_str(&format!(
            "<text x=\"{pad_x}\" y=\"{y}\" fill=\"#1A1A1A\" \
             font-family=\"{families}\" font-size=\"{font_size}\">\
             {escaped}</text>\n",
        ));
    }
    svg.push_str("</svg>");

    let tree = usvg::Tree::from_str(&svg, opt).map_err(|e| format!("Footer SVG parse: {e}"))?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| "Failed to allocate footer pixmap".to_string())?;
    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(0xFF, 0xFF, 0xFF, 0xFF));
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    Ok(pixmap)
}

/// Stack two pixmaps vertically with a gap filled by the dark background.
fn composite_vertically(
    top: &resvg::tiny_skia::Pixmap,
    bottom: &resvg::tiny_skia::Pixmap,
    gap: u32,
) -> Result<resvg::tiny_skia::Pixmap, String> {
    let w = top.width().max(bottom.width());
    let h = top.height() + gap + bottom.height();

    let mut result = resvg::tiny_skia::Pixmap::new(w, h)
        .ok_or_else(|| "Failed to allocate composite pixmap".to_string())?;
    result.fill(resvg::tiny_skia::Color::from_rgba8(0x12, 0x12, 0x12, 0xFF));

    let paint = resvg::tiny_skia::PixmapPaint::default();
    let xform = resvg::tiny_skia::Transform::identity();

    result.draw_pixmap(0, 0, top.as_ref(), &paint, xform, None);
    result.draw_pixmap(
        0,
        (top.height() + gap) as i32,
        bottom.as_ref(),
        &paint,
        xform,
        None,
    );

    Ok(result)
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Replace the hardcoded light-pastel event-box fills emitted by the
/// mermaid-rs-renderer with dark ilseon-palette equivalents.
///
/// The renderer cycles through six pastel colours for event boxes:
///   `#ECECFF`, `#FFE6CC`, `#D5E8D4`, `#F8CECC`, `#FFF2CC`, `#E1D5E7`
/// These are invisible on the dark `#121212` background and make the light
/// `#E0E0E0` text unreadable.  We swap them for deep-toned fills that
/// harmonise with the ilseon palette.
fn recolor_timeline_svg(svg: &str) -> String {
    svg.replace("#ECECFF", "#1A2D3A") // muted blue-teal
        .replace("#FFE6CC", "#2E2518") // deep amber
        .replace("#D5E8D4", "#1A2E24") // deep teal (matches Muted Teal)
        .replace("#F8CECC", "#2E1A1A") // deep red (matches Muted Red)
        .replace("#FFF2CC", "#2E2A18") // deep gold (matches Quiet Amber)
        .replace("#E1D5E7", "#241E2E") // deep purple
}

fn dark_render_options() -> RenderOptions {
    RenderOptions {
        theme: dark_theme(),
        ..RenderOptions::default()
    }
}

/// Render options for mindmap diagrams.
///
/// Uses the same dark theme but overrides `MindmapConfig.section_colors`
/// with ilseon palette tones so each branch depth gets a distinct colour.
fn dark_mindmap_render_options() -> RenderOptions {
    let mut layout = LayoutConfig::default();

    // Ilseon palette — cycled for branches at each depth:
    //   Muted Teal  #5A9B80   (primary)
    //   Quiet Amber #C08A3E   (secondary)
    //   Muted Red   #B35F5F   (tertiary)
    //   Light grey  #3A3A3A   (neutral – visible on #121212 bg)
    let fills = vec![
        "#5A9B80".into(), // teal
        "#C08A3E".into(), // amber
        "#B35F5F".into(), // red
        "#3A3A3A".into(), // neutral grey (lighter)
        "#5A9B80".into(),
        "#C08A3E".into(),
        "#B35F5F".into(),
        "#3A3A3A".into(),
        "#5A9B80".into(),
        "#C08A3E".into(),
        "#B35F5F".into(),
        "#3A3A3A".into(),
    ];

    // Label text — light on the coloured fills, lighter on the grey
    let labels = vec![
        "#E0E0E0".into(),
        "#E0E0E0".into(),
        "#E0E0E0".into(),
        "#B0B0B0".into(),
        "#E0E0E0".into(),
        "#E0E0E0".into(),
        "#E0E0E0".into(),
        "#B0B0B0".into(),
        "#E0E0E0".into(),
        "#E0E0E0".into(),
        "#E0E0E0".into(),
        "#B0B0B0".into(),
    ];

    // Lines between nodes — slightly lighter versions
    let lines = vec![
        "#7AC0A0".into(), // teal lighter
        "#D4A85E".into(), // amber lighter
        "#CC7F7F".into(), // red lighter
        "#707070".into(), // grey lighter
        "#7AC0A0".into(),
        "#D4A85E".into(),
        "#CC7F7F".into(),
        "#707070".into(),
        "#7AC0A0".into(),
        "#D4A85E".into(),
        "#CC7F7F".into(),
        "#707070".into(),
    ];

    layout.mindmap.section_colors = fills;
    layout.mindmap.section_label_colors = labels;
    layout.mindmap.section_line_colors = lines;

    // Centre root node — faded blue
    layout.mindmap.root_fill = Some("#4A6FA5".into());
    layout.mindmap.root_text = Some("#E0E0E0".into());

    RenderOptions {
        theme: dark_theme(),
        layout,
        ..RenderOptions::default()
    }
}

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

fn replace_svg_attr(svg: &str, attr: &str, new_value: f64) -> String {
    let needle = format!("{attr}=\"");
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
        assert!(result.matches("<rect").count() >= 2);
    }

    #[test]
    fn footer_uses_high_contrast_for_ocr() {
        let comments = vec!["searchable context".into()];
        let result = inject_svg_footer(STUB_SVG, &comments);
        assert!(result.contains("fill=\"#FFFFFF\""));
        assert!(result.contains("fill=\"#1A1A1A\""));
        assert!(result.contains("sans-serif"));
        assert!(result.contains("font-size=\"16\""));
    }

    #[test]
    fn footer_widens_viewbox_for_long_text() {
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

    // ── recolor_timeline_svg ────────────────────────────────────────────

    #[test]
    fn timeline_recolor_replaces_all_pastels() {
        let input = r##"fill="#ECECFF" fill="#FFE6CC" fill="#D5E8D4" fill="#F8CECC" fill="#FFF2CC" fill="#E1D5E7""##;
        let result = recolor_timeline_svg(input);
        // None of the light pastels should remain
        assert!(!result.contains("#ECECFF"));
        assert!(!result.contains("#FFE6CC"));
        assert!(!result.contains("#D5E8D4"));
        assert!(!result.contains("#F8CECC"));
        assert!(!result.contains("#FFF2CC"));
        assert!(!result.contains("#E1D5E7"));
    }

    #[test]
    fn timeline_recolor_uses_dark_fills() {
        let input = r##"fill="#ECECFF""##;
        let result = recolor_timeline_svg(input);
        assert!(result.contains("#1A2D3A"));
    }

    #[test]
    fn timeline_recolor_leaves_other_colors_intact() {
        let input = r##"fill="#121212" stroke="#5A9B80""##;
        let result = recolor_timeline_svg(input);
        assert_eq!(result, input);
    }
}
