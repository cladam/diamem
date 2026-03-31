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
