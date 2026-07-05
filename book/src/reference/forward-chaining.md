# Forward Chaining Reasoner

## Materialization Engine

Roxi's forward-chaining reasoner is located in [lib/src/reasoner.rs](file:///Users/sac/roxi/lib/src/reasoner.rs). It pre-computes and stores all inferable facts in the `TripleIndex`.

The materialization process runs a fixpoint execution loop:

```rust
pub fn materialize(&mut self) -> Result<(), ReasoningError> {
    let mut new_triples = Vec::new();
    loop {
        new_triples.clear();
        for rule in &self.rules {
            // Find all matching bindings for the rule body patterns
            let bindings = self.index.query_body(&rule.body);
            for binding in bindings {
                // Ground the rule head and check if it already exists
                let derived = rule.head.ground(&binding);
                if !self.index.contains(&derived) {
                    new_triples.push(derived);
                }
            }
        }
        if new_triples.is_empty() {
            break; // Fixpoint reached
        }
        self.index.insert_batch(&new_triples);
    }
    Ok(())
}
```

---

## Incremental Maintenance (DRed)

To avoid full recalculations when triples are deleted, Roxi uses the **Delete/Rederive (DRed)** algorithm:

1. **Delete Propagation**: When a base triple is deleted, the engine propagates the deletion. Any derived triple that depends on the deleted triple is marked for deletion.
2. **Rederivation Check**: The engine checks if the marked triples can be derived through alternate paths (e.g., via different rules or base facts). If an alternate path is found, the triple is unmarked.
3. **Clean-up**: Remaining marked triples are removed from the index.
