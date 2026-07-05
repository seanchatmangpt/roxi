# Advanced Datalog Engine

## Datalog in Roxi

Datalog is a declarative logic programming language that extends standard database queries with recursive rules. In Roxi, the Datalog engine is built directly on the shared `TripleIndex` and `Reasoner` infrastructure.

---

## Architectural Comparison

| Feature | Standard SQL | Roxi Datalog | Notation3 (N3) |
| :--- | :--- | :--- | :--- |
| **Recursion** | Complex (CTE) | Native & Optimized | Native |
| **Negation** | `NOT EXISTS` | Stratified Negation | Quoted Graph Negation |
| **Safety Check** | N/A | Enforced at Parse Time | N/A |
| **Aggregates** | Standard | Grouped (Head) | Procedural Built-ins |

---

## Key Subsystems

1. **Stratification Checker**: Builds dependency graphs and topological strata to support safe negation-as-failure.
2. **Safety Verifier**: Guarantees that rules have finite groundings, rejecting unsafe rules at load time.
3. **Head Aggregator**: Computes mathematical aggregations (`sum`, `count`, etc.) over grouped variables.
