# SPARQL 1.1 Engine

## W3C SPARQL Query Engine

Roxi includes a fully compliant SPARQL 1.1 Query and Update evaluator in [lib/src/sparql.rs](file:///Users/sac/roxi/lib/src/sparql.rs).

---

## Subsystems

1. **Algebra Compiler**: Translates query strings into SPARQL algebra trees.
2. **Optimizer**: Optimizes plan trees to minimize index lookups.
3. **Execution Engine**: Evaluates query plans against the `TripleIndex`.
