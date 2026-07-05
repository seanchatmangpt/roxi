# SHACL Shapes Validation in Roxi

## 1. W3C Shapes Constraint Language (SHACL)

SHACL (Shapes Constraint Language) is a W3C Recommendation for validating RDF graphs against a set of conditions. These conditions are represented as **shapes** (which are themselves RDF graphs). 

SHACL is essential for enforcing schema invariants, datatypes, cardinalities, and custom constraints in enterprise knowledge graphs.

In Roxi, the SHACL engine validates the target graph against a shapes graph. It leverages the `rudof` validation library through the `oxrdf` zero-copy adapter, ensuring high-performance validation directly on Roxi's core memory indexes.

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

Below is the Rust structural design of the validation loop and report generation in Roxi:

```rust
use std::collections::HashSet;

pub struct ValidationViolation {
    pub focus_node: String,
    pub path: Option<String>,
    pub message: String,
    pub severity: String,
}

pub struct SHACLReport {
    pub conforms: bool,
    pub results: Vec<ValidationViolation>,
}

pub struct SHACLValidator {
    // Shapes and data graphs
}

impl SHACLValidator {
    /// Validates focus nodes against minCount and maxCount constraints
    pub fn validate_property_cardinality(
        &self,
        focus_nodes: &[usize],
        predicate: usize,
        min_count: Option<usize>,
        max_count: Option<usize>,
        index: &roxi::tripleindex::TripleIndex,
    ) -> SHACLReport {
        let mut violations = Vec::new();

        for &node in focus_nodes {
            // Count matching triples: (?this, predicate, ?val)
            let match_count = index.count_matching_subjects(node, predicate);

            if let Some(min) = min_count {
                if match_count < min {
                    violations.push(ValidationViolation {
                        focus_node: format!("node_{}", node),
                        path: Some(format!("predicate_{}", predicate)),
                        message: format!("Property count {} is less than minimum {}.", match_count, min),
                        severity: "sh:Violation".to_string(),
                    });
                }
            }

            if let Some(max) = max_count {
                if match_count > max {
                    violations.push(ValidationViolation {
                        focus_node: format!("node_{}", node),
                        path: Some(format!("predicate_{}", predicate)),
                        message: format!("Property count {} exceeds maximum {}.", match_count, max),
                        severity: "sh:Violation".to_string(),
                    });
                }
            }
        }

        SHACLReport {
            conforms: violations.is_empty(),
            results: violations,
        }
    }
}
```
