# TICKET-008 — SHACL: validation + W3C `data-shapes` conformance suite

**Status**: Not started
**Size**: XL
**Depends on**: 001, 007

## Problem

Zero SHACL support exists anywhere in the repo (confirmed: no mentions of "shacl" in any `.rs`/`Cargo.toml`/markdown file). SHACL needs a shapes-graph parser and a validator covering `sh:targetClass`, `sh:property`, `sh:minCount`/`sh:maxCount`, `sh:datatype`, `sh:class`, `sh:pattern`, `sh:in`, and logical composition (`sh:and`/`sh:or`/`sh:not`), producing a conformance report. Per the v26.7.4 zero-gaps commitment, this must pass the official **W3C `data-shapes` SHACL test suite** in full, not just a hand-picked constraint sample.

## Implementation approach

1. New module `lib/src/shacl.rs` exposing: `ShapesGraph::parse(...)`, `Validator::validate(data: &TripleIndex, shapes: &ShapesGraph) -> ValidationReport`.
2. Parse shapes graphs via `shacl_ast` (shapes are themselves RDF, so this likely reuses the existing Turtle-parsing path through `rio_turtle`/`Parser` for the raw triples, with `shacl_ast` interpreting the shapes vocabulary on top).
3. Run validation via `shacl_validation` against the `oxrdf::Graph` produced by TICKET-007's adapter.
4. Translate the resulting `ValidationReport` back into a roxi-native report type that references original `Triple`s (via the adapter's reverse direction) so violations point at real graph data, not foreign term handles.
5. Vendor the W3C `data-shapes` SHACL test suite (core constraint components + SPARQL-based constraint components) into `lib/tests/shacl_conformance/`. Run it end-to-end. Where `shacl_validation` doesn't cover a constraint component the suite exercises, close the gap in `lib/src/shacl.rs` (e.g. a thin wrapper implementing the missing component against `TripleIndex::query_help`) rather than skipping the test.

## Definition of Done

- `lib/tests/shacl_validation.rs`: `test_min_max_count_violation`, `test_datatype_constraint_pass_fail`, `test_class_constraint`, `test_and_or_not_logical_constraints`, `test_conforms_true_for_valid_graph`.
- `lib/tests/shacl_conformance/`: the full vendored W3C suite running end-to-end, with a pass-rate manifest. Target 100%, or every excluded test explicitly spec-justified in the manifest.
- `cargo test --workspace --lib --bins shacl` passes.
