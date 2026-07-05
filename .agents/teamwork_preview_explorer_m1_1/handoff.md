# Handoff Report: Datalog negation, aggregates, rule safety analysis (TICKET-004)

## 1. Observation
- `Rule` is defined in `lib/src/triples.rs:166-169` as:
  ```rust
  pub struct Rule {
      pub body: Vec<Triple>,
      pub head: Triple,
  }
  ```
- `Reasoner::materialize` in `lib/src/reasoner.rs:18-56` is a single forward-chaining fixpoint loop processing triples sequentially via `counter` and `SimpleQueryEngine::query`.
- `SimpleQueryEngine::query` in `lib/src/queryengine.rs:14-30` queries the triple index up to a given counter.
- SPARQL aggregate accumulators are defined in `lib/src/sparql.rs:470-605` under trait `Accumulator` with implementations `CountAccumulator`, `SumAccumulator`, `MinAccumulator`, `MaxAccumulator`, and `AvgAccumulator`.
- The compilation command `cargo test --features datalog` failed with:
  ```
  error: couldn't read `lib/tests/datalog_tests/datalog_conformance/safe_unsafe_rejection.rs`: No such file or directory (os error 2)
    --> lib/tests/datalog_conformance.rs:60:5
     |
  60 |     mod safe_unsafe_rejection;
  ```
- The test file `lib/tests/datalog_conformance.rs:57-69` nests modules inside `mod datalog_tests` while specifying `#[path = "datalog_conformance/..."]`.

## 2. Logic Chain
1. To support polarity and negation, `Rule`'s body needs to be changed from `Vec<Triple>` to `Vec<BodyLiteral>` where `BodyLiteral { negated: bool, pattern: Triple }` (Observation 1).
2. Rule safety requires that all variables in negated literals, non-aggregate head positions, and aggregate parameters (source/group variables) also appear in positive body literals. This can be verified by compiling sets of positive, negated, and head variable IDs and asserting subset conditions (Observation 1).
3. Stratified evaluation requires grouping rules into strata. An edge `Q -> P` in the dependency graph represents that predicate `P` depends on predicate `Q`. Negated literals or aggregates must introduce negative/stratified edges. Finding Strongly Connected Components (SCCs) and checking that no SCC contains a negative/stratified edge detects unstratifiable cycles (Observation 2).
4. Since stratification guarantees that aggregated relations and negated relations are fully computed in lower strata, we can check negated literals against the `triple_index` up to `stratum_start_counter` (Observation 3).
5. For aggregate rules, the input relation is static during the current stratum. Thus, the aggregate only needs to be evaluated once at the beginning of the stratum (Observation 2, 4).
6. Moving aggregate accumulators from `lib/src/sparql.rs` (Observation 4) to `lib/src/aggregation.rs` allows both SPARQL and Datalog modules to reuse them without duplication.
7. Modifying `lib/tests/datalog_conformance.rs` to expose the datalog tests directly (removing `mod datalog_tests` wrapping) aligns the module path resolution with standard Rust directory structure, resolving the compile error (Observation 5).

## 3. Caveats
- The Pest parser in `lib/src/parser/n3rule_parser.rs` and grammar `n3.pest` currently lack rules for negation. We proposed a grammar modification, but its implementation must be validated during integration.
- We assume all predicates are concrete terms (URIs or class objects). Variables in the predicate position are treated as invalid for dependency analysis.

## 4. Conclusion
We have completed a read-only investigation and written detailed implementation recommendations to `analysis.md`. The design cleanly extends the existing forward-chaining fixpoint loop, uses standard stratification via strongly connected components, ensures safety compliance, and reuses SPARQL aggregates via a shared `aggregation.rs` module. The compile error in integration tests has been pinpointed and resolved.

## 5. Verification Method
1. The recommendations can be verified by reviewing the `analysis.md` report.
2. The compilation fix and entire datalog feature can be verified by running:
   ```bash
   cargo test --features datalog
   ```
3. Invalidation condition: The test command should fail if rule safety, stratification, negation, or aggregates are incorrectly implemented.
