## Project Specification: diamem
**Goal:** A high-performance desktop utility to "live-code" visual diagrams using a custom DSL, exporting them as searchable image assets for the **shotext** ecosystem.

### 1. Core Functional Requirements
* **DSL Editor:** A syntax-highlighted (or simple) text area in `egui` for rapid input.
* **Live Renderer:** Real-time conversion of DSL to Mermaid syntax and immediate visual feedback.
* **One-Click Externalise:** A button to render the current view to a high-resolution PNG and save it to a designated `shotext` watch folder.
* **Clipboard Support:** Ability to quickly paste a "brain dump" and have the DSL attempt a basic auto-format.

### 2. Technical Architecture (The Rust Stack)
* **GUI Framework:** `egui` (via `eframe`).
* **Mermaid Integration:**
    * `rusty-mermaid` for SVG generation.
* **Image Processing:** `resvg` to convert SVG renders into clean PNGs.
* **Serialization:** `serde` for saving user preferences (e.g., default export path).

### 3. The "Low-Friction" DSL Design
To avoid "syntax-paralysis," the DSL should follow a **Shorthand-First** approach.

| Input Type     | DSL Shorthand (Example)  | Mermaid Mapping                |
|:---------------|:-------------------------|:-------------------------------|
| **Connection** | `A -> B`                 | `A --> B`                      |
| **Labeled**    | `A -[action]-> B`        | `A -->                         |action| B` |
| **Sequence**   | `User > App : Request`   | `User ->> App: Request`        |
| **Grouping**   | `[Folder Name] { A, B }` | `subgraph Folder Name ... end` |

### 4. UI Layout (The "Split-Brain" Design)
The `egui` interface will be split into two primary panels:
1.  **The Engine (Left):**
    * A large `TextEdit` field.
    * A "Status" bar showing if the DSL is valid.
    * **Hotkeys:** `Ctrl + S` to export instantly.
2.  **The Memory (Right):**
    * An interactive `Image` or `Plot` widget showing the rendered diagram.
    * Zoom/Pan controls (essential for complex mind maps).
3.  **The Export Footer:**
    * Input for "Tags" (which will be burned into the image or filename).
    * A massive **"Commit to Shotext"** button.

### 5. Shotext Integration Strategy
To ensure the diagrams are useful once they land in `shotext`:
* **Filename Convention:** `diagram_YYYY-MM-DD_HHMM.png.png`.
* **The "Context Footer":** The tool will automatically append a small, high-contrast text bar to the bottom of the exported PNG containing the raw DSL.
    * *Why:* This ensures **shotext’s OCR** picks up the original text, making the diagram searchable even if the visual rendering is abstract.

### 6. Development Roadmap (The "ADHD-Proof" MVP)
* **Phase 1 (The Loop):** Simple `egui` window that takes text and prints "Mermaidified" text to the console.
* **Phase 2 (The Visual):** Integrate an SVG renderer to show the diagram in-app.
* **Phase 3 (The Export):** Implement the PNG saver and the `shotext` directory watcher link.
* **Phase 4 (The Polish):** Add custom DSL shortcuts and basic color themes (Nord, Gruvbox, etc.).

### 7. Success Metric
The tool is successful if you can go from **"Mental Image"** to **"Indexed in Shotext"** in under **60 seconds** without touching your mouse more than twice.

What’s the first "memory type" you’d want to test this with—a project plan, a code sequence, or a life routine?