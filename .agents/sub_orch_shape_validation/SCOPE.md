# Scope: Shape Validation (SHACL & ShEx) (Milestone 4)

## Architecture
- Adapter layer `lib/src/oxrdf_adapter.rs` to convert between roxi native `Term`/`Triple` and `oxrdf` equivalents.
- SHACL validator `lib/src/shacl.rs` using `shacl_validation` / `shacl_ast` crates.
- ShEx validator `lib/src/shex.rs` using `shex_validation` / `shex_ast` crates.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | TICKET-007: oxrdf adapter layer | Translation between TripleIndex and oxrdf Graph | none | DONE |
| 2 | TICKET-008: SHACL validation | W3C data-shapes validation and test suite | TICKET-007 | DONE |
| 3 | TICKET-009: ShEx validation | shexTest validation and test suite | TICKET-007 | DONE |

## Interface Contracts
### `oxrdf_adapter.rs` ↔ `shacl.rs` / `shex.rs`
- `oxrdf_adapter` converts `TripleIndex` to `oxrdf::Graph`.
- `shacl` / `shex` run validation on the converted graph and translate validation results back to roxi-native `Triple`-based reports.
