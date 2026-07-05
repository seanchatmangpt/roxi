# Handoff Report: TICKET-004 (Datalog negation, aggregates, rule safety)

## 1. Observation
- `Rule` struct in `lib/src/triples.rs:166-169`:
  ```rust
  pub struct Rule {
      pub body: Vec<Triple>,
      pub head: Triple,
  }
  ```
- Current forward chaining fixpoint logic in `lib/src/reasoner.rs:18-56` evaluates all rules concurrently using `RuleIndex::find_match` which indexes all body triples, without separating them by stratum or polarity.
- `lib/src/parser/n3rule_parser.rs:99-135` parses rules from N3 format using the `n3.pest` grammar but does not support `not` syntax.
- SPARQL accumulators are defined in `lib/src/sparql.rs:475-613`, including `CountAccumulator`, `SumAccumulator`, `MinAccumulator`, `MaxAccumulator`, `AvgAccumulator`, and `AccumulatorImpl`. They depend on `Encoder` and `Utils::remove_literal_tags`.
- Compilation of the workspace currently fails due to `oxrdf_adapter.rs:93-114`:
  ```rust
  #[cfg(feature = "rdf-12")]
  oxrdf::Term::Triple(_) => panic!("RDF-star Triple terms are not supported by roxi"),
  ```
  with the error:
  `error[E0004]: non-exhaustive patterns: &oxrdf::Term::Triple(_) not covered`
  because the `rdf-12` feature is active in the dependency `oxrdf` but not defined in the `roxi` crate.
- Standard Datalog conformance stubs are already written in `lib/tests/datalog_negation.rs` and `lib/tests/datalog_conformance/` but use `todo!()` placeholders, which crash/fail at test execution.

## 2. Logic Chain
- To implement negation-as-failure (NAF), we must distinguish between positive and negated body literals. This leads to changing `Rule.body` from `Vec<Triple>` to `Vec<BodyLiteral>` where `BodyLiteral` holds a `negated` boolean flag.
- To execute negated literals without infinite loops or unsoundness, we must ensure stratification. A predicate dependency graph (where negative/aggregate edges have weight 1 and positive edges have weight 0) allows us to compute stratum levels. Cycles of weight-1 edges indicate unstratifiable rulesets and must be rejected at rule-load time.
- During forward chaining, evaluating stratum $S$ can only depend on negated literals from strata $< S$. These lower strata are fully evaluated and static, so we can check negated patterns against the current state of `triple_index` without triggering rules when new facts matching those negated patterns are added. Thus, only positive body literals should be indexed in `RuleIndex`.
- Rule safety prevents infinite groundings. We check that (1) every head variable is bound in a positive body literal, and (2) negated/aggregate literals have their non-local variables bound. We handle local existential variables in negation by ensuring that for each negated literal, at least one variable is bound (or it contains only constants), preventing unbound global queries.
- SPARQL aggregators are functionally identical to Datalog aggregators. Moving them to `lib/src/aggregation.rs` and exposing them as a shared module allows the Datalog engine to reuse the existing parsing/accumulator types without duplicating code.

## 3. Caveats
- Did not investigate backward chaining cycle safety changes (which is part of Milestone 1 / TICKET-002, not TICKET-004).
- Assumed standard Datalog stratification is sufficient for all tests (which is verified by the stubs).
- The syntax for aggregates in the Pest grammar is not yet fully defined or tested; stubs in `aggregations.rs` construct the aggregate structs programmatically.

## 4. Conclusion
We have mapped out the precise changes needed for TICKET-004. The implementer must:
1. Update `Rule` and parsing code to support `BodyLiteral` with negation.
2. Fix the compilation bug in `oxrdf_adapter.rs` by removing the `#[cfg(feature = "rdf-12")]` attribute.
3. Implement the stratification solver and stratified fixpoint evaluation loop in `reasoner.rs`/`datalog.rs`.
4. Implement rule safety checks in `ruleindex.rs` or `datalog.rs`.
5. Refactor SPARQL accumulators to `aggregation.rs` and implement Datalog grouping/aggregation in rule head derivation.
6. Replace `todo!` placeholders in the conformance test suite.

## 5. Verification Method
- Execute the build command:
  `cargo check --tests --features datalog` (which should now succeed once the match arm fix is applied).
- Run the test suite:
  `cargo test --workspace --lib --bins datalog`
- Verify that all stubs in `lib/tests/datalog_negation.rs` and `lib/tests/datalog_conformance/` pass without panic or error.
