use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/diamem.pest"]
struct DiamemParser;

/// A parsed DSL statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Comment(String),
    Connection { from: String, to: String },
    LabeledConnection { from: String, to: String, label: String },
    Sequence { from: String, to: String, message: String },
    Grouping { name: String, nodes: Vec<String> },
    Node(String),
}

/// Parse DSL source into a list of statements.
pub fn parse(input: &str) -> Result<Vec<Statement>, String> {
    let pairs = DiamemParser::parse(Rule::diagram, input)
        .map_err(|e| format!("{e}"))?;

    let mut statements = Vec::new();

    for pair in pairs {
        if pair.as_rule() != Rule::diagram {
            continue;
        }
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::comment => {
                    let text = inner
                        .into_inner()
                        .find(|p| p.as_rule() == Rule::comment_text)
                        .map(|p| p.as_str().trim().to_string())
                        .unwrap_or_default();
                    statements.push(Statement::Comment(text));
                }
                Rule::connection => {
                    let mut inner = inner.into_inner();
                    let from = inner.next().unwrap().as_str().trim().to_string();
                    let to = inner.next().unwrap().as_str().trim().to_string();
                    statements.push(Statement::Connection { from, to });
                }
                Rule::labeled_connection => {
                    let mut inner = inner.into_inner();
                    let from = inner.next().unwrap().as_str().trim().to_string();
                    let label = inner.next().unwrap().as_str().trim().to_string();
                    let to = inner.next().unwrap().as_str().trim().to_string();
                    statements.push(Statement::LabeledConnection { from, to, label });
                }
                Rule::sequence => {
                    let mut inner = inner.into_inner();
                    let from = inner.next().unwrap().as_str().trim().to_string();
                    let to = inner.next().unwrap().as_str().trim().to_string();
                    let message = inner.next().unwrap().as_str().trim().to_string();
                    statements.push(Statement::Sequence { from, to, message });
                }
                Rule::grouping => {
                    let mut inner = inner.into_inner();
                    let name = inner.next().unwrap().as_str().trim().to_string();
                    let node_list = inner.next().unwrap();
                    let nodes: Vec<String> = node_list
                        .into_inner()
                        .filter(|p| p.as_rule() == Rule::ident)
                        .map(|p| p.as_str().trim().to_string())
                        .collect();
                    statements.push(Statement::Grouping { name, nodes });
                }
                Rule::node => {
                    let ident = inner.into_inner().next().unwrap().as_str().trim().to_string();
                    statements.push(Statement::Node(ident));
                }
                Rule::EOI => {}
                _ => {}
            }
        }
    }

    Ok(statements)
}
