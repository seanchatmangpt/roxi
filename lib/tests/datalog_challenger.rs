
use minimal::triples::{BodyLiteral, Rule, Triple, VarOrTerm, Aggregate, AggregateFunction};
use minimal::TripleStore;

/// Test empty relations in negation.
#[test]
fn test_empty_relations_negation() {
    let mut store = TripleStore::new();

    // Fact: :a is a Parent. But there are no hasChild relations at all.
    store.add(Triple::from(
        "http://example.org/a".to_string(),
        "http://example.org/type".to_string(),
        "http://example.org/Parent".to_string(),
    ));

    // Rule: if ?x is a Parent and ?x has no child ?y, ?x is Childless.
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

    let res = store.add_rules(vec![rule]);
    assert!(res.is_ok());

    let derived = store.materialize();
    let childless: Vec<String> = derived
        .iter()
        .filter(|t| {
            let s = TripleStore::decode_triple(t);
            s.contains("Childless")
        })
        .map(|t| TripleStore::decode_triple(t))
        .collect();

    assert_eq!(childless.len(), 1);
    assert!(childless[0].contains("http://example.org/a"));
}

/// Test empty relations in aggregations.
#[test]
fn test_empty_relations_aggregation() {
    let mut store = TripleStore::new();

    // No facts at all in the store.
    let rule = Rule {
        head: Triple::from(
            "?d".to_string(),
            "http://example.org/employeeCount".to_string(),
            "?count".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?d".to_string(),
                    "http://example.org/hasEmployee".to_string(),
                    "?e".to_string(),
                ),
            }
        ],
    };

    let agg = Aggregate {
        function: AggregateFunction::Count,
        source_var: "?e".to_string(),
        target_var: "?count".to_string(),
        group_vars: vec!["?d".to_string()],
    };

    let res = store.add_rule_with_aggregate(rule, agg);
    assert!(res.is_ok());

    let derived = store.materialize();
    assert_eq!(derived.len(), 0);
}

/// Test unbound aggregate source variable.
#[test]
fn test_unbound_aggregate_source_var() {
    let mut store = TripleStore::new();

    store.add(Triple::from(
        "http://example.org/d1".to_string(),
        "http://example.org/hasEmployee".to_string(),
        "http://example.org/e1".to_string(),
    ));

    // Rule where ?unbound is NOT bound in the body.
    let rule = Rule {
        head: Triple::from(
            "?d".to_string(),
            "http://example.org/employeeCount".to_string(),
            "?count".to_string(),
        ),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from(
                    "?d".to_string(),
                    "http://example.org/hasEmployee".to_string(),
                    "?e".to_string(),
                ),
            }
        ],
    };

    let agg = Aggregate {
        function: AggregateFunction::Count,
        source_var: "?unbound".to_string(),
        target_var: "?count".to_string(),
        group_vars: vec!["?d".to_string()],
    };

    let res = store.add_rule_with_aggregate(rule, agg);
    if res.is_ok() {
        let derived = store.materialize();
        // Since ?unbound is not in the bindings, it shouldn't produce any count fact.
        assert_eq!(derived.len(), 0);
    }
}

