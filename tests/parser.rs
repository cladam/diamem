//! Integration tests for the pest parser module.
//!
//! Tests the parser's public API directly with various inputs,
//! verifying the AST structure from the crate boundary.

use diamem::parser::{self, Statement};

// ── Round-trip: every statement type ────────────────────────────────────────

#[test]
fn parse_all_statement_types_in_one_input() {
    let input = "\
# comment
A -> B
X -[label]-> Y
User > App : Msg
[Group] { N1, N2 }
Solo
";
    let stmts = parser::parse(input).unwrap();
    assert_eq!(stmts.len(), 6);

    assert!(matches!(&stmts[0], Statement::Comment(t) if t == "comment"));
    assert!(matches!(&stmts[1], Statement::Connection { from, to } if from == "A" && to == "B"));
    assert!(
        matches!(&stmts[2], Statement::LabeledConnection { from, to, label }
        if from == "X" && to == "Y" && label == "label")
    );
    assert!(
        matches!(&stmts[3], Statement::Sequence { from, to, message }
        if from == "User" && to == "App" && message == "Msg")
    );
    assert!(matches!(&stmts[4], Statement::Grouping { name, nodes }
        if name == "Group" && nodes == &["N1", "N2"]));
    assert!(matches!(&stmts[5], Statement::Node(n) if n == "Solo"));
}

// ── Whitespace tolerance ────────────────────────────────────────────────────

#[test]
fn extra_blank_lines_between_statements() {
    let input = "A -> B\n\n\nC -> D\n";
    let stmts = parser::parse(input).unwrap();
    assert_eq!(stmts.len(), 2);
}

#[test]
fn leading_and_trailing_newlines() {
    let input = "\n\nA -> B\n\n";
    let stmts = parser::parse(input).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Statement counts ────────────────────────────────────────────────────────

#[test]
fn many_connections() {
    let input = (0..20)
        .map(|i| format!("N{i} -> N{}\n", i + 1))
        .collect::<String>();
    let stmts = parser::parse(&input).unwrap();
    assert_eq!(stmts.len(), 20);
    assert!(
        stmts
            .iter()
            .all(|s| matches!(s, Statement::Connection { .. }))
    );
}

// ── Clone / equality (derive coverage) ──────────────────────────────────────

#[test]
fn statement_clone_equals_original() {
    let stmts = parser::parse("A -> B\n").unwrap();
    let cloned = stmts[0].clone();
    assert_eq!(stmts[0], cloned);
}

#[test]
fn statement_debug_is_nonempty() {
    let stmts = parser::parse("A -> B\n").unwrap();
    let debug = format!("{:?}", stmts[0]);
    assert!(!debug.is_empty());
}

// ── Error cases ─────────────────────────────────────────────────────────────

#[test]
fn unclosed_grouping_bracket_errors() {
    let result = parser::parse("[Open { A, B }");
    assert!(result.is_err());
}

#[test]
fn unclosed_label_bracket_errors() {
    let result = parser::parse("A -[broken -> B\n");
    assert!(result.is_err());
}

// ── New syntax: chain connections ───────────────────────────────────────────

#[test]
fn chain_connection_produces_multiple_connections() {
    let stmts = parser::parse("A -> B -> C -> D\n").unwrap();
    assert_eq!(stmts.len(), 3);
    assert!(
        stmts
            .iter()
            .all(|s| matches!(s, Statement::Connection { .. }))
    );
}

// ── New syntax: @ header grouping ──────────────────────────────────────────

#[test]
fn header_grouping_works_like_bracket_grouping() {
    let bracket = parser::parse("[Backend] { API, DB }\n").unwrap();
    let header = parser::parse("@ Backend: API, DB\n").unwrap();
    assert_eq!(bracket, header);
}

// ── New syntax: -(label)> paren labeled ────────────────────────────────────

#[test]
fn paren_labeled_works_like_bracket_labeled() {
    let bracket = parser::parse("A -[sends]-> B\n").unwrap();
    let paren = parser::parse("A -(sends)> B\n").unwrap();
    assert_eq!(bracket, paren);
}

// ── All syntax types together ──────────────────────────────────────────────

#[test]
fn parse_all_syntax_types_including_new() {
    let input = "\
# comment
A -> B -> C
X -(label)> Y
@ Group: N1, N2
[Old] { N3, N4 }
User > App : Msg
Solo
";
    let stmts = parser::parse(input).unwrap();
    // comment(1) + chain(2 connections) + paren_label(1) + header_group(1) + bracket_group(1) + sequence(1) + node(1) = 8
    assert_eq!(stmts.len(), 8);
    assert!(matches!(&stmts[0], Statement::Comment(_)));
    assert!(matches!(&stmts[1], Statement::Connection { .. }));
    assert!(matches!(&stmts[2], Statement::Connection { .. }));
    assert!(matches!(&stmts[3], Statement::LabeledConnection { .. }));
    assert!(matches!(&stmts[4], Statement::Grouping { .. }));
    assert!(matches!(&stmts[5], Statement::Grouping { .. }));
    assert!(matches!(&stmts[6], Statement::Sequence { .. }));
    assert!(matches!(&stmts[7], Statement::Node(_)));
}
