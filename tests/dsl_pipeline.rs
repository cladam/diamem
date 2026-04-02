//! Integration tests for the diamem DSL → Mermaid pipeline.
//!
//! These test the full public API end-to-end: DSL text in → Mermaid text out,
//! exercising the pest parser and codegen together as a consumer would.

use diamem::dsl::{compile_dsl, dsl_to_mermaid};
use diamem::render;

// ── Happy-path: every DSL feature ───────────────────────────────────────────

#[test]
fn end_to_end_simple_connection() {
    let mermaid = dsl_to_mermaid("A -> B\n").unwrap();
    assert!(mermaid.starts_with("graph TD\n"));
    assert!(mermaid.contains("A --> B"));
}

#[test]
fn end_to_end_labeled_connection() {
    let mermaid = dsl_to_mermaid("Auth -[token]-> API\n").unwrap();
    assert!(mermaid.contains("Auth -->|token| API"));
}

#[test]
fn end_to_end_sequence() {
    let mermaid = dsl_to_mermaid("Browser > Server : GET /index\n").unwrap();
    assert!(mermaid.contains("Browser ->> Server: GET /index"));
}

#[test]
fn end_to_end_grouping() {
    let mermaid = dsl_to_mermaid("[Cloud] { LB, AppServer, DB }\n").unwrap();
    assert!(mermaid.contains("subgraph Cloud"));
    assert!(mermaid.contains("LB"));
    assert!(mermaid.contains("AppServer"));
    assert!(mermaid.contains("DB"));
    assert!(mermaid.contains("end"));
}

#[test]
fn end_to_end_standalone_node() {
    let mermaid = dsl_to_mermaid("Orphan\n").unwrap();
    assert!(mermaid.contains("Orphan"));
}

// ── Comments are invisible in output ────────────────────────────────────────

#[test]
fn comments_produce_no_mermaid_output() {
    let mermaid = dsl_to_mermaid("# this is a comment\n").unwrap();
    assert_eq!(mermaid, "graph TD\n");
}

#[test]
fn comments_mixed_with_statements() {
    let input = "# header\nA -> B\n# middle\nC -> D\n";
    let mermaid = dsl_to_mermaid(input).unwrap();
    assert!(mermaid.contains("A --> B"));
    assert!(mermaid.contains("C --> D"));
    assert!(!mermaid.contains("header"));
    assert!(!mermaid.contains("middle"));
}

// ── Full realistic diagrams ─────────────────────────────────────────────────

#[test]
fn realistic_architecture_diagram() {
    let input = "\
# Architecture overview
[Frontend] { WebApp, MobileApp }
[Backend] { API, Worker }
WebApp -> API
MobileApp -> API
API -[queries]-> DB
Worker -[reads]-> Queue
";
    let mermaid = dsl_to_mermaid(input).unwrap();

    assert!(mermaid.starts_with("graph TD\n"));
    assert!(mermaid.contains("subgraph Frontend"));
    assert!(mermaid.contains("subgraph Backend"));
    assert!(mermaid.contains("WebApp --> API"));
    assert!(mermaid.contains("MobileApp --> API"));
    assert!(mermaid.contains("API -->|queries| DB"));
    assert!(mermaid.contains("Worker -->|reads| Queue"));
}

