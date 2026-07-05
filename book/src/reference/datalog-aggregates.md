# Datalog Aggregations & Grouping

## 1. Declarative Aggregations

Aggregations allow Datalog rules to summarize multiple matching facts into single derived values (such as count, sum, minimum, maximum, and average). In Roxi, these calculations are performed dynamically during forward-chaining evaluation.

### Syntax
In Roxi Datalog, aggregations are declared directly in the rule head. Any variable in the head that is *not* an argument of the aggregation function is automatically treated as a **grouping key**.

Consider the query:

$$\text{managerReportCount}(?M, \text{count}(?R)) \leftarrow \text{manages}(?M, ?R)$$

* **Grouping Key**: `?M` (the manager).
* **Aggregate Target**: `?R` (the reports).
* **Output**: The rule yields a table where each manager is associated with their total count of direct reports.

---

## 2. Compilation and Binding Routing

When a rule containing aggregates is parsed, Roxi compiles it into a specialized rule structure. The body of the rule is evaluated first, producing a table of raw bindings.

```
                   Evaluate Rule Body
                           |
                           v
                  Raw Bindings Table
              +--------------------------+
              | ?manager    | ?report    |
              +-------------+------------+
              | alice       | bob        |
              | alice       | charlie    |
              | dave        | eve        |
              +-------------+------------+
                           |
                           v
                 Grouping & Sorting
               (Group by ?manager)
                           |
            +--------------+--------------+
            |                             |
            v                             v
     Group: [alice]                Group: [dave]
     Values: [bob, charlie]        Values: [eve]
            |                             |
            v                             v
    CountAccumulator              CountAccumulator
    (Count = 2)                   (Count = 1)
            |                             |
            +--------------+--------------+
                           |
                           v
                Derive Head Triples:
        managerReportCount(alice, 2)
        managerReportCount(dave, 1)
```

---

## 3. Reuse of SPARQL Accumulators

To minimize code duplication and maintain memory efficiency, Roxi's Datalog engine reuses the core aggregation machinery implemented for the SPARQL 1.1 query engine in [lib/src/sparql.rs](file:///Users/sac/roxi/lib/src/sparql.rs):

* **`CountAccumulator`**: Counts matching instances.
* **`SumAccumulator`**: Adds numeric literal values.
* **`MinAccumulator` / `MaxAccumulator`**: Tracks lexicographical or numeric extremes.
* **`AvgAccumulator`**: Computes running totals and counts to yield arithmetic means.

By sharing these accumulators, Datalog aggregations benefit from the same type checking and performance optimizations built for SPARQL queries.

---

## 4. Rust Aggregator Implementation

Below is the Rust implementation demonstrating how Roxi groups bindings and executes Datalog aggregations:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constant {
    Iri(usize),
    Integer(i64),
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub map: HashMap<String, Constant>,
}

pub enum AggregationFunction {
    Count,
    Sum,
}

pub struct AggregateEvaluator;

impl AggregateEvaluator {
    /// Groups bindings by grouping keys and evaluates the aggregate function
    pub fn evaluate(
        bindings: &[Binding],
        group_keys: &[String],
        agg_var: &str,
        func: AggregationFunction,
    ) -> HashMap<Vec<Constant>, Constant> {
        let mut groups: HashMap<Vec<Constant>, Vec<Constant>> = HashMap::new();

        // 1. Group bindings by key values
        for binding in bindings {
            let key_values: Vec<Constant> = group_keys
                .iter()
                .map(|key| binding.map.get(key).cloned().unwrap_or(Constant::Integer(0)))
                .collect();

            if let Some(target_val) = binding.map.get(agg_var).cloned() {
                groups.entry(key_values).or_default().push(target_val);
            }
        }

        let mut results = HashMap::new();

        // 2. Evaluate function per group
        for (keys, values) in groups {
            let aggregate_val = match func {
                AggregationFunction::Count => Constant::Integer(values.len() as i64),
                AggregationFunction::Sum => {
                    let mut sum = 0;
                    for val in values {
                        if let Constant::Integer(n) = val {
                            sum += n;
                        }
                    }
                    Constant::Integer(sum)
                }
            };
            results.insert(keys, aggregate_val);
        }

        results
    }
}
```

---

## 5. Recursion and Aggregation Boundaries

Aggregations must be handled carefully when combined with recursive rules. If a rule depends on an aggregate value that is itself recursively updated, evaluation can fail to reach a stable state.

To prevent this:
1. **Stratification Barrier**: Roxi requires that any aggregation must exist strictly on a **stratum boundary**.
2. **Fixed Input**: The relations being aggregated must belong to a lower stratum ($< i$) relative to the rule head predicate. This ensures the aggregated inputs are completely evaluated and fixed before the aggregation executes, guaranteeing mathematical correctness and termination.
3. **Safety Verification**: The stratification checker verifies these bounds at load time, rejecting rules with recursive aggregation loops.
