use crate::parser::{self, Statement};

/// Parse DSL source and convert it to Mermaid syntax.
pub fn dsl_to_mermaid(dsl: &str) -> Result<String, String> {
    let statements = parser::parse(dsl)?;
    Ok(build_mermaid(&statements))
}

/// Parse DSL source and return both Mermaid syntax and extracted comment text.
///
/// The comment list contains only non-empty `# …` lines, in document order.
/// These can be passed to [`crate::render::inject_svg_footer`] so Shotext
/// can OCR them out of the exported PNG.
pub fn compile_dsl(dsl: &str) -> Result<(String, Vec<String>), String> {
    let statements = parser::parse(dsl)?;
    let mermaid = build_mermaid(&statements);
    let comments = extract_comments(&statements);
    Ok((mermaid, comments))
}

/// Build Mermaid syntax from parsed statements.
///
/// If any statement is a `Mindmap`, the output uses Mermaid's `mindmap`
/// diagram type.  Otherwise it falls back to `graph TD`.
fn build_mermaid(statements: &[Statement]) -> String {
    let has_mindmap = statements
        .iter()
        .any(|s| matches!(s, Statement::Mindmap { .. }));

    if has_mindmap {
        build_mindmap(statements)
    } else {
        build_graph(statements)
    }
}

/// Build a `graph TD` Mermaid diagram.
fn build_graph(statements: &[Statement]) -> String {
    let mut output = String::from("graph TD\n");

    for stmt in statements {
        match stmt {
            Statement::Comment(_) | Statement::Mindmap { .. } => {}
            Statement::Connection { from, to } => {
                output.push_str(&format!("    {from} --> {to}\n"));
            }
            Statement::LabeledConnection { from, to, label } => {
                output.push_str(&format!("    {from} -->|{label}| {to}\n"));
            }
            Statement::Sequence { from, to, message } => {
                output.push_str(&format!("    {from} ->> {to}: {message}\n"));
            }
            Statement::Grouping { name, nodes } => {
                output.push_str(&format!("    subgraph {name}\n"));
                for node in nodes {
                    output.push_str(&format!("        {node}\n"));
                }
                output.push_str("    end\n");
            }
            Statement::Node(ident) => {
                output.push_str(&format!("    {ident}\n"));
            }
        }
    }

    output
}

/// Build a `mindmap` Mermaid diagram.
///
/// Dash-count in each entry maps to indentation depth:
/// root → 2 spaces, depth 1 (`-`) → 4 spaces, depth 2 (`--`) → 6, etc.
fn build_mindmap(statements: &[Statement]) -> String {
    let mut output = String::from("mindmap\n");

    for stmt in statements {
        if let Statement::Mindmap { root, entries } = stmt {
            // Root at indent level 1 (2 spaces)
            output.push_str(&format!("  {root}\n"));
            for entry in entries {
                let indent = " ".repeat((entry.depth + 1) * 2);
                output.push_str(&format!("{indent}{}\n", entry.label));
            }
        }
        // Comments and graph statements are silently skipped in mindmap mode
    }

    output
}

