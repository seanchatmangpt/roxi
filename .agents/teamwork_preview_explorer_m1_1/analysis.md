# Analysis & Implementation Design for TICKET-004

This document presents the detailed architectural analysis and implementation recommendations for **TICKET-004 (Datalog negation, aggregates, rule safety)** in the Roxi RDF engine.

---

## 1. Executive Summary
We recommend extending the existing forward-chaining reasoner (`Reasoner::materialize`) to support stratified negation and grouping aggregates. Rule safety and stratification will be validated at rule-load time, rejecting invalid or unstratifiable rulesets before execution begins. To avoid code duplication, SPARQL aggregate accumulators will be extracted to a shared module.

---

## 2. Rule Representation & Parser Enhancements

To support body literals with polarity (negation), the shape of a `Rule` must be changed in `lib/src/triples.rs` and its parser in `lib/src/parser/n3rule_parser.rs`.

### 2.1. Structural Changes in `triples.rs`
Define `BodyLiteral` and update the `Rule` struct:

```rust
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BodyLiteral {
    pub negated: bool,
    pub pattern: Triple,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Rule {
    pub body: Vec<BodyLiteral>,
    pub head: Triple,
    pub aggregate: Option<Aggregate>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AggregateFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Aggregate {
    pub function: AggregateFunction,
    pub source_var: usize,      // Encoded variable name
    pub target_var: usize,      // Encoded variable name
    pub group_vars: Vec<usize>, // Encoded variable names
}
```

### 2.2. Pest Grammar Updates in `n3.pest`
Introduce a nested `BodyLiteral` rule that parses both positive and negated triples:

```pest
BodyLiteral = { ("not" ~ WS* ~ "{" ~ WS* ~ TP ~ WS* ~ "}") | TP }
Body = { "{" ~ BodyLiteral+ ~ "}" }
```

### 2.3. Parsing Logic in `n3rule_parser.rs`
Update `n3rule_parser.rs` to process the inner pairs of `BodyLiteral`:
- If the pair begins with the `"not"` keyword (or has a `not` rule variant), extract the inner `TP` and set `negated: true`.
- Otherwise, parse the `TP` and set `negated: false`.
- Set `aggregate: None` for standard N3 rules (which do not use Datalog-style head aggregation).

---

## 3. Rule Safety Checking

Rule safety must be checked inside `RuleIndex::add` or when rules are loaded (e.g., in `TripleStore::add_rules`). If a rule is unsafe, a detailed error must be returned (rather than panicking).

### 3.1. Safety Criteria
A rule is **safe** if and only if:
1. Every variable in the rule head—excluding the aggregate `target_var` (if present)—is bound by at least one positive body literal.
2. Every variable in any negated body literal is bound by at least one positive body literal.
3. If an aggregate is present, both the `source_var` and all `group_vars` are bound by at least one positive body literal.

### 3.2. Algorithm Outline
```rust
pub fn check_rule_safety(rule: &Rule) -> Result<(), String> {
    let mut positive_vars = HashSet::new();
    let mut negated_vars = HashSet::new();
    let mut head_vars = HashSet::new();

    // Helper to collect variables from a Triple
    let collect_vars = |triple: &Triple, set: &mut HashSet<usize>| {
        if let VarOrTerm::Var(v) = &triple.s { set.insert(v.name); }
        if let VarOrTerm::Var(v) = &triple.p { set.insert(v.name); }
        if let VarOrTerm::Var(v) = &triple.o { set.insert(v.name); }
        if let Some(VarOrTerm::Var(v)) = &triple.g { set.insert(v.name); }
    };

    for literal in &rule.body {
        if literal.negated {
            collect_vars(&literal.pattern, &mut negated_vars);
        } else {
            collect_vars(&literal.pattern, &mut positive_vars);
        }
    }
    collect_vars(&rule.head, &mut head_vars);

    // 1. Verify negated variables
    for var in &negated_vars {
        if !positive_vars.contains(var) {
            return Err(format!(
                "Unsafe rule: variable ?{} in negated literal is not bound by a positive body literal",
                Encoder::decode(var).unwrap_or_else(|| var.to_string())
            ));
        }
    }

    // 2. Verify head variables
    let agg_target = rule.aggregate.as_ref().map(|a| a.target_var);
    for var in &head_vars {
        if Some(*var) == agg_target {
            continue; // The target of an aggregate is bound by the aggregation itself
        }
        if !positive_vars.contains(var) {
            return Err(format!(
                "Unsafe rule: variable ?{} in rule head is not bound by a positive body literal",
                Encoder::decode(var).unwrap_or_else(|| var.to_string())
            ));
        }
    }

    // 3. Verify aggregate variables
    if let Some(agg) = &rule.aggregate {
        if !positive_vars.contains(&agg.source_var) {
            return Err(format!(
                "Unsafe rule: aggregate source variable ?{} is not bound by a positive body literal",
                Encoder::decode(&agg.source_var).unwrap_or_else(|| agg.source_var.to_string())
            ));
        }
        for g_var in &agg.group_vars {
            if !positive_vars.contains(g_var) {
                return Err(format!(
                    "Unsafe rule: aggregate group variable ?{} is not bound by a positive body literal",
                    Encoder::decode(g_var).unwrap_or_else(|| g_var.to_string())
                ));
            }
        }
    }

    Ok(())
}
```

