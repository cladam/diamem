# Code Review: diamem

**Project:** Desktop utility to live-code visual diagrams via a custom DSL, exporting searchable PNGs for the shotext ecosystem.
**Stack:** Rust 2024, eframe/egui, pest parser, mermaid-rs-renderer, resvg/usvg.

---

## Overall Assessment

**Verdict: Well-structured, clean, and thoughtfully designed.** This is a solid MVP. The code is idiomatic Rust, the module boundaries are clear, the test suite is comprehensive, documentation is good, and the architecture cleanly separates parsing → codegen → rendering → UI. Below are specific findings grouped by severity.

---

## ✅ Strengths

| Area                  | Notes                                                                                                                                             |
|-----------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| **Architecture**      | Clean 4-stage pipeline: `pest grammar → Statement AST → Mermaid text → SVG/PNG`. Each stage is independently testable.                            |
| **Module boundaries** | `parser`, `dsl`, `render`, `theme`, `app` have tight cohesion and low coupling. The public API surface in `lib.rs` is minimal and correct.        |
| **Test coverage**     | ~80+ unit tests + integration tests covering every DSL feature, edge cases, error propagation, round-trips, and even shipped examples. Excellent. |
| **DSL design**        | Dual-syntax (classic `[Group]{…}` + shorthand `@ Group: …`) is user-friendly. Chain connections (`A -> B -> C`) are a nice touch.                 |
| **Error handling**    | `Result<_, String>` flows cleanly from parser → DSL → render. UI gracefully degrades on errors with distinct visual states.                       |
| **Theme**             | Well-organized palette constants with ordering invariant tests. Colour contrast is verified programmatically.                                     |
| **Security**          | No known CVEs in any dependency. No unsafe code. No user-input-as-command execution.                                                              |

---

## 🟡 Medium Issues

### 1. **13 Deprecated egui API Warnings** — `app.rs`

The code uses several deprecated eframe 0.34 APIs that will likely be removed in future versions:

| Deprecated | Replacement |
|---|---|
| `TopBottomPanel::top/bottom(…).show()` | `Panel::top/bottom(…).show_inside()` |
| `SidePanel::left(…).show()` | `Panel::left(…).show_inside()` |
| `CentralPanel::…show()` | `.show_inside()` |
| `egui::menu::bar()` | `egui::MenuBar::new().ui()` |
| `ui.close_menu()` | `ui.close()` |
| `.min_height()` / `.min_width()` | `.min_size()` |
| `.default_width()` | `.default_size()` |

**Recommendation:** Migrate to the new API now — these are all 1:1 renames and your code will compile warning-free.

### 2. **Blocking Mermaid render on every keystroke** — `app.rs:86-117`

`update_diagram()` is called on every frame and shells out to `mermaid-rs-renderer` whenever the DSL text changes. For complex diagrams, this could cause UI jank.

**Recommendation:** Add a debounce delay (e.g., 200–300ms after last keystroke) or move rendering to a background thread and display the last successful result until the new one is ready.

### 3. **`unwrap()` calls in parser** — `parser/mod.rs`

The parser uses `.unwrap()` ~12 times on pest inner pairs. While the grammar *should* guarantee these are present, a grammar bug could panic the app instead of producing a meaningful error.

**Recommendation:** Replace `unwrap()` calls with `.ok_or_else(|| "unexpected missing …".to_string())?` and propagate errors, or add a comment explaining the grammar invariant.

### 4. **No `#[deny(clippy::unwrap_used)]` or strict lints**

The project doesn't configure strict lints. Given this is a GUI app where panics kill the process, it would be wise to enforce error handling.

**Recommendation:** Add to `lib.rs`:
```rust
#![warn(clippy::unwrap_used, clippy::expect_used)]
```

---

## 🔵 Low / Style Issues

### 5. **Collapsible `if` statement** — `app.rs:164-168`

