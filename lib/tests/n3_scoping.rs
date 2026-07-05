//! Tests probing two specific N3 characteristics flagged by a TBL-lens
//! re-audit: (1) whether nested quoted graphs (a formula containing another
//! formula) are handled without corrupting data or crashing, and (2) whether
//! forward chaining correctly threads a derived fact from one rule into a
//! second rule's antecedent within the same `materialize()` fixpoint
//! (chained implication). Both are additive probes over the existing
//! `log:implies`/quoted-graph implementation in `lib/src/reasoner.rs` and
//! `lib/src/triples.rs` -- see those files' doc comments for the underlying
//! mechanism (`FORMULA_REGISTRY`, `process_log_implies_rule`).

use minimal::TripleStore;

fn decode_all(triples: &[minimal::triples::Triple]) -> Vec<String> {
    triples.iter().map(|t| TripleStore::decode_triple(t)).collect()
}

/// `materialize()` only returns newly-*derived* triples, not the base facts
/// already present in the store -- so checking whether a base fact was
/// preserved (as opposed to newly derived) requires reading the store's
/// index directly rather than relying on `materialize()`'s return value.
fn decode_all_stored_facts(store: &TripleStore) -> Vec<String> {
    (0..store.len())
        .filter_map(|i| store.triple_index.get(i))
        .map(TripleStore::decode_triple)
        .collect()
}

/// A quoted graph containing, as one of its own triples' objects, ANOTHER
/// quoted graph (two levels of nesting) must parse and materialize without
/// error, and the inner formula's triples must stay quoted -- i.e. they must
/// NOT leak into the plain asserted-fact set at any level. This guards
/// against a naive quoted-graph parser that only handles one level of `{ }`
/// nesting correctly and either flattens or drops the inner formula.
#[test]
fn test_nested_quoted_formula_does_not_leak_into_facts() {
    let data = "@prefix : <http://example.org/> .\n\
                \n\
                :carol a :Honest .\n\
                :alice :believes { :bob :says { :carol a :Dishonest } } .\n";

    let mut store = TripleStore::from(data);
    let _inferred = store.materialize();
    let decoded = decode_all_stored_facts(&store);

    // The nested claim ":carol a :Dishonest" is buried two formula-levels
    // deep inside :alice's belief about what :bob says -- it must never
    // surface as a plain top-level fact (that would mean nesting leaked and
    // :carol is now contradictorily both Honest and Dishonest at top level).
    assert!(
        !decoded.iter().any(|d| d.contains("/carol") && d.contains("Dishonest")),
        "the nested quoted claim ':carol a :Dishonest' must stay quoted inside two levels of formula nesting, not leak into top-level facts. Stored facts: {:?}",
        decoded
    );

    // The genuinely top-level, unquoted fact must still be present and
    // unaffected by the nested formula's contradictory content.
    assert!(
        decoded.iter().any(|d| d.contains("/carol") && d.contains("Honest") && !d.contains("Dishonest")),
        "the real top-level fact ':carol a :Honest' must be preserved. Stored facts: {:?}",
        decoded
    );
}

/// Two rules where the first rule's derived output is exactly the pattern
/// the second rule's antecedent requires, within a single `materialize()`
/// call (multi-step forward chaining, not manually re-invoking the
/// reasoner). This is distinct from the existing deep_taxonomy transitivity
/// tests (which chain the SAME rule against itself repeatedly); here two
/// DIFFERENT rules must correctly compose: rule 1 promotes a fact, rule 2
/// consumes exactly that promoted fact to derive a further one.
#[test]
fn test_chained_implication_across_two_distinct_rules() {
    let data = "@prefix : <http://example.org/> .\n\
                \n\
                :fido a :Dog .\n\
                \n\
                { ?x a :Dog } => { ?x a :Mammal }.\n\
                { ?x a :Mammal } => { ?x a :Animal }.\n";

    let mut store = TripleStore::from(data);
    let inferred = store.materialize();
    let decoded = decode_all(&inferred);

    assert!(
        decoded.iter().any(|d| d.contains("/fido") && d.contains("Mammal")),
        "rule 1 (Dog => Mammal) must fire directly off the base fact. Derived: {:?}",
        decoded
    );
    assert!(
        decoded.iter().any(|d| d.contains("/fido") && d.contains("Animal")),
        "rule 2 (Mammal => Animal) must fire off rule 1's OUTPUT within the same materialize() pass, not just the base fact. Derived: {:?}",
        decoded
    );
}

/// Chained implication combined with `log:implies`: a dynamically-reified
/// rule's derived output must itself be able to feed a second, ordinary
/// rule's antecedent in the same pass -- confirming `log:implies`'s dynamic
/// reification composes with normal forward chaining rather than only
/// working in isolation.
#[test]
fn test_chained_implication_through_log_implies_then_ordinary_rule() {
    let data = "@prefix : <http://example.org/> .\n\
                @prefix log: <http://www.w3.org/2000/10/swap/log#> .\n\
                \n\
                :bob a :GoodCitizen .\n\
                :alice :says { ?citizen a :GoodCitizen } .\n\
                { ?speaker :says ?formula . ?formula log:implies { ?citizen a :TaxPayer } } => { ?citizen a :TaxPayer }.\n\
                { ?p a :TaxPayer } => { ?p :owes :Taxes }.\n";

    let mut store = TripleStore::from(data);
    let inferred = store.materialize();
    let decoded = decode_all(&inferred);

    assert!(
        decoded.iter().any(|d| d.contains("/bob") && d.contains("TaxPayer")),
        "expected :bob a :TaxPayer via log:implies. Derived: {:?}",
        decoded
    );
    assert!(
        decoded.iter().any(|d| d.contains("/bob") && d.contains("owes") && d.contains("Taxes")),
        "expected :bob :owes :Taxes to be derived from the log:implies-produced TaxPayer fact feeding the second ordinary rule. Derived: {:?}",
        decoded
    );
}