---

## 4. Stratification & Dependency Analysis

A new module `lib/src/datalog.rs` will handle the dependency graph, strongly connected component (SCC) detection, and strata assignment.

### 4.1. Extracting Datalog Predicates
Predicates in RDF can be either plain URIs or class names when the predicate is `rdf:type` (represented as `a`). We can define a helper representation:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DatalogPredicate {
    Uri(String),  // Predicate URI (cleaned of <>)
    Type(String), // Object URI (cleaned of <>) if predicate is rdf:type
}

impl DatalogPredicate {
    pub fn from_triple(triple: &Triple) -> Option<Self> {
        if !triple.p.is_term() {
            return None; // Variables in predicate position are unsupported in core Datalog
        }
        let p_str = Encoder::decode(&triple.p.to_encoded())?;
        let p_clean = p_str.trim_matches(|c| c == '<' || c == '>');
        if p_clean == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type" {
            if triple.o.is_term() {
                let o_str = Encoder::decode(&triple.o.to_encoded())?;
                let o_clean = o_str.trim_matches(|c| c == '<' || c == '>');
                Some(DatalogPredicate::Type(o_clean.to_string()))
            } else {
                None // Variable class type in body
            }
        } else {
            Some(DatalogPredicate::Uri(p_clean.to_string()))
        }
    }
}
```

### 4.2. Stratification Algorithm
1. **Nodes**: All predicates appearing in rule heads and bodies.
2. **Edges**:
   - If a rule has head predicate `P` and positive body literal with predicate `Q`: add a positive edge `Q -> P` (P depends positively on Q).
   - If a rule has head predicate `P` and negated body literal with predicate `Q` (or the rule has an aggregate and Q is in the body): add a negative/stratified edge `Q -> P` (P depends negatively on Q).
3. **Cycle/Stratification Verification**:
   - Group nodes into Strongly Connected Components (SCCs) using Tarjan's or Kosaraju's algorithm.
   - For every SCC, if there exists a negative/stratified edge between any two nodes in that same SCC, the ruleset contains a negative recursion/aggregate cycle. Reject it by returning `Err("Unstratifiable ruleset")`.
4. **Stratum Assignment**:
   - Construct the DAG of SCCs.
   - Topologically sort the SCCs and assign strata indices bottom-up:
     - `stratum(SCC) = max( { stratum(Parent) + (1 if edge is negative/stratified else 0) } )` (default 0).
   - Assign each rule the stratum of its head predicate.

---

## 5. Stratified Reasoning Loop

Update `Reasoner::materialize` to evaluate rules layer-by-layer:

```rust
pub fn materialize(
    &mut self,
    triple_index: &mut TripleIndex,
    rules_index: &RuleIndex,
) -> Vec<Triple> {
    let mut inferred = Vec::new();

    // 1. Group rules by computed strata
    let stratified_rules: Vec<Vec<Rc<Rule>>> = rules_index.get_stratified_rules();

    for stratum in stratified_rules {
        let stratum_start_counter = triple_index.len();

        // Step A: Evaluate Aggregate and Static Rules Once
        // Since stratification guarantees aggregate input relations are fully evaluated
        // in lower strata, we can evaluate all aggregate rules (and rules depending only
        // on lower strata) once at the start of this stratum.
        for rule in &stratum {
            if rule.aggregate.is_some() || rule_has_only_lower_strata_pos_literals(rule, stratum_start_counter) {
                let new_triples = Self::infer_rule_heads_for_stratum(triple_index, rule, None, stratum_start_counter);
                for triple in new_triples {
                    if !triple_index.contains(&triple) {
                        inferred.push(triple.clone());
                        triple_index.add(triple);
                    }
                }
            }
        }

        // Step B: Incremental Fixpoint Loop for Recursive Stratum Rules
        let mut counter = stratum_start_counter;
        while counter < triple_index.len() {
            let process_quad = triple_index.get(counter).unwrap();
            let matching_rules = rules_index.find_match(process_quad);
            
            for rule in matching_rules {
                // Only evaluate rules belonging to the current stratum
                if !stratum.contains(&rule) || rule.aggregate.is_some() {
                    continue;
                }

                // Match and substitute process_quad into positive body literals
                let substituted_rules = Self::substitute_rule(process_quad, rule);
                let new_triples = Self::infer_rule_heads_for_stratum_list(
                    triple_index,
                    substituted_rules,
                    Some(counter + 1),
                    stratum_start_counter,
                );

                for triple in new_triples {
                    if !triple_index.contains(&triple) {
                        inferred.push(triple.clone());
                        triple_index.add(triple);
                    }
                }
            }
            counter += 1;
        }
    }

    inferred
}
```

### 5.1. Evaluating Negated Body Literals
When checking a negated body literal `not Q(...)`, we must substitute the bindings obtained from the positive literals to create a ground triple, and then check its presence in `triple_index` only up to `stratum_start_counter`:

```rust
let is_present = if stratum_start_counter == 0 {
    false
} else {
    triple_index.query(&negated_triple, Some(stratum_start_counter - 1)).is_some()
};
```
If `is_present` is true, the negation check fails, and we discard that binding row.

---

## 6. Shared Aggregate Module

To prevent code duplication, extract the aggregate accumulators from `lib/src/sparql.rs` to a new module `lib/src/aggregation.rs`:

1. **Move types & traits**: Move `Accumulator` trait, `CountAccumulator`, `SumAccumulator`, `MinAccumulator`, `MaxAccumulator`, `AvgAccumulator`, and the `AccumulatorImpl` enum to `lib/src/aggregation.rs`.
2. **Re-export/Expose**: Expose them from `aggregation.rs`.
3. **Usage**:
   - `sparql.rs` imports `AccumulatorImpl` from `crate::aggregation`.
   - `datalog.rs` / `reasoner.rs` uses the exact same accumulators.
4. **Aggregate Evaluation**:
   When evaluating a rule with `aggregate = Some(Aggregate { function, source_var, target_var, group_vars })`:
   - Group the body evaluation bindings by their values for `group_vars`.
   - For each group, instantiate an `AccumulatorImpl` and feed all `source_var` values into it.
   - Retrieve the result via `get()`, append it as `target_var` to the group's bindings, and emit the instantiated head triple.

---

## 7. Verification & Fixes

### 7.1. Test Suite Path Resolution Error
During `cargo test --features datalog`, compilation currently fails because of nested mod path resolution in `lib/tests/datalog_conformance.rs`:
```rust
#[cfg(feature = "datalog")]
mod datalog_tests {
    #[path = "datalog_conformance/safe_unsafe_rejection.rs"]
    mod safe_unsafe_rejection;
...
```
Because the `datalog_tests` mod wraps the modules, Rust resolves paths relative to `lib/tests/datalog_tests/`. We recommend removing the wrapper `mod datalog_tests` and exposing the modules directly under `#[cfg(feature = "datalog")]`:
```rust
#[cfg(feature = "datalog")]
mod safe_unsafe_rejection;
#[cfg(feature = "datalog")]
mod mutual_recursion;
#[cfg(feature = "datalog")]
mod negation_stratum;
#[cfg(feature = "datalog")]
mod negation_cycle;
#[cfg(feature = "datalog")]
mod aggregations;
```
This resolves the path directly to `lib/tests/datalog_conformance/` without custom path overrides, restoring clean compilation.

### 7.2. Test Command
Verify the implementation with:
```bash
cargo test --features datalog
```
All tests in `lib/tests/datalog_negation.rs` and `lib/tests/datalog_conformance/` should compile and pass.
