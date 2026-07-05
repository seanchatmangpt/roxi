# Scope: Datalog and N3 Dialects (Milestone 2)

## Architecture
- Negation-as-failure, stratified evaluation, and rule-safety verification for Datalog queries in `lib/src/datalog.rs`.
- Standard aggregates (`count`/`sum`/`min`/`max`/`avg`) over grouped variable bindings.
- Complete Notation3 (N3) parsing and reasoning (lists, blank nodes, multi-triple heads, quantifiers, quoted graphs/formulae, comments, and built-ins `log:`, `math:`, `list:`, `string:`).

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|---|---|---|---|
| 1 | TICKET-004: Datalog negation, aggregates, rule safety | Stratified evaluation and safety checks | none | PLANNED |
| 2 | TICKET-005: N3 full grammar and built-ins | Robust parsing, multi-triple heads, and standard built-ins | TICKET-004 | PLANNED |

## Interface Contracts
- Both Datalog and N3 rules share the updated `Rule` shape representing body literals with polarity (negation support).
