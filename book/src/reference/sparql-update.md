# SPARQL 1.1 Update & DRed Integration

## 1. SPARQL Update Operations

Roxi supports the W3C SPARQL 1.1 Update specification, which provides a declarative syntax for adding, deleting, and modifying RDF graphs. 

Supported operations include:
* **`INSERT DATA`**: Directly adds new triples to the graph index.
* **`DELETE DATA`**: Removes matching triples from the index.
* **`DELETE/INSERT`**: Queries the store first, binds variables, and uses the resulting bindings to delete old triples and insert new ones.
* **`CLEAR`**: Clears all triples from a specific graph or the entire workspace.

---

## 2. DRed Incremental Maintenance Integration

When triples are deleted via `DELETE DATA` or `DELETE/INSERT`, Roxi does not simply remove the records from the `TripleIndex`. Deleting base facts can render previously materialized forward-chained inferences invalid.

To maintain consistency without performing full, expensive re-reasoning runs, Roxi integrates SPARQL Update deletions directly with the **Delete/Rederive (DRed)** incremental maintenance engine:

```
        SPARQL Update Delete Query
                    |
                    v
          Delete Base Triples
                    |
                    v
    DRed Engine: Mark Dependent Triples
  (Collect derived facts that use deleted base)
                    |
                    v
     Rederive: Check Alternate Supports
  (Verify if facts can be derived by other paths)
                    |
                    v
       Confirm Deletions & Clean-up
  (Remove base & unsupported derived triples)
```

1. **Delete Propagation**: The SPARQL engine deletes the base triples. It then passes the deleted triples to the DRed engine. DRed identifies all derived triples that used the deleted triples as rule supports and marks them for deletion.
2. **Alternate Support Check (Rederive)**: The engine verifies if any of the marked derived triples can still be inferred through alternative rules or independent base facts. If an alternate support is found, the triple is unmarked.
3. **Execution**: Remaining marked triples are removed from the index.

This integration guarantees that updates are propagated through all rule layers instantly, maintaining graph consistency.

---

## 3. Rust Update Implementation Reference

Below is the Rust structural design of the SPARQL Update coordinator and DRed integration in Roxi:

```rust
use std::collections::HashSet;

pub struct UpdateOperation {
    pub insert_triples: Vec<roxi::triples::Triple>,
    pub delete_triples: Vec<roxi::triples::Triple>,
}

pub struct UpdateCoordinator {
    // Reference to index and reasoner
}

impl UpdateCoordinator {
    /// Executes a SPARQL Update operation, triggering DRed updates
    pub fn execute_update(
        &self,
        op: UpdateOperation,
        index: &mut roxi::tripleindex::TripleIndex,
        dred: &mut roxi::dred::DRed,
    ) -> Result<(), UpdateError> {
        // 1. Execute direct insertions
        if !op.insert_triples.is_empty() {
            index.insert_batch(&op.insert_triples);
        }

        // 2. Execute deletions with DRed incremental updates
        if !op.delete_triples.is_empty() {
            // Collect initial base deletions
            let mut deleted_base = HashSet::new();
            for triple in op.delete_triples {
                if index.contains(&triple) {
                    index.remove(&triple);
                    deleted_base.insert(triple);
                }
            }

            // Propagate deletions using the DRed engine
            dred.propagate_deletions(deleted_base, index);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum UpdateError {
    ExecutionFailure(String),
}
```
