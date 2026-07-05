# Shape Expressions (ShEx) in Roxi

## 1. Declarative Graph Schemas

Shape Expressions (ShEx) is an RDF validation language focused on graph topology, recursion, and schema definition. While SHACL is designed as an independent constraint-checking language, ShEx is built to represent graph shapes as grammar rules. 

ShEx is widely used in bioinformatics (e.g. genomic graphs) and library databases where graphs contain deep, recursive networks of relationships.

In Roxi, the ShEx validation pipeline evaluates shape constraints against the `TripleIndex` by mapping structures through the `oxrdf` zero-copy adapter and running validation checks via `shex_validation`.

---

## 2. ShExJ Format Compilation

To minimize parsing overhead and avoid compiling a second complex Turtle-like grammar, Roxi adopts **ShExJ** (the JSON serialization of ShEx schemas) as its primary schema input.

During schema loading:
1. The JSON schema string is parsed into AST structures using `serde_json`.
2. These structures are compiled into a recursive `Schema` object in [lib/src/shex.rs](file:///Users/sac/roxi/lib/src/shex.rs).
3. The schema is cached, ready to validate data mappings.

```
       JSON Schema String (ShExJ)
                   |
                   v
         serde_json Compiler
                   |
                   v
         Recursive Schema AST
                   |
                   v
         Validation Execution
```

---

## 3. ShapeMaps and Recursive Validation

ShEx uses a **ShapeMap** to link nodes in the data graph to expected shapes:

```json
[
  { "node": "http://example.org/alice", "shape": "http://example.org/UserShape" }
]
```

### The Recursive Validation Loop
Validation of deep, circular graph models requires tracking evaluation states:

1. **State Registration**: When validating a node `N` against a shape `S`, the engine registers the evaluation tuple `(N, S)` as *active*.
2. **Recursive Traversal**: If `S` requires validating a related node `N2` against `S2` (e.g. `?this :friend ?friend`), the engine recurses.
3. **Cycle Short-Circuiting**: If the engine encounters `(N, S)` again, it resolves the check based on the recursion rules:
   * Under a strict evaluation, circular dependencies are assumed to be valid pending outer matching success, preventing infinite recursion loops.
4. **Handoff**: The results are compiled into a validation report.

---

## 4. Rust Integration Reference

Below is the Rust structural design of the ShEx schema representations and state tracking in Roxi:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum NodeConstraint {
    Datatype(String),
    NodeKind(String),
}

#[derive(Debug, Clone)]
pub enum ShapeExpr {
    NodeConstraint(NodeConstraint),
    TripleConstraint {
        predicate: String,
        min: usize,
        max: Option<usize>,
        value_expr: Option<Box<ShapeExpr>>,
    },
    EachOf {
        expressions: Vec<ShapeExpr>,
    },
    ShapeRef(String), // Reference to another shape
}

pub struct Schema {
    pub shapes: HashMap<String, ShapeExpr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationResult {
    Conforms,
    Violates,
    Evaluating, // Used to detect recursive cycles
}

pub struct ShExValidator {
    pub schema: Schema,
}

impl ShExValidator {
    /// Validates a node against a shape, handling recursive cycles safely
    pub fn validate_node(
        &self,
        node: &str,
        shape_name: &str,
        state: &mut HashMap<(String, String), ValidationResult>,
    ) -> ValidationResult {
        let key = (node.to_string(), shape_name.to_string());

        // 1. Detect recursive cycle
        if let Some(&status) = state.get(&key) {
            if status == ValidationResult::Evaluating {
                // Assume conforms inside a cycle to allow termination
                return ValidationResult::Conforms;
            }
            return status;
        }

        // 2. Mark state as evaluating
        state.insert(key.clone(), ValidationResult::Evaluating);

        let shape_expr = match self.schema.shapes.get(shape_name) {
            Some(expr) => expr,
            None => {
                state.insert(key, ValidationResult::Violates);
                return ValidationResult::Violates;
            }
        };

        // 3. Evaluate shape expression
        let result = self.evaluate_expression(node, shape_expr, state);
        state.insert(key, result);

        result
    }

    fn evaluate_expression(
        &self,
        _node: &str,
        _expr: &ShapeExpr,
        _state: &mut HashMap<(String, String), ValidationResult>,
    ) -> ValidationResult {
        // Mock evaluation implementation
        ValidationResult::Conforms
    }
}
```
