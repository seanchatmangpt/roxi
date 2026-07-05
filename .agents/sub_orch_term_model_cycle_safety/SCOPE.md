# Scope: Term Model and Cycle Safety (Milestone 1)

## Architecture
- Sibling variants for `Term` representing IRI, Literal, and BlankNode.
- Global intern table (`Encoder`) supporting tagged kinds (IRI, lexical form, blank node label).
- Cycle guards in `backwardchaining.rs` and `csprite.rs` using visited-set tracking.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | TICKET-001: Term Model Literal/BlankNode | Introduce Term enum, extend Encoder and update matching sites | none | DONE |
| 2 | TICKET-002: Production Cycle Guard | Thread history through backwardchaining | TICKET-001 | DONE |
| 3 | TICKET-003: Csprite Cycle Guards | Thread history through csprite TODOs | TICKET-002 | DONE |

## Interface Contracts
### `triples.rs` ↔ `encoding.rs`
- `Term` represents IRIs, Literals, and Blank Nodes.
- `Encoder` must encode each term type and preserve all type-level and value-level properties on roundtrip.
