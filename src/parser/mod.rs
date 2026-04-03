use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/diamem.pest"]
struct DiamemParser;

/// A single entry in a mindmap block: depth (dash count) + label text.
#[derive(Debug, Clone, PartialEq)]
pub struct MindmapEntry {
    pub depth: usize,
    pub label: String,
}

/// A single entry in a timeline block: depth (dash count) + text + optional events.
///
/// Depth 1 without events → section header.
/// Depth 1 with events → top-level time period.
/// Depth 2 with events → time period inside a section.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineEntry {
    pub depth: usize,
    pub text: String,
    pub events: Vec<String>,
}

/// A parsed DSL statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Comment(String),
    Connection {
        from: String,
        to: String,
    },
    LabeledConnection {
        from: String,
        to: String,
        label: String,
    },
    Sequence {
        from: String,
        to: String,
        message: String,
    },
    Grouping {
        name: String,
        nodes: Vec<String>,
    },
    Mindmap {
        root: String,
        entries: Vec<MindmapEntry>,
    },
    Timeline {
        title: String,
        entries: Vec<TimelineEntry>,
    },
    Node(String),
}

/// Parse DSL source into a list of statements.
pub fn parse(input: &str) -> Result<Vec<Statement>, String> {
    let pairs = DiamemParser::parse(Rule::diagram, input).map_err(|e| format!("{e}"))?;

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
                Rule::mindmap_block => {
                    let mut inner = inner.into_inner();
                    let root = inner.next().unwrap().as_str().trim().to_string();
                    let entries: Vec<MindmapEntry> = inner
                        .filter(|p| p.as_rule() == Rule::mindmap_entry)
                        .map(|entry| {
                            let mut parts = entry.into_inner();
                            let depth = parts.next().unwrap().as_str().len();
                            let label = parts.next().unwrap().as_str().trim().to_string();
                            MindmapEntry { depth, label }
                        })
                        .collect();
                    statements.push(Statement::Mindmap { root, entries });
                }
                Rule::timeline_block => {
                    let mut inner = inner.into_inner();
                    let title = inner.next().unwrap().as_str().trim().to_string();
                    let entries: Vec<TimelineEntry> = inner
                        .filter(|p| p.as_rule() == Rule::timeline_entry)
                        .map(|entry| {
                            let mut parts = entry.into_inner();
                            let marker = parts.next().unwrap().as_str();
                            let content = parts.next().unwrap().as_str().trim().to_string();
                            if marker == "@" {
                                // Section header — never has events
                                TimelineEntry {
                                    depth: 0,
                                    text: content,
                                    events: vec![],
                                }
                            } else if let Some((period, events_str)) = content.split_once(':') {
                                let events = events_str
                                    .split(',')
                                    .map(|e| e.trim().to_string())
                                    .filter(|e| !e.is_empty())
                                    .collect();
                                TimelineEntry {
                                    depth: 1,
                                    text: period.trim().to_string(),
                                    events,
                                }
                            } else {
                                // Bare period without events
                                TimelineEntry {
                                    depth: 1,
                                    text: content,
                                    events: vec![],
                                }
                            }
                        })
                        .collect();
                    statements.push(Statement::Timeline { title, entries });
                }
                Rule::connection => {
                    let idents: Vec<String> = inner
                        .into_inner()
                        .filter(|p| p.as_rule() == Rule::ident)
                        .map(|p| p.as_str().trim().to_string())
                        .collect();
                    for pair in idents.windows(2) {
                        statements.push(Statement::Connection {
                            from: pair[0].clone(),
                            to: pair[1].clone(),
                        });
                    }
                }
                Rule::labeled_connection => {
                    let mut inner = inner.into_inner();
                    let from = inner.next().unwrap().as_str().trim().to_string();
                    let label = inner.next().unwrap().as_str().trim().to_string();
                    let to = inner.next().unwrap().as_str().trim().to_string();
                    statements.push(Statement::LabeledConnection { from, to, label });
                }
                Rule::paren_labeled_connection => {
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
                Rule::grouping | Rule::header_grouping => {
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
                    let ident = inner
                        .into_inner()
                        .next()
                        .unwrap()
                        .as_str()
                        .trim()
                        .to_string();
                    statements.push(Statement::Node(ident));
                }
                Rule::EOI => {}
                _ => {}
            }
        }
    }

    Ok(statements)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Comment ──────────────────────────────────────────────────────────

    #[test]
    fn parse_comment() {
        let stmts = parse("# hello world\n").unwrap();
        assert_eq!(stmts, vec![Statement::Comment("hello world".into())]);
    }

    #[test]
    fn parse_comment_empty() {
        let stmts = parse("#\n").unwrap();
        assert_eq!(stmts, vec![Statement::Comment(String::new())]);
    }

    // ── Connection ───────────────────────────────────────────────────────

    #[test]
    fn parse_simple_connection() {
        let stmts = parse("A -> B\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Connection {
                from: "A".into(),
                to: "B".into(),
            }]
        );
    }

    #[test]
    fn parse_connection_multiword_idents() {
        let stmts = parse("ServiceA -> ServiceB\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Connection {
                from: "ServiceA".into(),
                to: "ServiceB".into(),
            }]
        );
    }

    // ── Labeled connection ───────────────────────────────────────────────

    #[test]
    fn parse_labeled_connection() {
        let stmts = parse("A -[sends]-> B\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::LabeledConnection {
                from: "A".into(),
                to: "B".into(),
                label: "sends".into(),
            }]
        );
    }

    #[test]
    fn parse_labeled_connection_with_spaces_in_label() {
        let stmts = parse("X -[http post]-> Y\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::LabeledConnection {
                from: "X".into(),
                to: "Y".into(),
                label: "http post".into(),
            }]
        );
    }

    // ── Sequence ─────────────────────────────────────────────────────────

    #[test]
    fn parse_sequence() {
        let stmts = parse("User > App : Request\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Sequence {
                from: "User".into(),
                to: "App".into(),
                message: "Request".into(),
            }]
        );
    }

    #[test]
    fn parse_sequence_long_message() {
        let stmts = parse("Client > Server : POST /api/data\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Sequence {
                from: "Client".into(),
                to: "Server".into(),
                message: "POST /api/data".into(),
            }]
        );
    }

    // ── Grouping ─────────────────────────────────────────────────────────

    #[test]
    fn parse_grouping() {
        let stmts = parse("[Backend] { API, DB }\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Grouping {
                name: "Backend".into(),
                nodes: vec!["API".into(), "DB".into()],
            }]
        );
    }

    #[test]
    fn parse_grouping_single_node() {
        let stmts = parse("[Solo] { OnlyOne }\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Grouping {
                name: "Solo".into(),
                nodes: vec!["OnlyOne".into()],
            }]
        );
    }

    #[test]
    fn parse_grouping_name_with_spaces() {
        let stmts = parse("[My Group] { A, B, C }\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Grouping {
                name: "My Group".into(),
                nodes: vec!["A".into(), "B".into(), "C".into()],
            }]
        );
    }

    // ── Node ─────────────────────────────────────────────────────────────

    #[test]
    fn parse_standalone_node() {
        let stmts = parse("Standalone\n").unwrap();
        assert_eq!(stmts, vec![Statement::Node("Standalone".into())]);
    }

    // ── Chain connections ────────────────────────────────────────────────

    #[test]
    fn parse_chain_connection() {
        let stmts = parse("A -> B -> C\n").unwrap();
        assert_eq!(
            stmts,
            vec![
                Statement::Connection {
                    from: "A".into(),
                    to: "B".into(),
                },
                Statement::Connection {
                    from: "B".into(),
                    to: "C".into(),
                },
            ]
        );
    }

    #[test]
    fn parse_long_chain() {
        let stmts = parse("A -> B -> C -> D -> E\n").unwrap();
        assert_eq!(stmts.len(), 4);
        assert_eq!(
            stmts[0],
            Statement::Connection {
                from: "A".into(),
                to: "B".into()
            }
        );
        assert_eq!(
            stmts[3],
            Statement::Connection {
                from: "D".into(),
                to: "E".into()
            }
        );
    }

    // ── Header grouping (@) ─────────────────────────────────────────────

    #[test]
    fn parse_header_grouping() {
        let stmts = parse("@ Backend: API, DB\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Grouping {
                name: "Backend".into(),
                nodes: vec!["API".into(), "DB".into()],
            }]
        );
    }

    #[test]
    fn parse_header_grouping_with_spaces_in_name() {
        let stmts = parse("@ Phase 1: Scaffold, Parser, BasicUI\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Grouping {
                name: "Phase 1".into(),
                nodes: vec!["Scaffold".into(), "Parser".into(), "BasicUI".into()],
            }]
        );
    }

    #[test]
    fn parse_header_grouping_single_node() {
        let stmts = parse("@ Solo: OnlyOne\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Grouping {
                name: "Solo".into(),
                nodes: vec!["OnlyOne".into()],
            }]
        );
    }

    // ── Paren labeled connection ────────────────────────────────────────

    #[test]
    fn parse_paren_labeled_connection() {
        let stmts = parse("A -(sends)> B\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::LabeledConnection {
                from: "A".into(),
                to: "B".into(),
                label: "sends".into(),
            }]
        );
    }

    #[test]
    fn parse_paren_labeled_connection_with_spaces_in_label() {
        let stmts = parse("X -(http post)> Y\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::LabeledConnection {
                from: "X".into(),
                to: "Y".into(),
                label: "http post".into(),
            }]
        );
    }

    // ── Multiple statements ──────────────────────────────────────────────

    #[test]
    fn parse_multiple_statements() {
        let input = "# setup\nA -> B\nB -[calls]-> C\n";
        let stmts = parse(input).unwrap();
        assert_eq!(stmts.len(), 3);
        assert!(matches!(&stmts[0], Statement::Comment(_)));
        assert!(matches!(&stmts[1], Statement::Connection { .. }));
        assert!(matches!(&stmts[2], Statement::LabeledConnection { .. }));
    }

    // ── Mindmap ──────────────────────────────────────────────────────────

    #[test]
    fn parse_mindmap_basic() {
        let input = "mindmap: Central\n- Branch1\n- Branch2\n";
        let stmts = parse(input).unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Mindmap {
                root: "Central".into(),
                entries: vec![
                    MindmapEntry {
                        depth: 1,
                        label: "Branch1".into()
                    },
                    MindmapEntry {
                        depth: 1,
                        label: "Branch2".into()
                    },
                ],
            }]
        );
    }

    #[test]
    fn parse_mindmap_nested() {
        let input = "mindmap: Root\n- A\n-- A1\n-- A2\n- B\n--- Deep\n";
        let stmts = parse(input).unwrap();
        if let Statement::Mindmap { root, entries } = &stmts[0] {
            assert_eq!(root, "Root");
            assert_eq!(entries.len(), 5);
            assert_eq!(
                entries[0],
                MindmapEntry {
                    depth: 1,
                    label: "A".into()
                }
            );
            assert_eq!(
                entries[1],
                MindmapEntry {
                    depth: 2,
                    label: "A1".into()
                }
            );
            assert_eq!(
                entries[2],
                MindmapEntry {
                    depth: 2,
                    label: "A2".into()
                }
            );
            assert_eq!(
                entries[3],
                MindmapEntry {
                    depth: 1,
                    label: "B".into()
                }
            );
            assert_eq!(
                entries[4],
                MindmapEntry {
                    depth: 3,
                    label: "Deep".into()
                }
            );
        } else {
            panic!("Expected Mindmap statement");
        }
    }

    #[test]
    fn parse_mindmap_root_only() {
        let stmts = parse("mindmap: JustRoot\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Mindmap {
                root: "JustRoot".into(),
                entries: vec![],
            }]
        );
    }

    #[test]
    fn parse_mindmap_root_with_spaces() {
        let stmts = parse("mindmap: My Big Idea\n- First\n").unwrap();
        if let Statement::Mindmap { root, .. } = &stmts[0] {
            assert_eq!(root, "My Big Idea");
        } else {
            panic!("Expected Mindmap");
        }
    }

    #[test]
    fn parse_mindmap_word_is_valid_node() {
        // "mindmap" without ":" is a standalone node, not a mindmap block
        let stmts = parse("mindmap\n").unwrap();
        assert_eq!(stmts, vec![Statement::Node("mindmap".into())]);
    }

    // ── Timeline ──────────────────────────────────────────────────────

    #[test]
    fn parse_timeline_basic() {
        let input = "timeline: History\n- 2002 : LinkedIn\n- 2004 : Facebook\n";
        let stmts = parse(input).unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Timeline {
                title: "History".into(),
                entries: vec![
                    TimelineEntry {
                        depth: 1,
                        text: "2002".into(),
                        events: vec!["LinkedIn".into()],
                    },
                    TimelineEntry {
                        depth: 1,
                        text: "2004".into(),
                        events: vec!["Facebook".into()],
                    },
                ],
            }]
        );
    }

    #[test]
    fn parse_timeline_multiple_events() {
        let input = "timeline: History\n- 2004 : Facebook, Google\n";
        let stmts = parse(input).unwrap();
        if let Statement::Timeline { entries, .. } = &stmts[0] {
            assert_eq!(entries[0].events, vec!["Facebook", "Google"]);
        } else {
            panic!("Expected Timeline statement");
        }
    }

    #[test]
    fn parse_timeline_with_sections() {
        let input = "\
timeline: History
@ Early Days
- 2002 : LinkedIn
- 2004 : Facebook, Google
@ Growth
- 2005 : Youtube
- 2006 : Twitter
";
        let stmts = parse(input).unwrap();
        if let Statement::Timeline { title, entries } = &stmts[0] {
            assert_eq!(title, "History");
            assert_eq!(entries.len(), 6);
            // Section headers have depth 0 and no events
            assert_eq!(
                entries[0],
                TimelineEntry {
                    depth: 0,
                    text: "Early Days".into(),
                    events: vec![],
                }
            );
            assert_eq!(entries[1].depth, 1);
            assert_eq!(entries[1].text, "2002");
            assert_eq!(entries[1].events, vec!["LinkedIn"]);
            assert_eq!(entries[3].text, "Growth");
            assert!(entries[3].events.is_empty());
            assert_eq!(entries[5].events, vec!["Twitter"]);
        } else {
            panic!("Expected Timeline statement");
        }
    }

    #[test]
    fn parse_timeline_title_only() {
        let stmts = parse("timeline: Just A Title\n").unwrap();
        assert_eq!(
            stmts,
            vec![Statement::Timeline {
                title: "Just A Title".into(),
                entries: vec![],
            }]
        );
    }

    #[test]
    fn parse_timeline_word_is_valid_node() {
        // "timeline" without ":" is a standalone node, not a timeline block
        let stmts = parse("timeline\n").unwrap();
        assert_eq!(stmts, vec![Statement::Node("timeline".into())]);
    }

    // ── Empty / whitespace ───────────────────────────────────────────────

    #[test]
    fn parse_empty_input() {
        let stmts = parse("").unwrap();
        assert!(stmts.is_empty());
    }

    #[test]
    fn parse_blank_lines_only() {
        let stmts = parse("\n\n\n").unwrap();
        assert!(stmts.is_empty());
    }

    // ── Error cases ──────────────────────────────────────────────────────

    #[test]
    fn parse_invalid_syntax_returns_error() {
        let result = parse("??? totally broken {{{");
        assert!(result.is_err());
    }
}
