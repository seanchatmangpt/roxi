# TICKET-004 — Datalog: stratified negation, aggregates, rule safety, and full conformance suite

**Status**: Not started
**Size**: XL
**Depends on**: 001, 002
**Coordinate with**: 005 (both change `Rule`'s shape in `triples.rs` — land this ticket first or in the same review window to avoid conflicting struct edits)

## Problem

There is no dedicated Datalog engine and no negation-as-failure, stratification, or aggregate support anywhere in the workspace (confirmed zero hits for `datalog`/`crepe`/`ascent`/`differential-dataflow` in every `Cargo.toml`). `Rule { body: Vec<Triple>, head: Triple }` (`lib/src/triples.rs:123-127`) has no polarity concept — every body literal is implicitly positive.

`Reasoner::materialize` (`lib/src/reasoner.rs:16-44`) is a working forward-chaining fixpoint loop — it grows `triple_index` and reprocesses via a counter (lines 19-36), and recursive/transitive closure is genuinely tested (`lib/src/pipeline.rs:64`, `lib/src/imars_reasoner.rs:63`). This ticket extends that engine rather than forking a new one, per the project's design decision to reuse working, tested machinery instead of duplicating it.

**Scope commitment**: per the v26.7.4 goal of zero gaps, this ticket must cover full standard-core Datalog semantics — safe rules, stratified negation, recursion, and aggregates — verified by a comprehensive hand-authored conformance suite (there is no single official Datalog spec/test-suite to defer to, unlike SPARQL/SHACL/ShEx).

## Implementation approach

1. **Body polarity**: change `Rule.body: Vec<Triple>` to `Vec<BodyLiteral>` where `BodyLiteral { negated: bool, pattern: Triple }`. Update every construction site: `lib/src/parser/n3rule_parser.rs`, `lib/src/ruleindex.rs`.
2. **Stratification**: new module `lib/src/datalog.rs`. Build a predicate dependency graph from `RuleIndex` (edge = "predicate P depends on predicate Q in rule body", tagged positive/negative). Compute strata via standard stratification (topological ordering over the positive+negative dependency graph); **reject** (return an error, not a panic or silent misevaluation) any rule set where a predicate depends negatively on itself through a cycle (unstratifiable).
3. **Stratified evaluation**: modify `Reasoner::materialize` (`reasoner.rs:16-44`) to evaluate strata in ascending order, reusing the existing per-stratum fixpoint loop; when evaluating a negated body literal, check it against the **already-fixed** `triple_index` contents from strictly lower strata (never the current or higher stratum).
4. **Rule safety**: at rule-load time in `ruleindex.rs`, reject rules where a variable appears in a negated body literal or in the head but not in any positive body literal (the standard Datalog safety condition — prevents infinite/undefined groundings).
5. **Aggregates**: extend/expose `count`/`sum`/`min`/`max`/`avg` over grouped variable bindings for Datalog rule heads. Reuse `AccumulatorImpl`/`PlanAggregationFunction`/`CountAccumulator`/`SumAccumulator`/`MinAccumulator`/`MaxAccumulator`/`AvgAccumulator` already implemented in `lib/src/sparql.rs` — extract them into a shared module (e.g. `lib/src/aggregation.rs`) used by both `sparql.rs` and the new `datalog.rs`, rather than duplicating the accumulator logic.
6. **Conformance suite**: author `lib/tests/datalog_conformance/` covering, exhaustively:
   - safe rules accepted; unsafe rules rejected (each unsafety pattern: unbound var in negated literal, unbound var in head)
   - linear recursion (transitive closure — already covered elsewhere, re-assert here for completeness)
   - mutual recursion (two predicates each depending on the other)
   - negation across a single stratum boundary (accepted)
   - negation through a cycle (rejected, with a clear error, not a hang)
   - every aggregate function combined with grouping and with recursive input relations
   Include a manifest comment block at the top of the test module cross-referencing which standard Datalog semantic each test covers, so "100% coverage" is auditable rather than asserted.

## Definition of Done

- `lib/tests/datalog_negation.rs`: `test_stratified_negation_basic`, `test_unstratifiable_rules_rejected`, `test_rule_safety_check_rejects_unbound_negated_var`, `test_fixpoint_terminates_on_recursive_ruleset`.
- `lib/tests/datalog_conformance/*.rs`: one test per semantic case enumerated above, zero `#[ignore]`d.
- `cargo test --workspace --lib --bins datalog` passes.
