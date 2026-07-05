# Implementation Plan - TICKET-008: SHACL Validation

## Step 1: Implement `lib/src/shacl.rs`
Implement the SHACL validator in `lib/src/shacl.rs`. Expose:
- `ShapesGraph`
- `ShapesGraph::parse(shapes_str: &str) -> Result<Self, String>`
- `ValidationReport`
- `ValidationResult`
- `Validator::validate(data: &TripleIndex, shapes: &ShapesGraph) -> ValidationReport`
- Serialization of `ValidationReport` to `Vec<Triple>` using standard SHACL terms.

The validator will handle:
- Targets: `sh:targetClass`, `sh:targetNode`, `sh:targetSubjectsOf`, `sh:targetObjectsOf`, plus implicit class target.
- Core Constraints: `sh:property`, `sh:minCount`, `sh:maxCount`, `sh:datatype`, `sh:class`, `sh:pattern`, `sh:in`.
- Logical Components: `sh:and`, `sh:or`, `sh:not`.

## Step 2: Register the module in `lib/src/lib.rs`
Declare `pub mod shacl;` in `lib/src/lib.rs`.

## Step 3: Write Integration Tests in `lib/tests/shacl_validation.rs`
Implement the 5 required test cases:
- `test_min_max_count_violation`
- `test_datatype_constraint_pass_fail`
- `test_class_constraint`
- `test_and_or_not_logical_constraints`
- `test_conforms_true_for_valid_graph`

## Step 4: Update Conformance Runner and Manifest
Update `lib/tests/shacl_conformance/main.rs` to run the validator against `mock_suite/data.ttl` and `mock_suite/shapes.ttl`, then serialize the report and verify against `mock_suite/expected_report.ttl`. Write `docs/jira/26.7.4/manifests/shacl_manifest.md` detailing the pass rate.

## Step 5: Verify via Cargo Test
Run the test suites:
- `cargo test --workspace --test shacl_validation`
- `cargo test --workspace --test shacl_conformance`
