# Handoff Report — SPARQL 1.1 Conformance Spike (TICKET-006)

This handoff details the findings, logic, conclusions, and recommended verification methods for the next agent working on TICKET-006.

---

## 1. Observation

### 1.1 SPARQL Engine Code Gaps (`lib/src/sparql.rs`)
- **Unsupported Graph Patterns**:
  In `extract_query_plan` (lines 97-173), unhandled `GraphPattern` variants fall through to:
  ```rust
  _ => PlanNode::Done,
  ```
  This skips execution for `LeftJoin`, `Union`, `Minus`, `OrderBy`, `Distinct`, `Reduced`, `Slice`, `Values`, `Path`, and `Service`.
- **Cross-Join Panic**:
  In `PlanNode::Join` evaluation (lines 280-327), the code does:
  ```rust
  let intersection: Vec<usize> = right_bindings
      .iter()
      .filter(|b| left_vars.contains(&b.var))
      .map(|b| b.var)
      .collect();
  // ...
  left.into_iter().for_each(|bindings| {
      for binding in &bindings {
          if binding.var == *intersection.get(0).unwrap() {
              // ...
  ```
  If there are no intersecting variables, `intersection` is empty. Accessing `get(0).unwrap()` causes a panic during evaluation.
- **Incorrect Multi-Variable Joins**:
  The join logic only matches on the first shared variable:
  ```rust
  if binding.var == *intersection.get(0).unwrap() {
  ```
  It ignores any subsequent variables in the `intersection` vector, producing incorrect bindings that fail to validate constraints on other shared variables.
- **Aggregate Projection Limitation**:
  In `PlanNode::Aggregate` evaluation (line 346):
  ```rust
  let (aggregate_function, aggregate_var) = aggregates.iter().next().unwrap();
  ```
  This extracts only the first aggregate function. If a query contains multiple aggregate functions, all subsequent aggregates are silently ignored. If `aggregates` is empty (e.g. a plain `GROUP BY`), it panics on `unwrap()`.
- **Variable Prefix Encoding Mismatches**:
  - In `spargebra::Expression::Variable::into_string()`, the returned string starts with `?` (e.g., `"?var"`).
  - In `Project` and `VarOrTerm::convert`, variables are added to the `Encoder` without `?` (e.g., `"var"`).
  - In `PlanNode::Extend` evaluation:
    ```rust
    let encoded_from = Encoder::get(from.as_str()).unwrap();
    let encoded_to = Encoder::get(to.as_str()).unwrap();
    ```
    `from.as_str()` returns the variable name without `?`, but it was added with `?` in `extract_query_plan`, leading to `Encoder::get` lookup mismatch.

### 1.2 Test Execution Behavior
- Running:
  ```bash
  cargo test --workspace --lib --bins sparql11_conformance
  ```
  Results in cargo compiling the libraries and binaries but running `0 tests` because integration tests are not included under the `--lib --bins` filter.
- Running:
  ```bash
  cargo test --test sparql11_conformance
  ```
  Executes the integration test correctly:
  ```
  running 1 test
  test test_sparql11_conformance ... ok
  ```

---

## 2. Logic Chain

1. **Test Runner Correctness**: Since the W3C SPARQL 1.1 Query Evaluation suite does not exist in cached form anywhere in `/Users/sac` (as verified by `find_by_name` searches), we must build our own data-driven suite.
2. **Current Harness Inadequacy**: The current integration test runner in `lib/tests/sparql11_conformance/main.rs` performs hardcoded checks and ignores the expected JSON results file. Hence, it cannot scale to test multiple features. We must replace it with a generic runner that deserializes expected bindings from JSON and compares sets of results.
3. **Panic & Correctness Risks**:
   - The occurrence of `.unwrap()` on `intersection.get(0)` inside the Join execution loop directly causes a panic if the patterns share no variables (cross join).
   - Only checking the first intersection variable (`intersection.get(0)`) leads to incorrect join logic when queries join on multiple variables.
   - Using `aggregates.iter().next().unwrap()` directly restricts the engine to evaluating only one aggregate function and panicking on empty group-by projections.
4. **Variable Lookup Failure**: Since some parts of the code register variable IDs using the `?` prefix and others query or register them without it, lookups via `Encoder::get` return mismatched indices, preventing variable matching or causing panics inside `Extend` nodes.
5. **Implementation Necessity**: Therefore, to achieve W3C SPARQL 1.1 Query Evaluation conformance, the implementer must:
   - Introduce a JSON manifest and a generic test runner (prototyped in `/Users/sac/roxi/.agents/explorer_sparql11_conformance_1/`).
   - Fix variable naming inconsistency in `Encoder`.
   - Update Join and Aggregate evaluation algorithms in `lib/src/sparql.rs`.
   - Add new `PlanNode` variants for `LeftJoin`, `Union`, `Minus`, and map them in `evaluate_plan` using outer join, flat map, and anti-join iterators.

---

## 3. Caveats

- We assumed that `spargebra` 0.4.6 successfully parses all SPARQL 1.1 syntax. We did not investigate whether `spargebra` has any parsing bugs, but it is a standard community library, so we assume its parsing is correct.
- We did not evaluate the performance/complexity of the Cartesian product and Anti-Join iterators; the implementation should focus on correctness first as required by the conformance milestone.

---

## 4. Conclusion

The `roxi` SPARQL engine is not conformant with SPARQL 1.1 query evaluation. It has major gaps (unsupported LeftJoin, Union, Minus, OrderBy, Slice, etc.) and core bugs (panics on cross joins, multi-variable join failures, single aggregate limitations, and variable encoding conflicts). 

Achieving conformance requires implementing the generic runner, resolving the core bugs in `sparql.rs`, and extending the evaluator to support outer joins (`LeftJoin`), unions (`Union`), anti-joins (`Minus`), and logical/arithmetic expressions in `Filter` and `Extend` nodes.

---

## 5. Verification Method

1. **Verify Current Integration Test**:
   Run the following command to verify that the existing mock test compiles and passes:
   ```bash
   cargo test --test sparql11_conformance
   ```
2. **Verify Proposed Artifacts**:
   Inspect the proposed JSON manifest and test runner in the explorer's directory:
   - Manifest: `/Users/sac/roxi/.agents/explorer_sparql11_conformance_1/proposed_manifest.json`
   - Test Runner: `/Users/sac/roxi/.agents/explorer_sparql11_conformance_1/proposed_test_runner.rs`
3. **Invalidation Conditions**:
   If the implementer moves the proposed files and runs `cargo test --test sparql11_conformance`, it should fail because the engine does not support LeftJoin, Union, Minus, or multiple aggregates yet. The test suite will only pass once all gaps highlighted in the `analysis.md` are resolved.