/// Collect non-empty comment text from parsed statements, in document order.
fn extract_comments(statements: &[Statement]) -> Vec<String> {
    statements
        .iter()
        .filter_map(|s| match s {
            Statement::Comment(text) if !text.is_empty() => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mermaid_header() {
        let result = dsl_to_mermaid("").unwrap();
        assert_eq!(result, "graph TD\n");
    }

    #[test]
    fn mermaid_connection() {
        let result = dsl_to_mermaid("A -> B\n").unwrap();
        assert!(result.contains("A --> B"));
    }

    #[test]
    fn mermaid_labeled_connection() {
        let result = dsl_to_mermaid("A -[sends]-> B\n").unwrap();
        assert!(result.contains("A -->|sends| B"));
    }

    #[test]
    fn mermaid_sequence() {
        let result = dsl_to_mermaid("User > App : Request\n").unwrap();
        assert!(result.contains("User ->> App: Request"));
    }

    #[test]
    fn mermaid_grouping() {
        let result = dsl_to_mermaid("[Backend] { API, DB }\n").unwrap();
        assert!(result.contains("subgraph Backend"));
        assert!(result.contains("API"));
        assert!(result.contains("DB"));
        assert!(result.contains("end"));
    }

    #[test]
    fn mermaid_standalone_node() {
        let result = dsl_to_mermaid("Lonely\n").unwrap();
        assert!(result.contains("    Lonely\n"));
    }

    #[test]
    fn mermaid_comments_are_skipped() {
        let result = dsl_to_mermaid("# just a comment\n").unwrap();
        assert_eq!(result, "graph TD\n");
    }

    #[test]
    fn mermaid_full_diagram() {
        let input = "\
# My diagram
A -> B
B -[validate]-> C
[Infra] { Redis, Postgres }
";
        let result = dsl_to_mermaid(input).unwrap();
        assert!(result.starts_with("graph TD\n"));
        assert!(result.contains("A --> B"));
        assert!(result.contains("B -->|validate| C"));
        assert!(result.contains("subgraph Infra"));
        assert!(result.contains("Redis"));
        assert!(result.contains("Postgres"));
    }

    #[test]
    fn mermaid_propagates_parse_error() {
        let result = dsl_to_mermaid("{{{{ broken");
        assert!(result.is_err());
    }

    // ── New syntax features ─────────────────────────────────────────────

    #[test]
    fn mermaid_chain_connection() {
        let result = dsl_to_mermaid("A -> B -> C\n").unwrap();
        assert!(result.contains("A --> B"));
        assert!(result.contains("B --> C"));
    }

    #[test]
    fn mermaid_header_grouping() {
        let result = dsl_to_mermaid("@ Backend: API, DB\n").unwrap();
        assert!(result.contains("subgraph Backend"));
        assert!(result.contains("API"));
        assert!(result.contains("DB"));
        assert!(result.contains("end"));
    }

    #[test]
    fn mermaid_paren_labeled_connection() {
        let result = dsl_to_mermaid("A -(sends)> B\n").unwrap();
        assert!(result.contains("A -->|sends| B"));
    }

    #[test]
    fn mermaid_mixed_old_and_new_syntax() {
        let input = "\
@ Phase1: Scaffold, Parser, BasicUI
[Phase2] { SVGRender, LivePreview }
Scaffold -> Parser -> BasicUI
API -(queries)> DB
API -[caches]-> Redis
";
        let result = dsl_to_mermaid(input).unwrap();
        assert!(result.contains("subgraph Phase1"));
        assert!(result.contains("subgraph Phase2"));
        assert!(result.contains("Scaffold --> Parser"));
        assert!(result.contains("Parser --> BasicUI"));
        assert!(result.contains("API -->|queries| DB"));
        assert!(result.contains("API -->|caches| Redis"));
    }

    // ── Mindmap ─────────────────────────────────────────────────────────

    #[test]
    fn mindmap_header() {
        let input = "mindmap: Root\n- Child\n";
        let result = dsl_to_mermaid(input).unwrap();
        assert!(result.starts_with("mindmap\n"));
    }

    #[test]
    fn mindmap_root_and_branches() {
        let input = "mindmap: Central Topic\n- Branch1\n- Branch2\n";
        let result = dsl_to_mermaid(input).unwrap();
        assert!(result.contains("  Central Topic\n"));
        assert!(result.contains("    Branch1\n"));
        assert!(result.contains("    Branch2\n"));
    }

    #[test]
    fn mindmap_nested_depth() {
        let input = "\
mindmap: Root
- A
-- A1
--- Deep
";
        let result = dsl_to_mermaid(input).unwrap();
        // Root at 2 spaces, depth 1 at 4, depth 2 at 6, depth 3 at 8
        assert!(result.contains("  Root\n"));
        assert!(result.contains("    A\n"));
        assert!(result.contains("      A1\n"));
        assert!(result.contains("        Deep\n"));
    }

    #[test]
    fn mindmap_does_not_emit_graph_td() {
        let input = "mindmap: X\n- Y\n";
        let result = dsl_to_mermaid(input).unwrap();
        assert!(!result.contains("graph TD"));
    }

    #[test]
    fn graph_without_mindmap_emits_graph_td() {
        let result = dsl_to_mermaid("A -> B\n").unwrap();
        assert!(result.starts_with("graph TD\n"));
        assert!(!result.starts_with("mindmap\n"));
    }
}
