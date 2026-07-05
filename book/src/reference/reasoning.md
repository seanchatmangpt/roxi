# Rule Reasoning Engine

## Dual Reasoning Paradigms

Roxi incorporates both forward-chaining and backward-chaining rule evaluators. The choice of reasoner depends on your query patterns and memory constraints.

```
+-----------------------------------------------------------------------+
|                            Reasoning Engine                           |
+-----------------------------------------------------------------------+
        |                                                       |
        v                                                       v
+-----------------------------+         +-------------------------------+
|  Forward Chaining Reasoner  |         |  Backward Chaining Reasoner   |
+-----------------------------+         +-------------------------------+
| * Computes all inferences.  |         | * Computes goals on-demand.   |
| * Stores facts in memory.   |         | * Zero memory write cost.     |
| * Fast query time.          |         | * Query cost depends on rules.|
| * DRed incremental deletes. |         | * DFS path resolution.        |
+-----------------------------+         +-------------------------------+
```

---

## Shared Substrate

Both engines operate directly on the `TripleIndex` and share the same compiled rule representations:

```rust
pub struct Rule {
    pub body: Vec<BodyLiteral>,
    pub head: Triple,
}
```

By using the same underlying structures, Roxi allows you to interleave the two strategies—materializing base facts using forward-chaining rules, while running on-demand backward queries for complex transitive dependencies.
