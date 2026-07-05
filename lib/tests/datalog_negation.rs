
use minimal::triples::{BodyLiteral, Rule, Triple, VarOrTerm};
use minimal::TripleStore;

/// TICKET-004: Test basic stratified negation.
/// Negation-as-failure should be correctly evaluated across stratum boundaries.
///
/// Rules under test:
/// { ?x <http://example.org/type> <http://example.org/Parent> . not { ?x <http://example.org/hasChild> ?y } } => { ?x <http://example.org/type> <http://example.org/Childless> }
///
/// Facts:
/// :a <http://example.org/type> <http://example.org/Parent> .
/// :b <http://example.org/type> <http://example.org/Parent> .
/// :a <http://example.org/hasChild> :b .
///
/// Expected outcome:
/// :b is derived as Childless, but :a is not.
#[test]
fn test_stratified_negation_basic() {
    let mut store = TripleStore::new();

    // Add facts
    store.add(Triple::from(
        "http://example.org/a".to_string(),
        "http://example.org/type".to_string(),
        "http://example.org/Parent".to_string(),
    ));
    store.add(Triple::from(
        "http://example.org/b".to_string(),
        "http://example.org/type".to_string(),
        "http://example.org/Parent".to_string(),
    ));
    store.add(Triple::from(
        "http://example.org/a".to_string(),
        "http://example.org/hasChild".to_string(),
        "http://example.org/b".to_string(),
    ));

    // Define rule with negated body literal
    let rule = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/Childless".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/Parent".to_string(),
                ),
            },
            BodyLiteral {
                negated: true,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/hasChild".to_string(),
                    "?y".to_string(),
                ),
            },
        ],
    };

    store.add_rules(vec![rule]);

    // Materialize derivations
    let derived = store.materialize();

    // Verify results
    let childless_triples: Vec<String> = derived
        .iter()
        .filter(|t| {
            let decoded_p = TripleStore::decode_triple(t);
            decoded_p.contains("Childless")
        })
        .map(|t| TripleStore::decode_triple(t))
        .collect();

    // Expecting exactly one Childless derivation: :b
    assert_eq!(
        childless_triples.len(),
        1,
        "There should be exactly one childless parent derived"
    );
    assert!(
        childless_triples[0].contains("http://example.org/b"),
        "http://example.org/b should be Childless"
    );
    assert!(
        !childless_triples[0].contains("http://example.org/a"),
        "http://example.org/a has a child, so it should not be Childless"
    );
}

/// TICKET-004: Test that unstratifiable rulesets are rejected rather than leading to hang or incorrect results.
///
/// Rules under test (self-negation cycle):
/// { ?x <http://example.org/type> <http://example.org/A> . not { ?x <http://example.org/type> <http://example.org/B> } } => { ?x <http://example.org/type> <http://example.org/B> }
#[test]
fn test_unstratifiable_rules_rejected() {
    let mut store = TripleStore::new();

    let rule = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/B".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/A".to_string(),
                ),
            },
            BodyLiteral {
                negated: true,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/B".to_string(),
                ),
            },
        ],
    };

    let res = store.add_rules(vec![rule]);
    assert!(res.is_err());
}

/// TICKET-004: Test that the rule safety check rejects rules with unbound variables in negated literals.
///
/// Rules under test (unsafe):
/// { not { ?x <http://example.org/type> <http://example.org/A> } } => { ?x <http://example.org/type> <http://example.org/B> }
#[test]
fn test_rule_safety_check_rejects_unbound_negated_var() {
    let mut store = TripleStore::new();

    let unsafe_rule = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/B".to_string(),
        ),
        body: vec![BodyLiteral {
            negated: true,
            pattern: Triple::from(
                "?x".to_string(),
                "http://example.org/type".to_string(),
                "http://example.org/A".to_string(),
            ),
        }],
    };

    // Rule safety check must reject this because ?x is only in a negated body literal.
    let res = store.add_rules(vec![unsafe_rule]);
    assert!(res.is_err());
}

