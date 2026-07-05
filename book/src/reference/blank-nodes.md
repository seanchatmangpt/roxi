# Blank Nodes

## Anonymous Resources in Roxi

Blank nodes (or anonymous nodes) represent resources that do not have an explicit IRI. In RDF syntax, they are represented with the `_:` prefix (e.g., `_:b1`).

Roxi models blank nodes using the `BlankNodeImpl` struct:

```rust
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlankNodeImpl {
    pub id: usize,
}
```

---

## Scoping and Identifier Allocation

Because blank node labels (like `_:b1`) are local to the document, they must be scoped carefully to prevent collisions:

1. **Parser Allocation**: When a parser processes a document, it tracks local blank node labels. For each unique label, it requests a new identifier from the global `Encoder`.
2. **Session Scoping**: If you load multiple files into the same `TripleStore`, the encoder guarantees that `_:b1` in File A and `_:b1` in File B receive different `usize` IDs, preserving graph isolation.
3. **Adapter Translation**: When mapping to `oxrdf` structures, Roxi translates these internal IDs to unique blank node labels (`b0`, `b1`, etc.) so external validation engines can map blank node relations correctly.
