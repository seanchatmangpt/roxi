//! Stress and counterfactual tests for the SHACL validator, going beyond the
//! W3C-suite-driven conformance checks to probe behavior at scale and check
//! specifically for over-permissive validation (false conformance).

use minimal::shacl::{ShapesGraph, Validator};
use minimal::parser::{Parser, Syntax};
use minimal::tripleindex::TripleIndex;
use std::time::{Duration, Instant};

fn build_data_index(data_str: &str) -> TripleIndex {
    let triples = Parser::parse_triples(data_str, Syntax::Turtle).unwrap();
    let mut index = TripleIndex::new();
    for t in triples {
        index.add(t);
    }
    index
}

/// Counterfactual: an `sh:and` shape combining a constraint that's always
/// satisfiable with one that's never satisfiable by any real value (a
/// contradictory conjunction: the value must simultaneously equal ex:X via
/// sh:hasValue AND belong to sh:in a disjoint value set). No conforming data
/// can exist. An over-permissive validator that only checks
/// sub-constraints independently (e.g. short-circuiting on the first
/// satisfied branch instead of requiring ALL branches) would incorrectly
/// report conformance; a correct one reports a violation for every focus
/// node hitting this shape.
#[test]
fn test_counterfactual_contradictory_and_never_conforms() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:ImpossibleShape a sh:NodeShape ;
            sh:targetClass ex:Widget ;
            sh:and (
                [ sh:property [ sh:path ex:status ; sh:hasValue ex:Active ] ]
                [ sh:property [ sh:path ex:status ; sh:in ( ex:Inactive ex:Archived ) ] ]
            ) .
    "#;
    let shapes = ShapesGraph::parse(shapes_str).unwrap();

    // Try every value that could conceivably be argued to satisfy one
    // branch or the other -- none should conform, since sh:and requires ALL.
    for status in ["ex:Active", "ex:Inactive", "ex:Archived", "ex:Unknown"] {
        let data_str = format!(
            r#"
            @prefix ex: <http://example.org/> .
            ex:w1 a ex:Widget ; ex:status {status} .
        "#
        );
        let data = build_data_index(&data_str);
        let report = Validator::validate(&data, &shapes);
        assert!(
            !report.conforms,
            "ex:status={} must NOT conform to a contradictory sh:and (hasValue ex:Active AND in (Inactive,Archived)), but validator reported conformance",
            status
        );
    }
}

/// Counterfactual: `sh:not` wrapping a shape that's itself never satisfiable
/// (an empty `sh:in` list -- no value can be a member of an empty set) means
/// `sh:not` should ALWAYS hold (negating "never true" gives "always true").
/// This specifically stresses that `sh:not`'s "does the inner shape conform"
/// check is followed correctly even at the edges (an inner shape that always
/// fails), rather than e.g. an implementation that treats an empty sh:in
/// list as vacuously true by accident.
#[test]
fn test_counterfactual_not_of_unsatisfiable_always_conforms() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:NeverInEmptySetShape a sh:NodeShape ;
            sh:in () .

        ex:AlwaysConformsShape a sh:NodeShape ;
            sh:targetClass ex:Thing ;
            sh:not ex:NeverInEmptySetShape .
    "#;
    let shapes = ShapesGraph::parse(shapes_str).unwrap();

    let data = build_data_index(
        r#"
        @prefix ex: <http://example.org/> .
        ex:anything a ex:Thing .
    "#,
    );
    let report = Validator::validate(&data, &shapes);
    assert!(
        report.conforms,
        "sh:not of an always-unsatisfiable inner shape (empty sh:in) must always conform, got violations: {:?}",
        report.results
    );
}

/// Counterfactual: `sh:xone` (exactly one) with a focus node whose value
/// satisfies BOTH candidate branches must be REJECTED (xone requires
/// exactly one match, not "at least one" like sh:or). This specifically
/// catches an implementation that conflates xone with or.
#[test]
fn test_counterfactual_xone_rejects_multiple_matches() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:XoneShape a sh:NodeShape ;
            sh:targetClass ex:Item ;
            sh:xone (
                [ sh:property [ sh:path ex:value ; sh:minInclusive 0 ] ]
                [ sh:property [ sh:path ex:value ; sh:maxInclusive 100 ] ]
            ) .
    "#;
    let shapes = ShapesGraph::parse(shapes_str).unwrap();

    // value=50 satisfies BOTH branches (>=0 AND <=100) -- xone must reject this.
    let data_both = build_data_index(
        r#"
        @prefix ex: <http://example.org/> .
        ex:i1 a ex:Item ; ex:value 50 .
    "#,
    );
    let report_both = Validator::validate(&data_both, &shapes);
    assert!(
        !report_both.conforms,
        "value satisfying BOTH xone branches must be rejected (xone means exactly one, not at least one)"
    );

    // value=-10 satisfies ONLY the maxInclusive branch (not minInclusive) -- xone must accept this.
    let data_one = build_data_index(
        r#"
        @prefix ex: <http://example.org/> .
        ex:i2 a ex:Item ; ex:value -10 .
    "#,
    );
    let report_one = Validator::validate(&data_one, &shapes);
    assert!(
        report_one.conforms,
        "value satisfying EXACTLY ONE xone branch must conform, got violations: {:?}",
        report_one.results
    );
}

/// Stress: validate 1000 focus nodes (half conforming, half violating)
/// against a shape with several constraint components combined, and check
/// both correctness (exactly the right half is flagged) and that it
/// completes in a practical time budget.
#[test]
fn test_large_scale_validation_1000_focus_nodes() {
    const N: usize = 1000;
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:property [
                sh:path ex:age ;
                sh:minInclusive 0 ;
                sh:maxInclusive 150 ;
                sh:minCount 1 ;
                sh:maxCount 1 ;
            ] .
    "#;
    let shapes = ShapesGraph::parse(shapes_str).unwrap();

    let mut data_str = String::from("@prefix ex: <http://example.org/> .\n");
    for i in 0..N {
        // Even-indexed people get a valid age; odd-indexed get an out-of-range one.
        let age = if i % 2 == 0 { (i % 120) as i64 } else { -5 };
        data_str.push_str(&format!("ex:person{} a ex:Person ; ex:age {} .\n", i, age));
    }
    let data = build_data_index(&data_str);

    let start = Instant::now();
    let report = Validator::validate(&data, &shapes);
    let elapsed = start.elapsed();

    assert!(!report.conforms, "half of 1000 focus nodes have an invalid age and must produce violations");

    let violated_persons: std::collections::HashSet<String> = report
        .results
        .iter()
        .map(|r| r.focus_node.to_string())
        .filter(|s| s.contains("person"))
        .collect();

    // Exactly the odd-indexed (N/2) people should have violations -- not
    // more (false positives on valid data) and not fewer (false negatives
    // missing real violations).
    assert_eq!(
        violated_persons.len(), N / 2,
        "expected exactly {} violating focus nodes (the odd-indexed ones with age=-5), got {}",
        N / 2, violated_persons.len()
    );

    assert!(
        elapsed < Duration::from_secs(15),
        "validating 1000 focus nodes took {:?}, expected well under 15s",
        elapsed
    );
}
