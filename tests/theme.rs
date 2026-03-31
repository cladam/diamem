//! Integration tests for theme palette constants.
//!
//! Verifies the public colour values match the ilseon specification
//! and that the palette has internally consistent contrast ordering.

use diamem::theme;

#[test]
fn ilseon_muted_teal() {
    // #5A9B80
    let c = theme::MUTED_TEAL;
    assert_eq!((c.r(), c.g(), c.b()), (0x5A, 0x9B, 0x80));
}

#[test]
fn ilseon_quiet_amber() {
    // #C08A3E
    let c = theme::QUIET_AMBER;
    assert_eq!((c.r(), c.g(), c.b()), (0xC0, 0x8A, 0x3E));
}

#[test]
fn ilseon_muted_red() {
    // #B35F5F
    let c = theme::MUTED_RED;
    assert_eq!((c.r(), c.g(), c.b()), (0xB3, 0x5F, 0x5F));
}

#[test]
fn ilseon_dark_grey() {
    // #121212
    let c = theme::DARK_BG;
    assert_eq!((c.r(), c.g(), c.b()), (0x12, 0x12, 0x12));
}

#[test]
fn background_surfaces_increase_in_brightness() {
    assert!(theme::DARK_BG.r() < theme::PANEL_BG.r());
    assert!(theme::PANEL_BG.r() < theme::SURFACE.r());
    assert!(theme::SURFACE.r() < theme::SURFACE_HOVER.r());
}

#[test]
fn text_hierarchy_increases_in_brightness() {
    assert!(theme::TEXT_MUTED.r() < theme::TEXT_SECONDARY.r());
    assert!(theme::TEXT_SECONDARY.r() < theme::TEXT_PRIMARY.r());
}

#[test]
fn text_primary_is_readable_on_dark_bg() {
    // Minimum contrast: text should be at least 150 brightness units above bg
    let contrast = theme::TEXT_PRIMARY.r() as i32 - theme::DARK_BG.r() as i32;
    assert!(contrast >= 150, "Text/bg contrast too low: {contrast}");
}