/// TICKET-004: Test that evaluation terminates when recursive rulesets are processed.
///
/// Rules under test:
/// { ?x <http://example.org/type> <http://example.org/Node> } => { ?x <http://example.org/reachable> ?x }
/// { ?x <http://example.org/edge> ?y . ?y <http://example.org/reachable> ?z } => { ?x <http://example.org/reachable> ?z }
/// { ?x <http://example.org/type> <http://example.org/Node> . not { ?x <http://example.org/reachable> <http://example.org/c> } } => { ?x <http://example.org/type> <http://example.org/Disconnected> }
///
/// Facts:
/// :a <http://example.org/type> <http://example.org/Node> .
/// :b <http://example.org/type> <http://example.org/Node> .
/// :c <http://example.org/type> <http://example.org/Node> .
/// :a <http://example.org/edge> :b .
/// :b <http://example.org/edge> :c .
///
/// Expected outcome:
/// - Reachability computes: :a -> :a, :b -> :b, :c -> :c, :a -> :b, :b -> :c, :a -> :c
/// - Disconnectedness computes: Only :c is disconnected from :c? No, :c reaches :c, so nobody is disconnected from :c except if we have another node :d.
/// Let's add :d <http://example.org/type> <http://example.org/Node> (isolated).
/// - :d should be derived as Disconnected from :c.
#[test]
fn test_fixpoint_terminates_on_recursive_ruleset() {
    let mut store = TripleStore::new();

    // Facts
    store.add(Triple::from(
        "http://example.org/a".to_string(),
        "http://example.org/type".to_string(),
        "http://example.org/Node".to_string(),
    ));
    store.add(Triple::from(
        "http://example.org/b".to_string(),
        "http://example.org/type".to_string(),
        "http://example.org/Node".to_string(),
    ));
    store.add(Triple::from(
        "http://example.org/c".to_string(),
        "http://example.org/type".to_string(),
        "http://example.org/Node".to_string(),
    ));
    store.add(Triple::from(
        "http://example.org/d".to_string(),
        "http://example.org/type".to_string(),
        "http://example.org/Node".to_string(),
    ));
    store.add(Triple::from(
        "http://example.org/a".to_string(),
        "http://example.org/edge".to_string(),
        "http://example.org/b".to_string(),
    ));
    store.add(Triple::from(
        "http://example.org/b".to_string(),
        "http://example.org/edge".to_string(),
        "http://example.org/c".to_string(),
    ));

    // Rules
    let r1 = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/reachable".to_string(),
            "?x".to_string(),
        ),
        body: vec![BodyLiteral {
            negated: false,
            pattern: Triple::from(
                "?x".to_string(),
                "http://example.org/type".to_string(),
                "http://example.org/Node".to_string(),
            ),
        }],
    };

    let r2 = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/reachable".to_string(),
            "?z".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/edge".to_string(),
                    "?y".to_string(),
                ),
            },
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?y".to_string(),
                    "http://example.org/reachable".to_string(),
                    "?z".to_string(),
                ),
            },
        ],
    };

    let r3 = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/Disconnected".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/Node".to_string(),
                ),
            },
            BodyLiteral {
                negated: true,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/reachable".to_string(),
                    "http://example.org/c".to_string(),
                ),
            },
        ],
    };

    store.add_rules(vec![r1, r2, r3]);

    let derived = store.materialize();

    let disconnected: Vec<String> = derived
        .iter()
        .filter(|t| {
            let s = TripleStore::decode_triple(t);
            s.contains("Disconnected")
        })
        .map(|t| TripleStore::decode_triple(t))
        .collect();

    // Verify that ONLY :d is disconnected from :c
    assert_eq!(
        disconnected.len(),
        1,
        "There should be exactly one disconnected node"
    );
    assert!(
        disconnected[0].contains("http://example.org/d"),
        "Node d should be disconnected"
    );
}

#[test]
fn test_negation_empty_relations() {
    let mut store = TripleStore::new();
    let rule = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/C".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/A".to_string(),
                ),
            },
            BodyLiteral {
                negated: true,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/B".to_string(),
                ),
            },
        ],
    };
    assert!(store.add_rules(vec![rule]).is_ok());
    let derived = store.materialize();
    assert!(derived.is_empty(), "Derived set should be empty on empty relations");
}

#[test]
fn test_negation_unbound_vars_rejected() {
    let mut store = TripleStore::new();
    let rule = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/C".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/A".to_string(),
                ),
            },
            BodyLiteral {
                negated: true,
                pattern: Triple::from(
                    "?y".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/B".to_string(),
                ),
            },
        ],
    };
    // ?y is unbound by any positive body literal, should be rejected
    assert!(store.add_rules(vec![rule]).is_err());
}

#[test]
fn test_empty_body_rule() {
    let mut store = TripleStore::new();
    let rule = Rule {
        head: Triple::from(
            "http://example.org/a".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/C".to_string(),
        ),
        body: vec![],
    };
    assert!(store.add_rules(vec![rule]).is_ok());
    let derived = store.materialize();
    assert_eq!(derived.len(), 1);
    let s = TripleStore::decode_triple(&derived[0]);
    assert!(s.contains("http://example.org/a") && s.contains("C"));
}

#[test]
fn test_long_unstratifiable_cycle_rejected() {
    let mut store = TripleStore::new();
    // Rule 1: A and not B => C
    let r1 = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/C".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/A".to_string(),
                ),
            },
            BodyLiteral {
                negated: true,
                pattern: Triple::from(
                    "?x".to_string(),
                    "http://example.org/type".to_string(),
                    "http://example.org/B".to_string(),
                ),
            },
        ],
    };
    // Rule 2: C => B
    let r2 = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/B".to_string(),
        ),
        body: vec![BodyLiteral {
            negated: false,
            pattern: Triple::from(
                "?x".to_string(),
                "http://example.org/type".to_string(),
                "http://example.org/C".to_string(),
            ),
        }],
    };
    // Rule 3: B => A
    let r3 = Rule {
        head: Triple::from(
            "?x".to_string(),
            "http://example.org/type".to_string(),
            "http://example.org/A".to_string(),
        ),
        body: vec![BodyLiteral {
            negated: false,
            pattern: Triple::from(
                "?x".to_string(),
                "http://example.org/type".to_string(),
                "http://example.org/B".to_string(),
            ),
        }],
    };
    // This forms a cycle B -> A -> C -> B, where A -> C is negative.
    // So C depends negatively on B, and B depends positively on C.
    // Thus it is unstratifiable.
    assert!(store.add_rules(vec![r1, r2, r3]).is_err());
}

