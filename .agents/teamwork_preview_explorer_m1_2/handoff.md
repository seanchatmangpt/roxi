# Handoff Report: Datalog Negation, Aggregates, and Rule Safety (TICKET-004)

## 1. Observation
- `Rule` struct is defined in `lib/src/triples.rs` at lines 166-169 as:
  ```rust
  pub struct Rule {
      pub body: Vec<Triple>,
      pub head: Triple,
  }
  ```
- `Reasoner::materialize` is defined in `lib/src/reasoner.rs` at lines 18-56. It processes triples using a monotonic forward-chaining fixpoint loop over `triple_index`.
- The Pest grammar file `lib/src/parser/n3.pest` defines rules at lines 87-89:
  ```pest
  Body = {"{"~TP+~"}"}
  Head = {"{"~WS*~TP~WS*~"}"}
  rule = { Body~WS*~"=>"~WS*~Head~WS*~"."? ~WS*}
  ```
- Accumulators are implemented in `lib/src/sparql.rs` at lines 470-613 (`CountAccumulator`, `SumAccumulator`, `MinAccumulator`, `MaxAccumulator`, `AvgAccumulator`, and `AccumulatorImpl`).
- The Datalog conformance tests in `lib/tests/datalog_negation.rs` and `lib/tests/datalog_conformance/` currently contain `todo!()` macros and are ignored or incomplete. For example, in `lib/tests/datalog_negation.rs:138`:
  ```rust
  todo!("Ensure stratification rejection returns a Datalog/Stratification error");
  ```

## 2. Logic Chain
- Since negation-as-failure requires assessing the *absence* of triples, rule bodies must hold polarity details. Thus, `Rule.body` must change from `Vec<Triple>` to `Vec<BodyLiteral>` where `BodyLiteral` has a `negated` boolean flag.
- Since cyclic negative dependencies (e.g., $A \leftarrow \neg B$ and $B \leftarrow A$) lead to infinite oscillation or ambiguous semantics, we must build a dependency graph of predicate identifiers to detect cycles containing negative/aggregate edges (unstratifiable rulesets).
- Since predicates can be class memberships (expressed via `rdf:type`), treating all `rdf:type` properties as a single predicate would lead to false dependency cycles. Therefore, class identifiers (the object of an `rdf:type` property) must serve as the predicate identifier when the predicate is `rdf:type`.
- In stratified negation, negated literals must only match against triples derived in strictly lower strata. Sequentially executing stratum fixpoint iterations and setting a query counter limit to the database size at the start of the stratum guarantees this isolation.
- To prevent duplicating aggregation arithmetic, the accumulators in `sparql.rs` must be moved to a shared module (`lib/src/aggregation.rs`) and reused during Datalog forward chaining by grouping rows of matches and running the accumulator.

## 3. Caveats
- We assume that variable names are encoded via the global `Encoder` and that we can map them back and forth safely using variable/term IDs.
- We assume that SHACL and ShEx (Milestone 4) or other rule systems do not rely on a flat `Vec<Triple>` body in `Rule`. Any other modules using `Rule` will need a minor adjustment to match the `BodyLiteral` shape.

## 4. Conclusion
We recommend modifying the `Rule` representation, implementing the SCC stratification algorithm, adapting `Reasoner::materialize` to evaluate strata sequentially, implementing compile-time safety checks, extracting the SPARQL aggregate accumulators to `lib/src/aggregation.rs`, and integrating them into Datalog rule head evaluation.

## 5. Verification Method
- **Implementation Inspection**: Confirm that `lib/src/aggregation.rs` contains the shared accumulator definitions and that both `lib/src/sparql.rs` and `lib/src/datalog.rs` import them.
- **Project Test Execution**: Run the conformance suite:
  ```sh
  cargo test --test datalog_negation
  cargo test --test datalog_conformance
  ```
  Ensure all tests compile and pass successfully, confirming that negation-as-failure, stratification rejection, safety checking, and aggregations function correctly.
