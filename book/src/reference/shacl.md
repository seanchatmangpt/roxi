# SHACL Shapes Validation in Roxi

## 1. W3C Shapes Constraint Language (SHACL)

SHACL (Shapes Constraint Language) is a W3C Recommendation for validating RDF graphs against a set of conditions. These conditions are represented as **shapes** (which are themselves RDF graphs). 

SHACL is essential for enforcing schema invariants, datatypes, cardinalities, and custom constraints in enterprise knowledge graphs.

In Roxi, the SHACL engine (`lib/src/shacl.rs`) is a fully native implementation -- no external SHACL validation library is used. It operates directly on Roxi's `TripleIndex` with no intermediate graph-conversion step, parsing the shapes graph itself (`ShapesGraph::parse`) and validating focus nodes directly against it (`Validator::validate`).

---

## 2. Validation Targets & Scope

Before validating constraints, the engine must select which nodes in the data graph (called **focus nodes**) are target subjects for validation. SHACL defines four target types, all supported in Roxi:

1. **`sh:targetNode`**: Selects specific individual nodes.
2. **`sh:targetClass`**: Selects all instances of a class (nodes that have an `rdf:type` relationship to the target class, including subclass hierarchies computed via `CSprite`).
3. **`sh:targetSubjectsOf`**: Selects all subjects of triples that use a specific predicate.
4. **`sh:targetObjectsOf`**: Selects all objects of triples that use a specific predicate.

```
                  SHACL Target Matcher
                           |
            +--------------+--------------+
            |                             |
            v                             v
     sh:targetClass               sh:targetNode
  (Collect instances)          (Match specific node)
            |                             |
            +--------------+--------------+
                           |
                           v
                      Focus Nodes
```

---

## 3. Core Constraint Evaluation

Once focus nodes are collected, Roxi validates them against shape declarations:

### Cardinality Constraints
* **`sh:minCount`**: Ensures a property has at least $N$ values.
* **`sh:maxCount`**: Ensures a property has at most $M$ values.
* *Optimization*: Cardinality checks perform lazy count evaluations on the `TripleIndex` without materializing the full result sets.

### Range & Datatype Constraints
* **`sh:datatype`**: Verifies that literal values match expected XML Schema datatypes (e.g. `xsd:integer`, `xsd:string`).
* **`sh:class`**: Verifies that object values belong to a specific class hierarchy.

### Logical & String Constraints
* **`sh:and` / `sh:or` / `sh:not`**: Evaluates logical combinations of shapes.
* **`sh:pattern`**: Validates string values against regular expressions.

---

## 4. SPARQL-Based Constraints

For complex validation rules that cannot be expressed using core SHACL constraints, SHACL allows embedding SPARQL queries:

```turtle
:UserShape a sh:NodeShape ;
    sh:targetClass :User ;
    sh:sparql [
        sh:message "Users cannot have a registration date in the future." ;
        sh:select """
            PREFIX : <http://example.org/>
            PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
            SELECT ?this WHERE {
                ?this :registeredDate ?date .
                FILTER (?date > NOW())
            }
        """
    ] .
```

* **Evaluation**: The embedded SPARQL query is parsed by `spargebra` and executed by Roxi's query engine over the data graph.
* **Violation**: If the query returns any bindings for `?this` (the focus node), a validation violation is recorded.

---

## 5. Rust Integration Reference

The real public entry point, from `lib/src/lib.rs` and `lib/src/shacl.rs`:

```rust
impl TripleStore {
    /// Parses `shapes_turtle` as a SHACL shapes graph and validates
    /// this store's data against it.
    pub fn validate_shacl(&self, shapes_turtle: &str) -> Result<shacl::ValidationReport, String>;
}

pub struct ValidationReport {
    pub conforms: bool,
    pub results: Vec<ValidationResult>,
}

impl ValidationReport {
    /// Serializes this report as a real `sh:ValidationReport` RDF graph
    /// (a `Vec<Triple>`), per the SHACL spec's own vocabulary for reports.
    pub fn to_triples(&self) -> Vec<Triple>;
}

pub struct ValidationResult {
    pub focus_node: Term,
    pub result_path: Option<Term>,
    pub value: Option<Term>,
    pub source_constraint_component: Term,
    pub source_shape: Term,
    pub severity: Term,
    pub message: Option<String>,
}
```
