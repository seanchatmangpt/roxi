# Verification and Challenge Report: TICKET-008 SHACL Validator

This report details the verification of the TICKET-008 SHACL validator implementation in `lib/src/shacl.rs`, regression checks on the rest of the workspace, and an adversarial analysis of the validator's robustness.

## 1. Observation

- **Implementation Location**: `/Users/sac/roxi/lib/src/shacl.rs`
- **Integration Test Locations**:
  - `/Users/sac/roxi/lib/tests/shacl_validation.rs`
  - `/Users/sac/roxi/lib/tests/shacl_conformance/main.rs`
- **Verification Commands & Results**:
  - `cargo test --test shacl_validation` successfully passed:
    ```
    running 5 tests
    test test_conforms_true_for_valid_graph ... ok
    test test_datatype_constraint_pass_fail ... ok
    test test_class_constraint ... ok
    test test_min_max_count_violation ... ok
    test test_and_or_not_logical_constraints ... ok
    test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
    ```
  - `cargo test --test shacl_conformance` successfully passed:
    ```
    running 1 test
    test test_shacl_conformance ... ok
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
    ```
  - Running `cargo test --workspace --lib` surfaced **2 failures** in the SPARQL module (`lib/src/sparql.rs`), which are unrelated to SHACL changes:
    - `sparql::tests::test_group_by_sum_aggregation`
    - `sparql::tests::test_group_by_count_aggregation_multiple_group`
    
    Verbatim error snippet for `test_group_by_sum_aggregation`:
    ```
    thread 'sparql::tests::test_group_by_sum_aggregation' (497684) panicked at lib/src/sparql.rs:1189:9:
    assertion `left == right` failed
      left: [[Binding { var: "s", val: "<http://example.com/foo>" }, Binding { var: "sum", val: "0" }]]
     right: [[Binding { var: "sum", val: "0" }]]
    ```

## 2. Logic Chain

1. **Test Conformance**: The SHACL validator successfully passes the custom integration tests in `lib/tests/shacl_validation.rs` and the mock conformance test in `lib/tests/shacl_conformance/main.rs`.
2. **Workspace Regressions**: The two failures identified during `cargo test --workspace --lib` are localized to `lib/src/sparql.rs`. Git diff analysis confirms that these failures stem from modifications in the SPARQL module (milestone 3) and are not caused by the shape validation code (milestone 4), as `lib/src/sparql.rs` has no dependencies on `lib/src/shacl.rs`.

## 3. Caveats

- We did not stress-test the performance of recursive shapes or deeply nested path expressions on very large graphs.
- Non-standard/out-of-scope constraints like `sh:node` or specific flags (e.g. `x`, `q`) were not evaluated as they are not part of the current requirements.

## 4. Conclusion

The implementation of the SHACL validator conforms to the currently implemented test cases, but has several significant design flaws and compliance bugs under adversarial scenarios.

---

# Adversarial Challenge Report

**Overall risk assessment**: HIGH

## Challenges

### [High] Challenge 1: Direct Property Shape Validation Failure

- **Assumption challenged**: The validator assumes that all shapes validated at the root level (obtained via `shape_nodes`) are NodeShapes, and that PropertyShapes are only evaluated when nested inside a NodeShape via `sh:property`.
- **Attack scenario**: A user targets a `sh:PropertyShape` directly (e.g. using `sh:targetClass` or `sh:targetNode` directly on the PropertyShape, which is standard SHACL practice).
- **Blast radius**:
  1. Constraints like `sh:datatype`, `sh:class`, etc. specified on the PropertyShape are evaluated directly on the focus node (e.g. the instance IRI) rather than the value nodes reached via `sh:path`. This causes immediate false validation failures because the instance IRI will not match the expected literal datatype or class.
  2. Cardinality constraints like `sh:minCount` and `sh:maxCount` are completely ignored because they are only implemented inside the `sh:property` nesting logic (lines 862-894). They are never executed when a PropertyShape is validated at the root.