#[test]
fn realistic_sequence_diagram() {
    let input = "\
User > AuthService : Login
AuthService > DB : Verify
DB > AuthService : OK
AuthService > User : Token
";
    let mermaid = dsl_to_mermaid(input).unwrap();

    assert!(mermaid.contains("User ->> AuthService: Login"));
    assert!(mermaid.contains("AuthService ->> DB: Verify"));
    assert!(mermaid.contains("DB ->> AuthService: OK"));
    assert!(mermaid.contains("AuthService ->> User: Token"));
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn empty_input_produces_only_header() {
    assert_eq!(dsl_to_mermaid("").unwrap(), "graph TD\n");
}

#[test]
fn blank_lines_produce_only_header() {
    assert_eq!(dsl_to_mermaid("\n\n\n").unwrap(), "graph TD\n");
}

#[test]
fn statement_ordering_is_preserved() {
    let input = "A -> B\nC -> D\nE -> F\n";
    let mermaid = dsl_to_mermaid(input).unwrap();

    let a_pos = mermaid.find("A --> B").unwrap();
    let c_pos = mermaid.find("C --> D").unwrap();
    let e_pos = mermaid.find("E --> F").unwrap();
    assert!(a_pos < c_pos);
    assert!(c_pos < e_pos);
}

// ── Error propagation ───────────────────────────────────────────────────────

#[test]
fn invalid_syntax_returns_error() {
    let result = dsl_to_mermaid("{{{{ not valid syntax ]]]]");
    assert!(result.is_err());
}

#[test]
fn error_message_is_nonempty() {
    let err = dsl_to_mermaid("{{{{ broken").unwrap_err();
    assert!(!err.is_empty());
}

// ── New features: chain connections ─────────────────────────────────────────

#[test]
fn end_to_end_chain_connection() {
    let mermaid = dsl_to_mermaid("Scaffold -> Parser -> BasicUI -> SVGRender\n").unwrap();
    assert!(mermaid.contains("Scaffold --> Parser"));
    assert!(mermaid.contains("Parser --> BasicUI"));
    assert!(mermaid.contains("BasicUI --> SVGRender"));
}

// ── New features: @ header grouping ─────────────────────────────────────────

#[test]
fn end_to_end_header_grouping() {
    let mermaid = dsl_to_mermaid("@ Phase 1: Scaffold, Parser, BasicUI\n").unwrap();
    assert!(mermaid.contains("subgraph Phase 1"));
    assert!(mermaid.contains("Scaffold"));
    assert!(mermaid.contains("Parser"));
    assert!(mermaid.contains("BasicUI"));
    assert!(mermaid.contains("end"));
}

// ── New features: -(label)> paren labeled connection ────────────────────────

#[test]
fn end_to_end_paren_labeled_connection() {
    let mermaid = dsl_to_mermaid("Auth -(token)> API\n").unwrap();
    assert!(mermaid.contains("Auth -->|token| API"));
}

// ── New features: realistic mixed diagram ───────────────────────────────────

#[test]
fn realistic_mixed_new_and_old_syntax() {
    let input = "\
# Simplified workflow
@ Phase1: Scaffold, Parser, BasicUI
@ Phase2: SVGRender, LivePreview
[Infra] { Kafka, Postgres, Redis }

Scaffold -> Parser -> BasicUI
API -(queries)> Postgres
API -[caches]-> Redis
User > API : POST /login
";
    let mermaid = dsl_to_mermaid(input).unwrap();

    assert!(mermaid.starts_with("graph TD\n"));
    assert!(mermaid.contains("subgraph Phase1"));
    assert!(mermaid.contains("subgraph Phase2"));
    assert!(mermaid.contains("subgraph Infra"));
    assert!(mermaid.contains("Scaffold --> Parser"));
    assert!(mermaid.contains("Parser --> BasicUI"));
    assert!(mermaid.contains("API -->|queries| Postgres"));
    assert!(mermaid.contains("API -->|caches| Redis"));
    assert!(mermaid.contains("User ->> API: POST /login"));
}

// ── Comment footer for Shotext OCR ──────────────────────────────────────────

#[test]
fn compile_dsl_extracts_comments() {
    let input = "\
# Architecture overview
# Version 2.0
A -> B
# internal note
";
    let (_mermaid, comments) = compile_dsl(input).unwrap();
    assert_eq!(
        comments,
        vec!["Architecture overview", "Version 2.0", "internal note"]
    );
}

#[test]
fn compile_dsl_empty_comments_are_excluded() {
    let input = "#\nA -> B\n# real comment\n";
    let (_, comments) = compile_dsl(input).unwrap();
    assert_eq!(comments, vec!["real comment"]);
}

#[test]
fn compile_dsl_no_comments_gives_empty_vec() {
    let input = "A -> B\nC -> D\n";
    let (_, comments) = compile_dsl(input).unwrap();
    assert!(comments.is_empty());
}

#[test]
fn inject_footer_makes_comments_visible_in_svg() {
    let stub_svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100" width="200" height="100"><rect/></svg>"##;
    let comments = vec!["Project: diamem".into(), "Author: Claes".into()];
    let result = render::inject_svg_footer(stub_svg, &comments);

    assert!(result.contains("Project: diamem"));
    assert!(result.contains("Author: Claes"));
    assert!(result.contains("<text"));
    // viewBox height should have grown
    assert!(!result.contains("viewBox=\"0 0 200 100\""));
}

// ── Mindmap end-to-end ──────────────────────────────────────────────────────

#[test]
fn end_to_end_mindmap() {
    let input = "\
mindmap: My Project
- Frontend
-- React
-- CSS
- Backend
-- Rust
-- PostgreSQL
";
    let mermaid = dsl_to_mermaid(input).unwrap();
    assert!(mermaid.starts_with("mindmap\n"));
    assert!(mermaid.contains("  My Project\n"));
    assert!(mermaid.contains("    Frontend\n"));
    assert!(mermaid.contains("      React\n"));
    assert!(mermaid.contains("      CSS\n"));
    assert!(mermaid.contains("    Backend\n"));
    assert!(mermaid.contains("      Rust\n"));
    assert!(mermaid.contains("      PostgreSQL\n"));
}

#[test]
fn mindmap_comments_are_extracted() {
    let input = "\
# Project overview
mindmap: Root
- A
";
    let (mermaid, comments) = compile_dsl(input).unwrap();
    assert!(mermaid.contains("mindmap\n"));
    assert_eq!(comments, vec!["Project overview"]);
}
