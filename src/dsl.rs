use crate::parser::{self, Statement};

/// Parse DSL source and convert it to Mermaid syntax.
pub fn dsl_to_mermaid(dsl: &str) -> Result<String, String> {
    let statements = parser::parse(dsl)?;

    let mut output = String::from("graph TD\n");

    for stmt in &statements {
        match stmt {
            Statement::Comment(_) => {}
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

    Ok(output)
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
}
