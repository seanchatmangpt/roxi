# TICKET-007 — `oxrdf` adapter layer (`TripleIndex` ↔ `oxrdf::Graph`/`Term`)

**Status**: Not started
**Size**: M
**Depends on**: 001
**Blocks**: 008, 009

## Problem

TICKET-008 (SHACL) and TICKET-009 (ShEx) adopt the `rudof`/shex-rs ecosystem crates (`shacl_ast`, `shacl_validation`, `shex_ast`, `shex_validation`) rather than hand-rolling spec-conformant validators — this mirrors how roxi already delegates SPARQL parsing to `spargebra` and RDF parsing to `rio_turtle`/`rio_api` instead of owning those grammars itself. Those crates are built on `oxrdf::Term`/`NamedNode`/`Literal`/`BlankNode`, which is structurally close to (but not identical to) roxi's own `Term` model post-TICKET-001. Something has to translate between them.

## Implementation approach

1. **Spike first**: confirm the current crates.io state and API surface of `shacl_ast`, `shacl_validation`, `shex_ast`, `shex_validation` before writing integration code — the research backing this plan cited these from general ecosystem knowledge, not a live crates.io check. Document actual versions and API shapes found. If the ecosystem has shifted (crates renamed/archived/API-incompatible), document the fallback decision (a minimal hand-rolled core-constraints validator) explicitly here rather than silently changing scope in TICKET-008/009.
2. Add `oxrdf` as a **direct** dependency in `lib/Cargo.toml` (it's already present transitively via `spargebra`/`sparesults` at 0.3.3, so this is a low-risk promotion, not a new dependency).
3. New module `lib/src/oxrdf_adapter.rs`:
   - `TripleIndex` → `oxrdf::Graph` (or an in-memory `oxrdf`-compatible dataset): iterate all triples, translate each `Term::Iri`/`Term::Literal`/`Term::BlankNode` (post-001) into the corresponding `oxrdf::NamedNode`/`Literal`/`BlankNode`, preserving datatype and language tag exactly.
   - Reverse direction: `oxrdf::Term` → roxi's `Term`/`Encoder` representation, so validation violations can be reported against original `Triple`s rather than a foreign term type.

## Definition of Done

- `lib/tests/oxrdf_adapter.rs`: `test_triple_index_to_oxrdf_graph_roundtrip`, `test_literal_datatype_langtag_preserved_across_adapter`, `test_blank_node_identity_preserved`.
- `cargo test --workspace --lib --bins oxrdf_adapter` passes.
