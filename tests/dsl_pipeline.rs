//! Integration tests for the diamem DSL → Mermaid pipeline.
//!
//! These test the full public API end-to-end: DSL text in → Mermaid text out,
//! exercising the pest parser and codegen together as a consumer would.

use diamem::dsl::dsl_to_mermaid;

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
