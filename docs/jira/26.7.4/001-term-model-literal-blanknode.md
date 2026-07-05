# TICKET-001 — Extend `VarOrTerm`/`TermImpl` with Literal and BlankNode variants

**Status**: Not started
**Size**: L
**Depends on**: none (foundational)
**Blocks**: 002, 004, 005, 006, 007, 008, 009

## Problem

roxi's term model cannot represent RDF literals or blank nodes. `VarOrTerm` (`lib/src/triples.rs:4-8`) has only two variants:

```rust
pub enum VarOrTerm {
    Var(Variable),
    Term(TermImpl),
}
```

`TermImpl { iri: usize }` (`lib/src/triples.rs:79-81`) is just an interned integer handed out by the global `Encoder` (`lib/src/encoding.rs`) — there is no type-level distinction between an IRI, a string literal, a typed literal (`"5"^^xsd:integer`), a language-tagged literal (`"chat"@en`), or a blank node. Literal/BlankNode variants exist only as comments (`triples.rs:7-8`, `82-89`), never implemented.

This blocks:
- **SPARQL 1.1 conformance (006)**: many test-suite queries compare literal datatypes/langtags, which can't round-trip today.
- **SHACL (008)** and **ShEx (009)**: `sh:datatype`, `sh:nodeKind`, ShEx `NodeConstraint` value/datatype facets are meaningless without literal typing.
- **N3 (005)**: N3 syntax allows string/numeric/langtag/datatype literals and blank nodes (`_:x`, `[...]`) in rule bodies/heads — the current grammar can't parse them because the term model has nowhere to put them.

## Implementation approach

1. Introduce a `Term` enum with sibling variants (not a side-table — see rationale below):
   ```rust
   pub enum Term {
       Iri(TermImpl),
       Literal(LiteralImpl),
       BlankNode(BlankNodeImpl),
   }
   ```
   `VarOrTerm::Term(TermImpl)` becomes `VarOrTerm::Term(Term)`.
2. `LiteralImpl { value: usize, datatype: Option<usize>, lang: Option<usize> }` and `BlankNodeImpl { id: usize }`, both interned via `Encoder`.
3. Extend `Encoder` (`lib/src/encoding.rs`) so the global intern table tags each entry with its kind (IRI vs. literal lexical form vs. blank node label) — `add`/`get`/`decode` (encoding.rs:47, 60, 69) must round-trip the kind, not just the string.
4. Fix every exhaustive match on `VarOrTerm`/`TermImpl` that the compiler flags after the enum change — expected hit list: `tripleindex.rs` (SPO/POS/OSP indexing), `ruleindex.rs`, `queryengine.rs`, `sparql.rs` (`extract_query_plan`, `extract_expression`), `dred.rs`, `parser.rs`. Treat compiler errors as the authoritative checklist rather than grepping manually.

### Why sibling variants, not a side-table

A side-table keyed by the existing `usize` id (i.e., leave `VarOrTerm` untouched and look up "is this actually a literal?" in a separate map) avoids touching every match site, but permanently risks the two representations drifting out of sync — a correctness bug class with no compiler backstop. The sibling-enum approach is the one-time-cost, permanently-safe choice, and it structurally mirrors `oxrdf::Term` (`Iri`/`Literal`/`BlankNode`), which directly eases the adapter layer in TICKET-007.

## Definition of Done

- `lib/src/triples.rs`: `test_literal_term_roundtrip`, `test_blank_node_term_encoding`.
- `lib/src/encoding.rs`: `test_encoder_literal_vs_iri_distinct`, `test_literal_datatype_and_langtag_preserved`.
- Full `cargo test --workspace --lib --bins` passes with **zero regressions** against the existing 68-test baseline suite.

## Notes

This is the highest-blast-radius ticket in the plan — expect it to touch most files under `lib/src/`. Do it once, correctly, rather than letting later tickets (005/008/009) patch the term model independently and incompatibly.
