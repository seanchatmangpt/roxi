# SPARQL Query Algebra & Planning

## 1. SPARQL Compilation Pipeline

Roxi implements a native SPARQL 1.1 query engine in [lib/src/sparql.rs](file:///Users/sac/roxi/lib/src/sparql.rs). When a SPARQL query string is executed, it passes through three translation steps:

```
        SPARQL Query String
                |
                v
         Spargebra Parser
        (Query Algebra AST)
                |
                v
     Plan Extraction Pipeline
     (Optimized PlanNode Tree)
                |
                v
        Execution Engine
    (Lazy TripleIndex Scans)
```

1. **Syntactic Parsing**: The query string is parsed by the `spargebra` crate, which compiles it into a query algebra tree.
2. **Plan Extraction**: The query algebra is mapped to a Roxi-native `PlanNode` tree. This step simplifies joins, optimizes filters, and structures triple pattern lookups.
3. **Execution**: The `PlanNode` tree is evaluated against the `TripleIndex` to generate a stream of bindings.

---

## 2. Query Optimization Strategies

To ensure high performance over large graphs, Roxi applies several optimization rules during plan extraction:

* **Triple Pattern Ordering**: Triple patterns within a basic graph pattern (BGP) are ordered based on variable binding selectivity. Patterns with bound subjects or predicates are evaluated first, reducing intermediate result sizes.
* **Lazy Join Evaluation**: Joins are evaluated lazily. Bindings from the left-hand input are propagated to the right-hand input on-the-fly, allowing the engine to perform index lookups directly rather than generating cross-products.
* **Filter Pushdown**: Filter expressions are pushed down the plan tree as close as possible to the triple pattern scan nodes. This filters out invalid bindings early, avoiding overhead in upper join nodes.

---

## 3. PlanNode Representations

The query plan is represented as a tree of `PlanNode` variants:

```rust
pub enum PlanNode {
    Done,
    Scan(TriplePattern),
    Join {
        left: Box<PlanNode>,
        right: Box<PlanNode>,
    },
    Filter {
        expression: PlanExpression,
        input: Box<PlanNode>,
    },
    Project {
        variables: Vec<String>,
        input: Box<PlanNode>,
    },
}
```

* **`Scan`**: Scans the `TripleIndex` using a pattern (subject, predicate, object, graph).
* **`Join`**: Computes the relational join of two sub-plans.
* **`Filter`**: Evaluates expression conditions over bindings, filtering out non-matching rows.
* **`Project`**: Restricts bindings to select output variables.

---

## 4. Rust Execution Code Reference

Below is the Rust structural design of the query planner and evaluator in Roxi:

```rust
use std::collections::HashMap;

pub struct TriplePattern {
    pub s: usize,
    pub p: usize,
    pub o: usize,
}

pub struct Binding {
    pub map: HashMap<String, usize>,
}

pub enum PlanNode {
    Scan(TriplePattern),
    Join {
        left: Box<PlanNode>,
        right: Box<PlanNode>,
    },
    Filter {
        expression: String, // Simplified string representation
        input: Box<PlanNode>,
    },
}

pub struct QueryEvaluator;

impl QueryEvaluator {
    /// Evaluates a plan node recursively against the TripleIndex
    pub fn evaluate(
        &self,
        node: &PlanNode,
        index: &roxi::tripleindex::TripleIndex,
        input_bindings: Vec<Binding>,
    ) -> Vec<Binding> {
        match node {
            PlanNode::Scan(pattern) => {
                let mut results = Vec::new();
                for binding in input_bindings {
                    // Match pattern with bound variables from input
                    let matches = index.query_pattern(
                        Self::resolve_var(pattern.s, &binding),
                        Self::resolve_var(pattern.p, &binding),
                        Self::resolve_var(pattern.o, &binding),
                    );
                    for m in matches {
                        let mut new_binding = binding.clone();
                        // Bind variables based on match
                        new_binding.map.insert("subject".to_string(), m.s);
                        results.push(new_binding);
                    }
                }
                results
            }
            PlanNode::Join { left, right } => {
                // Lazily join left and right plans
                let left_results = self.evaluate(left, index, input_bindings);
                self.evaluate(right, index, left_results)
            }
            PlanNode::Filter { expression, input } => {
                let input_results = self.evaluate(input, index, input_bindings);
                input_results
                    .into_iter()
                    .filter(|binding| self.evaluate_filter_expr(expression, binding))
                    .collect()
            }
        }
    }

    fn resolve_var(val: usize, _binding: &Binding) -> usize {
        val
    }

    fn evaluate_filter_expr(&self, _expr: &str, _binding: &Binding) -> bool {
        true
    }
}
```