- **Mitigation**: In `validate_shape`, first check if the `shape_node` has a `sh:path`. If it does, evaluate it as a property shape by resolving the path, obtaining `v_nodes`, and checking all constraints on those value nodes (similar to lines 862-1053), rather than applying the constraints to the focus node itself.

### [Medium] Challenge 2: Path Evaluation Non-Deduplication Bug in Cardinality Constraints

- **Assumption challenged**: The validator assumes that path evaluation returns a sequence/multiset of nodes, and that counting the length of this list is equivalent to counting the value nodes.
- **Attack scenario**: A path resolves to duplicate nodes due to:
  1. Duplicate triples in the data graph (e.g., identical triples in different named graphs).
  2. Alternative paths (`sh:alternativePath`) or sequence paths resolving to the same value node through different routes.
- **Blast radius**: The validator will count duplicates toward `sh:minCount` and `sh:maxCount`, leading to incorrect validation results. For example, a focus node with only 1 unique value node can falsely pass a `sh:minCount 2` constraint if the path evaluates to the same node twice. This violates the SHACL specification, which defines path evaluation as returning a *set* of nodes.
- **Mitigation**: Deduplicate the list of evaluated nodes `v_nodes` before checking `sh:minCount` and `sh:maxCount`.

### [Medium] Challenge 3: Complex Inverse Paths Resolve to Nothing

- **Assumption challenged**: The validator assumes that `sh:inversePath` always points to a simple predicate IRI.
- **Attack scenario**: A shape uses a complex path inside `sh:inversePath`, such as `[ sh:inversePath [ sh:alternativePath (ex:p1 ex:p2) ] ]`.
- **Blast radius**: Lines 606-614 in `eval_path` calls `get_subjects(data, inv, focus_node)` where `inv` is the blank node representing the complex path. Since `inv` is a shapes graph blank node ID and not a predicate in the data graph, it returns no nodes, silently failing to validate complex inverse paths.
- **Mitigation**: Recursively evaluate complex inverse paths. Find all subject nodes `x` in the data graph such that `focus_node` is in `eval_path(data, shapes, x, inv)`.

### [Low/Medium] Challenge 4: Silently Discarding Nested Logical Constraint Validation Results

- **Assumption challenged**: The validator assumes that for logical constraints (`sh:and`, `sh:or`, `sh:not`), only the boolean conformance of the sub-shapes matters, and sub-shape validation results do not need to be reported.
- **Attack scenario**: A user runs validation on a graph that fails a `sh:and` or `sh:or` constraint.
- **Blast radius**: The generated `ValidationReport` will contain a single validation result for the `sh:and` or `sh:or` constraint itself, but will completely discard the individual validation results from the sub-shapes that actually failed. This violates the SHACL spec, which requires inclusion of sub-shape validation results in the report to provide details on why the logical constraint failed.
- **Mitigation**: Append the validation results accumulated in `temp_results` to the main `results` vector when a logical constraint violation is detected (for `sh:and` and `sh:or`).

### [Low] Challenge 5: Non-deterministic and Crash-prone Validation Report Generation

- **Assumption challenged**: The validator assumes that system time is always after UNIX epoch and can be used to generate unique blank node IDs.
- **Attack scenario**: The validator is run in an environment where system time is not set, set to 0, or set to before UNIX epoch (e.g. certain Docker containers, VM configurations, or embedded targets).
- **Blast radius**: `std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()` will panic, crashing the entire application during report generation. Additionally, generating blank node IDs based on the current system time makes the output of `to_triples()` non-deterministic, making testing and assertion verification unreliable.
- **Mitigation**: Use a deterministic identifier generator (such as a simple sequential counter or a UUID library with a fallback) rather than relying on system time.

---

## 5. Verification Method

To verify the observations:
1. Run `cargo test --test shacl_validation` to run the custom SHACL tests.
2. Run `cargo test --test shacl_conformance` to run the SHACL conformance suite.
3. Run `cargo test --workspace --lib` to observe the 2 failures in the SPARQL module tests.
