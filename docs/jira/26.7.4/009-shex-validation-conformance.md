# TICKET-009 — ShEx: validation + `shexTest` conformance suite

**Status**: Not started
**Size**: XL
**Depends on**: 001, 007
**Benefits from** (not blocked by): 008 landing first, for shared adapter-usage patterns

## Problem

Zero ShEx support exists anywhere in the repo. ShEx's shape-expression model (`NodeConstraint`, `EachOf`/`OneOf`, recursive shape references, shape maps) is structurally distinct from SHACL's independent-constraint model, so it gets its own module rather than being folded into TICKET-008. Per the v26.7.4 zero-gaps commitment, this must pass the **`shexTest`** suite (shex.io/shex-test) in full.

## Implementation approach

1. New module `lib/src/shex.rs`: `Schema::parse(...)`, `ShapeExpr` (recursive enum: `NodeConstraint | Shape | ShapeAnd | ShapeOr | ShapeNot | ShapeRef`), `Validator::validate(data: &TripleIndex, schema: &Schema, shape_map: &ShapeMap) -> ValidationResult`.
2. Parse ShEx schemas via `shex_ast` — prefer **ShExJ** (JSON serialization) over ShExC initially, since it avoids authoring a second Turtle-like pest grammar (trading input-format ergonomics for materially less new parsing code); revisit ShExC support as a follow-up if consumers need the compact syntax.
3. Validate via `shex_validation` against the `oxrdf::Graph` produced by TICKET-007's adapter (same adapter as TICKET-008 — factor any genuinely shared plumbing, e.g. datatype/facet constraint checks on literals, into a small shared module rather than duplicating).
4. Translate results back into a roxi-native report referencing original `Triple`s.
5. Vendor the `shexTest` suite (schemas + validation manifests + positive/negative syntax and semantic tests) into `lib/tests/shex_conformance/`, run end-to-end, close any gap `shex_validation` doesn't cover in `lib/src/shex.rs`.

## Definition of Done

- `lib/tests/shex_validation.rs`: `test_node_constraint_datatype`, `test_each_of_shape`, `test_cardinality_on_triple_constraint`, `test_shape_ref_recursive`, `test_shape_map_pass_fail`.
- `lib/tests/shex_conformance/`: the full vendored `shexTest` suite running end-to-end, with a pass-rate manifest. Target 100%, or every excluded test explicitly spec-justified in the manifest.
- `cargo test --workspace --lib --bins shex` passes.
