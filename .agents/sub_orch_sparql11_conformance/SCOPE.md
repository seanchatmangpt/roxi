# Scope: SPARQL 1.1 Conformance (Milestone 3)

## Architecture
- SPARQL 1.1 Query and Update engine in `lib/src/sparql.rs`.
- Extract aggregate accumulators to shared module (`lib/src/aggregation.rs`).

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | TICKET-006: SPARQL 1.1 Conformance | W3C sparql11-test-suite integration and gap closure | none | PLANNED |

## Interface Contracts
- None external to SPARQL engine, but requires Term model (TICKET-001) to compare literal datatypes/langtags correctly.
