# Shape Expressions (ShEx) in Roxi

## 1. Declarative Graph Schemas

Shape Expressions (ShEx) is an RDF validation language focused on graph topology, recursion, and schema definition. While SHACL is designed as an independent constraint-checking language, ShEx is built to represent graph shapes as grammar rules. 

ShEx is widely used in bioinformatics (e.g. genomic graphs) and library databases where graphs contain deep, recursive networks of relationships.

In Roxi, the ShEx validation pipeline (`lib/src/shex_native.rs`, re-exported from `lib/src/shex.rs`) is a fully native implementation operating directly on `TripleIndex` -- no external ShEx crate and no `oxrdf` conversion step. An earlier version delegated to external `shex_ast`/`shex_validation` crates; that was replaced after fuzzing found a real spec violation in their `OneOf` handling (a satisfied alternative plus an unrelated "extra" predicate on a non-CLOSED shape was wrongly rejected). Owning the validation logic natively -- mirroring `shacl.rs`, which was native from the start -- closed that gap and removed the "unverified because it's someone else's code" risk.

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

The real public entry point and AST, from `lib/src/lib.rs` and `lib/src/shex_native.rs`:

```rust
impl TripleStore {
    /// Validates this store's data against a ShExJ schema and a shape map
    /// (a list of `(focus_node_iri, shape_id)` pairs).
    pub fn validate_shex(
        &self,
        schema_json: &str,
        shape_map: &[(String, String)],
    ) -> Result<shex::ShexValidationReport, String>;
}

pub struct Schema {
    pub shapes: Vec<ShapeDecl>,
}

pub struct ShapeDecl {
    pub id: String,
    pub shape_expr: ShapeExpr,
}

/// A shape-expression position may hold either an inline expression or a
/// bare-string reference to another top-level `ShapeDecl` by its `id`.
pub enum ShapeExprOrRef {
    Ref(String),
    Expr(ShapeExpr),
}

pub enum ShapeExpr {
    Shape { closed: bool, extra: Vec<String>, expression: Option<TripleExpr> },
    ShapeAnd { shape_exprs: Vec<ShapeExprOrRef> },
    ShapeOr { shape_exprs: Vec<ShapeExprOrRef> },
    ShapeNot { shape_expr: Box<ShapeExprOrRef> },
    NodeConstraint { /* nodeKind, datatype, facets, values */ },
}

pub enum TripleExpr {
    TripleConstraint { predicate: String, value_expr: Option<Box<ShapeExprOrRef>>, min: Option<i64>, max: Option<i64> },
    EachOf { expressions: Vec<TripleExpr> },
    OneOf { expressions: Vec<TripleExpr> },
}
```

Cycle safety uses a `HashSet<(usize focus_node, String shape_label)>` visited-set exactly like `shacl.rs::validate_shape`'s own cycle guard (see `validate_ref` in `shex_native.rs`): re-entering an already-active `(focus, shape)` pair short-circuits to success rather than recursing forever, so mutually- or self-referential shapes terminate.
