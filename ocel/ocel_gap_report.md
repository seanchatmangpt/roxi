# OCEL Gap Report

## Analysis Status

Audit performed: 2026-07-05

| Component | Status | Notes |
|-----------|--------|-------|
| Term model (Literals, Blank Nodes) | implemented | Typed literals and language-tagged literals supported via `VarOrTerm::new_literal`; blank nodes via `VarOrTerm::new_blank_node` |
| oxrdf adapter | implemented | `triple_index_to_oxrdf_graph` and `oxrdf_term_to_roxi_term` round-trip verified |
| SPARQL 1.1 evaluation | implemented | Full SPARQL 1.1 algebra; GROUP BY, HAVING, aggregates, property paths, subqueries |
| SPARQL 1.1 update | implemented | INSERT DATA, DELETE DATA, DELETE/INSERT WHERE |
| Datalog stratified negation | implemented | Stratum sort with cycle detection; unsafe rules rejected |
| Datalog aggregates | implemented | SUM, COUNT, MIN, MAX, AVG with GROUP BY grouping |
| N3 parsing | implemented | PEG grammar covers lists, quantifiers, quoted graphs, built-ins |
| SHACL core validation | implemented | MinCount, MaxCount, Datatype, Class, And, Or, Not, NodeKind, property paths |
| SHACL SPARQL-based constraints | implemented | sh:sparql constraint evaluation via embedded SPARQL engine |
| ShEx validation | implemented | Shape declarations, TripleConstraints, EachOf, OneOf, cardinality, recursive shapes via memo |
| Cycle-safe backward chaining | implemented | Visited-set guard in `eval_backward_inner`; stack-based fallback in CSprite |
| CI conformance gate | implemented | `.github/workflows/conformance.yml` runs all five test suites |

## Remaining Items Under Review

- SPARQL 1.1 conformance harness: W3C test suite manifest loading is integration-tested; full manifest-driven suite coverage is pending dataset download in CI.
- ShEx: Semantic actions (se:) are parsed but not yet evaluated; recorded as known limitation.
- N3 built-ins: `crypto:` and `os:` built-in families are stubbed with parse-only paths; out of scope for core implementation.