/// Test boundary numeric inputs in aggregations.
#[test]
fn test_boundary_numeric_inputs_aggregation() {
    let mut store = TripleStore::new();

    // Facts with negative numbers, floats, large numbers, and invalid numbers.
    store.add(Triple::from("http://example.org/d1".to_string(), "http://example.org/hasEmployee".to_string(), "http://example.org/e1".to_string()));
    store.add(Triple::from("http://example.org/e1".to_string(), "http://example.org/salary".to_string(), "-1000.50".to_string()));

    store.add(Triple::from("http://example.org/d1".to_string(), "http://example.org/hasEmployee".to_string(), "http://example.org/e2".to_string()));
    store.add(Triple::from("http://example.org/e2".to_string(), "http://example.org/salary".to_string(), "2000.75".to_string()));

    store.add(Triple::from("http://example.org/d1".to_string(), "http://example.org/hasEmployee".to_string(), "http://example.org/e3".to_string()));
    store.add(Triple::from("http://example.org/e3".to_string(), "http://example.org/salary".to_string(), "invalid_number".to_string()));

    // Sum rule: { ?d :hasEmployee ?e . ?e :salary ?s } => { ?d :totalSalary sum(?s) }
    let rule_sum = Rule {
        head: Triple::from("?d".to_string(), "http://example.org/totalSalary".to_string(), "?total".to_string()),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from("?d".to_string(), "http://example.org/hasEmployee".to_string(), "?e".to_string()),
            },
            BodyLiteral {
                negated: false,
                pattern: Triple::from("?e".to_string(), "http://example.org/salary".to_string(), "?s".to_string()),
            }
        ],
    };
    let agg_sum = Aggregate {
        function: AggregateFunction::Sum,
        source_var: "?s".to_string(),
        target_var: "?total".to_string(),
        group_vars: vec!["?d".to_string()],
    };

    // Min rule
    let rule_min = Rule {
        head: Triple::from("?d".to_string(), "http://example.org/minSalary".to_string(), "?min".to_string()),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from("?d".to_string(), "http://example.org/hasEmployee".to_string(), "?e".to_string()),
            },
            BodyLiteral {
                negated: false,
                pattern: Triple::from("?e".to_string(), "http://example.org/salary".to_string(), "?s".to_string()),
            }
        ],
    };
    let agg_min = Aggregate {
        function: AggregateFunction::Min,
        source_var: "?s".to_string(),
        target_var: "?min".to_string(),
        group_vars: vec!["?d".to_string()],
    };

    // Max rule
    let rule_max = Rule {
        head: Triple::from("?d".to_string(), "http://example.org/maxSalary".to_string(), "?max".to_string()),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from("?d".to_string(), "http://example.org/hasEmployee".to_string(), "?e".to_string()),
            },
            BodyLiteral {
                negated: false,
                pattern: Triple::from("?e".to_string(), "http://example.org/salary".to_string(), "?s".to_string()),
            }
        ],
    };
    let agg_max = Aggregate {
        function: AggregateFunction::Max,
        source_var: "?s".to_string(),
        target_var: "?max".to_string(),
        group_vars: vec!["?d".to_string()],
    };

    // Avg rule
    let rule_avg = Rule {
        head: Triple::from("?d".to_string(), "http://example.org/avgSalary".to_string(), "?avg".to_string()),
        body: vec![
            BodyLiteral {
                negated: false,
                pattern: Triple::from("?d".to_string(), "http://example.org/hasEmployee".to_string(), "?e".to_string()),
            },
            BodyLiteral {
                negated: false,
                pattern: Triple::from("?e".to_string(), "http://example.org/salary".to_string(), "?s".to_string()),
            }
        ],
    };
    let agg_avg = Aggregate {
        function: AggregateFunction::Avg,
        source_var: "?s".to_string(),
        target_var: "?avg".to_string(),
        group_vars: vec!["?d".to_string()],
    };

    assert!(store.add_rule_with_aggregate(rule_sum, agg_sum).is_ok());
    assert!(store.add_rule_with_aggregate(rule_min, agg_min).is_ok());
    assert!(store.add_rule_with_aggregate(rule_max, agg_max).is_ok());
    assert!(store.add_rule_with_aggregate(rule_avg, agg_avg).is_ok());

    let derived = store.materialize();

    let mut sum_val = 0.0;
    let mut min_val = 0.0;
    let mut max_val = 0.0;
    let mut avg_val = 0.0;

    for t in derived.iter() {
        let s = TripleStore::decode_triple(t);
        if s.contains("totalSalary") {
            let parts: Vec<&str> = s.split_whitespace().collect();
            sum_val = parts[2].replace("\"", "").parse::<f64>().unwrap();
        } else if s.contains("minSalary") {
            let parts: Vec<&str> = s.split_whitespace().collect();
            min_val = parts[2].replace("\"", "").parse::<f64>().unwrap();
        } else if s.contains("maxSalary") {
            let parts: Vec<&str> = s.split_whitespace().collect();
            max_val = parts[2].replace("\"", "").parse::<f64>().unwrap();
        } else if s.contains("avgSalary") {
            let parts: Vec<&str> = s.split_whitespace().collect();
            avg_val = parts[2].replace("\"", "").parse::<f64>().unwrap();
        }
    }

    assert!((sum_val - 1000.25).abs() < 1e-5, "Sum should be 1000.25 but got {}", sum_val);
    assert!((min_val - -1000.50).abs() < 1e-5, "Min should be -1000.50 but got {}", min_val);
    assert!((max_val - 2000.75).abs() < 1e-5, "Max should be 2000.75 but got {}", max_val);
    assert!((avg_val - 500.125).abs() < 1e-5, "Avg should be 500.125 but got {}", avg_val);
}