Clippy reports the nested `if let` in `expand_tilde` can be collapsed:
```rust
// Current:
if let Some(rest) = path.strip_prefix("~/") {
    if let Some(home) = dirs_home() {
        return format!("{}/{rest}", home);
    }
}

// Preferred:
if let (Some(rest), Some(home)) = (path.strip_prefix("~/"), dirs_home()) {
    return format!("{home}/{rest}");
}
```

### 6. **Dead struct update** — `render.rs:328`

```rust
RenderOptions {
    theme: dark_theme(),
    layout,
    ..RenderOptions::default()  // ← no effect, all fields specified
}
```
Clippy correctly notes the `..Default` is a no-op. Remove it for clarity.

### 7. **Hardcoded version string** — `app.rs:214`

```rust
self.status_message = "diamem v0.1.0 — Live Diagram Editor".into();
```
This will drift from `Cargo.toml`. Use:
```rust
format!("diamem v{} — Live Diagram Editor", env!("CARGO_PKG_VERSION"))
```

### 8. **`dirs_home()` reimplements `dirs::home_dir()`** — `app.rs:170-172`

The custom `dirs_home()` only reads `$HOME`, which won't work correctly on Windows. The `dirs` crate (tiny, 0 deps) handles all platforms.

**Recommendation:** Either add `dirs = "6"` as a dependency, or document that Windows is not a target.

### 9. **Footer text width estimation is approximate** — `render.rs:182`

```rust
let char_w = font_size * 0.62;
```
This assumes a monospaced-like character width for proportional fonts. Long CJK or emoji text will overflow; short text will under-fill.

**Recommendation:** Acceptable for MVP. For future accuracy, use the `usvg` tree's computed text bounding box.

### 10. **`inject_svg_footer` is legacy but still public** — `render.rs:68`

The doc comment says "for backward compat / tests" but it's exposed as `pub`. If it's only for tests, gate it:
```rust
#[cfg(test)]
pub fn inject_svg_footer(…) { … }
```
Or keep it public but mark it `#[deprecated(note = "use render_diagram instead")]`.

### 11. **Filename double extension in spec** — `docs/Project Sprecification.md:44`

> `diagram_YYYY-MM-DD_HHMM.png.png`

This appears to be a typo in the spec (`.png.png`). The actual code in `app.rs:139` correctly generates `.png` once. The spec also has "Sprecification" (typo for "Specification") in the filename.

### 12. **No `Ctrl+S` hint on macOS**

```rust
// app.rs:203
if ui.button("Export PNG (Ctrl+S)").clicked() {
```
On macOS the actual modifier is `⌘S` (Command+S), which is correctly wired in the `ctx.input()` check (line 191), but the label says "Ctrl+S".

**Recommendation:** Detect OS at runtime and show the correct modifier:
```rust
let shortcut = if cfg!(target_os = "macos") { "⌘S" } else { "Ctrl+S" };
```

---

## 📊 Metrics Summary

| Metric                      | Value                        |
|-----------------------------|------------------------------|
| Source lines (excl. tests)  | ~750                         |
| Test count                  | 80+ (unit + integration)     |
| Compiler warnings           | 13 (all deprecated API)      |
| Clippy warnings             | 15 (13 deprecated + 2 style) |
| CVEs                        | 0                            |
| Unsafe code                 | 0                            |
| `unwrap()` in non-test code | ~12 (all in parser)          |
| Public API surface          | Minimal and clean            |

---

## 🏁 Recommended Priority Actions

1. **Fix the 13 deprecated egui API calls** — prevents breakage on next eframe upgrade
2. **Replace parser `unwrap()`s** with proper error propagation
3. **Add render debounce** — improves UX for complex diagrams
4. **Fix the 2 clippy warnings** (collapsible `if`, dead struct update)
5. **Use `CARGO_PKG_VERSION`** instead of hardcoded version string
6. **Fix macOS shortcut label** — cosmetic but user-facing
