# diamem

A desktop utility to **live-code visual diagrams** using a minimal DSL, exporting them as searchable PNG assets for [shotext](https://github.com/cladam/shotext).

Built with Rust, [eframe/egui](https://github.com/emilk/egui), and [mermaid-rs-renderer](https://crates.io/crates/mermaid-rs-renderer). Themed with the **ilseon** dark palette.

---

## Features

- **Live preview** — type DSL on the left, see the rendered diagram on the right
- **PNG export** — `Ctrl+S` or the ⬆ button saves a timestamped PNG
- **Shotext integration** — DSL comments (`# ...`) are rendered as a high-contrast OCR-readable footer in every exported PNG, so shotext can index them
- **Mindmap support** — `mindmap:` blocks generate Mermaid mindmaps with the ilseon colour palette
- **Dark theme** — OLED-friendly `#121212` background with muted teal / amber / red accents

## DSL Syntax

| What               | Syntax            | Example                     |
|--------------------|-------------------|-----------------------------|
| Comment            | `# text`          | `# this is ignored`         |
| Connection         | `A -> B`          | `Code -> Deploy`            |
| Chain              | `A -> B -> C`     | `Code -> Build -> Deploy`   |
| Labeled (classic)  | `A -[label]-> B`  | `API -[REST]-> DB`          |
| Labeled (short)    | `A -(label)> B`   | `API -(REST)> DB`           |
| Sequence           | `A > B : Message` | `User > App : Login`        |
| Grouping (classic) | `[Name] { A, B }` | `[Backend] { API, Worker }` |
| Grouping (short)   | `@ Name: A, B`    | `@ Backend: API, Worker`    |
| Mindmap root       | `mindmap: Root`   | `mindmap: My Project`       |
| Mindmap branch     | `- Name`          | `- Frontend`                |
| Mindmap leaf       | `-- Name`         | `-- React`                  |
| Standalone node    | `Name`            | `Orphan`                    |

> Both classic and short syntaxes can be mixed freely in one file.

## Quick Examples

### Flowchart

```text
# Architecture overview
@ Frontend: WebApp, MobileApp
@ Backend: API, Worker

WebApp -> API
MobileApp -> API
API -(queries)> DB
Worker -(reads)> Queue
```

### Mindmap

```text
mindmap: diamem
- DSL
-- Parser
-- Grammar
- Rendering
-- Mermaid
-- SVG
-- PNG Export
- UI
-- Editor
-- Preview
-- Themes
- Integration
-- Shotext
-- OCR Footer
```

### Sequence

```text
User > Browser : Opens app
Browser > API : POST /login
API > Auth : Validate
Auth > API : JWT
API > Browser : 200 OK
```

## Building

```bash
# Clone
git clone https://github.com/cladam/diamem.git
cd diamem

# Run
cargo run

# Run tests
cargo test
```

### Requirements

- Rust 2024 edition (1.85+)
- macOS / Linux / Windows (eframe + wgpu)

## Project Structure

```
src/
  main.rs          — entry point
  app.rs           — eframe UI (editor + preview + export)
  dsl.rs           — DSL → Mermaid code generation
  render.rs        — Mermaid → SVG → PNG pipeline + OCR footer
  theme.rs         — ilseon colour palette
  parser/
    diamem.pest    — pest grammar
    mod.rs         — pest → Statement AST
docs/
  dsl-cheatsheet.md
  examples.dsl
tests/
  parser.rs        — parser integration tests
  dsl_pipeline.rs  — end-to-end DSL → Mermaid tests
  examples.rs      — shipped examples smoke test
  theme.rs         — palette invariant tests
```

## License

[MIT](LICENSE) — Claes Adamsson
