//! Integration test: verify the shipped examples file parses cleanly.

use diamem::dsl::dsl_to_mermaid;

const EXAMPLES: &str = include_str!("../docs/examples.dsl");

#[test]
fn examples_file_parses_without_errors() {
    let result = dsl_to_mermaid(EXAMPLES);
    assert!(
        result.is_ok(),
        "examples.dsl failed to parse:\n{}",
        result.unwrap_err()
    );
}

#[test]
fn examples_file_produces_nonempty_mermaid() {
    let mermaid = dsl_to_mermaid(EXAMPLES).unwrap();
    // Should have the header + many output lines
    assert!(
        mermaid.lines().count() > 20,
        "Expected substantial output from examples"
    );
}
