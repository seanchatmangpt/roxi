# Project: roxi RDF Engine Enhancements

## Architecture
- **Term Model & Encoding (`lib/src/triples.rs`, `lib/src/encoding.rs`)**: Sibling enum variants for `Term` (Iri, Literal, BlankNode) instead of a flat side-table, and type-safe encoder/decoder.
- **Rule Engines (`lib/src/backwardchaining.rs`, `lib/src/csprite.rs`, `lib/src/reasoner.rs`)**: Cycle-safe backward chain and CsSprite with visited-set recursion prevention. Stratified negation-as-failure fixpoint iteration for Datalog.
- **Language Parsers (`lib/src/parser/`)**: Robust Pest-based parsers for N3 rules, handling multi-triple heads, lists, blank nodes, and quoted graphs.
- **SPARQL 1.1 Engine (`lib/src/sparql.rs`)**: Complete query evaluation/update, aggregate accumulators extracted to a shared space.
- **Adapter Layer (`lib/src/oxrdf_adapter.rs`)**: Direct conversion between roxi native `Term`/`Triple` and `oxrdf` equivalent types.
- **Shape Validation (`lib/src/shacl.rs`, `lib/src/shex.rs`/`lib/src/shex_native.rs`)**: Fully native validators (no external `shacl_validation`/`shex_validation`/`shex_ast`/`rudof` crates) operating directly on `TripleIndex`. ShEx was originally a thin wrapper around external `shex_ast`/`shex_validation` crates; fuzzing found a real spec violation in their `OneOf` handling, so it was rewritten native (see `shex_native.rs`'s module doc). SHACL was native from the start.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | Term Model and Cycle Safety | TICKET-001, TICKET-002, TICKET-003 | none | DONE |
| 2 | Datalog and N3 Dialects | TICKET-004, TICKET-005 | M1 | DONE |
| 3 | SPARQL 1.1 Conformance | TICKET-006 | M1 | DONE |
| 4 | Shape Validation (SHACL & ShEx) | TICKET-007, TICKET-008, TICKET-009 | M1 | DONE |
| 5 | Continuous Conformance CI Gate | TICKET-010 | M2, M3, M4 | PLANNED |

## Interface Contracts
### `triples.rs` ↔ `encoding.rs`
- `Term` must represent IRIs, Literals (lexical value, datatype, language tag), and Blank Nodes.
- `Encoder` must encode each term type and preserve all type-level and value-level properties on roundtrip.

### `shacl.rs` / `shex.rs` (`shex_native.rs`)
- Both operate directly on `TripleIndex` (no `oxrdf` conversion step) and return roxi-native `ValidationReport`/`ShexValidationReport` structs.
- `TripleStore::validate_shacl(shapes_turtle)` / `TripleStore::validate_shex(schema_json, shape_map)` are the public entry points (`lib/src/lib.rs`).
