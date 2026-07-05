# TICKET-004 Analysis: Datalog Negation, Aggregates, and Rule Safety

This document presents the detailed architectural recommendations and design specifications for implementing TICKET-004 in the Roxi RDF Engine.

---

## 1. Body Polarity & AST Changes (`triples.rs`)

### Objective
Extend the `Rule` representation to support negated body literals while preserving compatibility with existing parsing and execution logic.

### Design Recommendations
1. **Introduce `BodyLiteral`**:
   Define `BodyLiteral` in `lib/src/triples.rs` to associate each triple pattern with a polarity:
   ```rust
   #[derive(Debug, Clone, Eq, PartialEq, Hash)]
   pub struct BodyLiteral {
       pub negated: bool,
       pub pattern: Triple,
   }
   ```
2. **Update `Rule` Struct**:
   Modify `Rule` in `lib/src/triples.rs` and add `Aggregate` metadata for rule heads:
   ```rust
   #[derive(Debug, Clone, Eq, PartialEq, Hash)]
   pub struct Rule {
       pub body: Vec<BodyLiteral>,
       pub head: Triple,
       pub aggregates: Vec<Aggregate>,
   }
   ```
   *Note: If no aggregates are defined, `aggregates` should be an empty `Vec`.*

3. **Update AST Construction Sites**:
   - **`lib/src/parser/n3rule_parser.rs`**:
     - Extend the `n3.pest` grammar to support `not` expressions:
       ```pest
       negated_tp = { "not" ~ WS* ~ "{" ~ TP ~ "}" ~ WS* }
       Literal = { negated_tp | TP }
       Body = { "{" ~ Literal+ ~ "}" }
       ```
     - Update `n3rule_parser::parse` to handle `Rule::negated_tp` and map it to `BodyLiteral { negated: true, pattern: ... }`.
   - **`lib/src/parser.rs`**:
     - Update the simpler fallback `Parser::parse` method to handle `not { ... }` syntax by detecting the `not` prefix, stripping brackets, and creating `BodyLiteral` entries accordingly.
   - **`lib/src/ruleindex.rs`**:
     - `RuleIndex::add_rc` indexes rules to speed up matching during forward-chaining.
     - Modify `add_rc` to **only index positive body literals** (i.e. `!literal.negated`). Under stratified evaluation, negated literals refer to predicates of strictly lower strata that are already completely computed and static. Therefore, rules do not need to be triggered by new matches on negated patterns.
     - Modify the indexing loop in `add_rc`:
       ```rust
       for literal in rule.body.iter() {
           if literal.negated {
               continue;
           }
           let Triple { s, p, o, .. } = &literal.pattern;
           // ... existing index insertion logic ...
       }
       ```

---

## 2. Predicate Dependency & Stratification (`datalog.rs`)

### Objective
Ensure Datalog negation is evaluated in a stratified manner by sorting rules into strata based on predicate dependencies and rejecting unstratifiable rulesets containing self-negation cycles.

### Design Recommendations
1. **Represent Predicates**:
   Use the property `p` of the `Triple` pattern as the predicate identifier (encoded as a `usize` ID via `Encoder`).

2. **Build Dependency Graph**:
   For each rule with head predicate $H$:
   - For each positive body literal with predicate $B$, add a positive dependency edge $B \xrightarrow{0} H$.
   - For each negated body literal with predicate $B$, add a negative dependency edge $B \xrightarrow{1} H$.
   - For each aggregated body literal with predicate $B$, add a negative dependency edge $B \xrightarrow{1} H$ (since aggregation requires the input relation to be fully evaluated first).

3. **Stratification Solver Algorithm**:
   A simple, robust iterative algorithm (Bellman-Ford-like) assigns stratum numbers:
   ```rust
   pub fn stratify(rules: &[Rule]) -> Result<HashMap<usize, usize>, String> {
       let mut stratum: HashMap<usize, usize> = HashMap::new();
       let predicates: HashSet<usize> = rules.iter()
           .flat_map(|r| {
               let mut preds = vec![r.head.p.to_encoded()];
               for lit in &r.body {
                   preds.push(lit.pattern.p.to_encoded());
               }
               preds
           })
           .collect();

       for &p in &predicates {
           stratum.insert(p, 0);
       }

       let n = predicates.len();
       for iteration in 0..=n {
           let mut changed = false;
           for rule in rules {
               let h = rule.head.p.to_encoded();
               let h_stratum = *stratum.get(&h).unwrap();
               
               for lit in &rule.body {
                   let b = lit.pattern.p.to_encoded();
                   let b_stratum = *stratum.get(&b).unwrap();
                   
                   let required = if lit.negated { b_stratum + 1 } else { b_stratum };
                   if required > h_stratum {
                       stratum.insert(h, required);
                       changed = true;
                   }
               }
               
               for agg in &rule.aggregates {
                   // Aggregates behave like negation for stratification (stratum must be strictly greater)
                   for lit in &rule.body {
                       let b = lit.pattern.p.to_encoded();
                       let b_stratum = *stratum.get(&b).unwrap();
                       if b_stratum + 1 > h_stratum {
                           stratum.insert(h, b_stratum + 1);
                           changed = true;
                       }
                   }
               }
           }
           if !changed {
               return Ok(stratum);
           }
           if iteration == n {
               return Err("Ruleset is not stratifiable due to a negation or aggregation cycle".to_string());
           }
       }
       Ok(stratum)
   }
   ```

---

## 3. Stratified Evaluation (`reasoner.rs`)

### Objective
Update `Reasoner::materialize` to evaluate rules layer-by-layer using the stratum assignments.

### Design Recommendations
1. **Sequential Strata Evaluation Loop**:
   Modify `Reasoner::materialize` to partition rules by their head predicate's stratum and evaluate them sequentially:
   ```rust
   pub fn materialize_stratified(
       &mut self,
       triple_index: &mut TripleIndex,
       rules: &[Rule],
   ) -> Result<Vec<Triple>, String> {
       let stratum_map = stratify(rules)?;
       let max_stratum = *stratum_map.values().max().unwrap_or(&0);
       
       let mut inferred = Vec::new();
       
       for s in 0..=max_stratum {
           // Partition rules for the current stratum
           let stratum_rules: Vec<Rule> = rules.iter()
               .filter(|r| *stratum_map.get(&r.head.p.to_encoded()).unwrap() == s)
               .cloned()
               .collect();
               
           if stratum_rules.is_empty() {
               continue;
           }
           
           let mut stratum_rules_index = RuleIndex::new();
           for r in &stratum_rules {
               stratum_rules_index.add(r.clone());
           }
           
           // Run fixpoint evaluation for the current stratum
           let mut stratum_counter = 0;
           while stratum_counter < triple_index.triples.len() {
               let process_quad = triple_index.get(stratum_counter).unwrap();
               let matching_rules = stratum_rules_index.find_match(process_quad);
               
               // Substitute the matching triple into positive literals
               let matching_rules: Vec<Rule> = matching_rules
                   .into_iter()
                   .flat_map(|r| Self::substitute_rule(process_quad, r))
                   .collect();
                   
               let new_triples = Self::infer_rule_heads_stratified(
                   triple_index,
                   Some(stratum_counter + 1),
                   matching_rules,
               );
               
               for triple in new_triples {
                   if !triple_index.contains(&triple) {
                       inferred.push(triple.clone());
                       triple_index.add(triple);
                   }
               }
               stratum_counter += 1;
           }
       }
       Ok(inferred)
   }
   ```

2. **Evaluate Negated Body Literals**:
   In `infer_rule_heads_stratified`, evaluate the rule body:
   - Query all positive body literals of the rule using `SimpleQueryEngine::query` on the current `triple_index`.
   - If bindings are found, filter the rows in `Binding` using the negated body literals.
   - For each negated literal `not { pattern }` in the rule body:
     - For each row in the `Binding` (from 0 to `len - 1`):
       - Substitute the bound variables for this row into the negated `pattern`.
       - Query the `triple_index` for any match of the substituted pattern.
       - If any match is found, remove/discard this row from the `Binding`.
   - If any rows remain in `Binding`, substitute them into the head (and compute aggregates if present) to produce the inferred head triples.

---

## 4. Rule Safety Checking

### Objective
Reject unsafe rules at load-time to prevent infinite groundings and undefined queries.

### Safety Rules Formulation
A Datalog rule is safe if and only if:
1. **Head Variable Binding**: Every variable in the rule head appears in at least one **positive body literal**.
2. **Negated Variable Binding**: Every variable in a negated literal that is used elsewhere in the rule (in the head or other literals) must appear in at least one **positive body literal**.
3. **Disconnected Negation Check**: For each negated literal, at least one of its variables must appear in a **positive body literal** (or be a constant/term). This rejects completely unbound negated queries (like `not { ?y :hasChild ?z }` in a rule about `?x :type :Parent`).
4. **Aggregate Variable Binding**: Every variable in the aggregate expression (`source_var`, `group_vars`) must appear in at least one **positive body literal**.

### Implementation in `lib/src/ruleindex.rs`
Create a validator method `validate_rule_safety(rule: &Rule) -> Result<(), RuleSafetyError>` and invoke it inside `RuleIndex::add`:
- Extract `positive_vars` from `rule.body.iter().filter(|l| !l.negated)`.
- Verify `rule.head` variables are a subset of `positive_vars`.
- Verify variables in negated literals satisfy the safety rules above.
- Verify aggregate variables are a subset of `positive_vars`.

---

## 5. SPARQL Aggregator Extraction & Reuse (`aggregation.rs`)

### Objective
Consolidate aggregation accumulator logic to prevent duplication and facilitate reuse.

### Refactoring Steps
1. **Create `lib/src/aggregation.rs`**:
   Extract the following constructs from `lib/src/sparql.rs` into `aggregation.rs`:
   - `Accumulator` trait.
   - `CountAccumulator`, `SumAccumulator`, `MinAccumulator`, `MaxAccumulator`, `AvgAccumulator` structs.
   - `AccumulatorImpl` enum (implementing the `Accumulator` trait by delegation).
   - `PlanAggregationFunction` enum (re-named or aliased as `AggregateFunction` for Datalog).
2. **Expose via `lib.rs`**:
   Declare `pub mod aggregation;` in `lib.rs` to allow imports from both `sparql.rs` and `datalog.rs`.
3. **Execute Aggregates in Datalog Rule Evaluation**:
   During Datalog rule head evaluation:
   - Identify if the rule has associated aggregates:
     ```rust
     #[derive(Debug, Clone, Eq, PartialEq, Hash)]
     pub struct Aggregate {
         pub function: AggregateFunction,
         pub source_var: String,
         pub target_var: String,
         pub group_vars: Vec<String>,
     }
     ```
   - For the query result `Binding` table:
     - Group rows based on the values of the `group_vars`.
     - For each group, instantiate an accumulator corresponding to `function`.
     - Feed the values of `source_var` for that group into the accumulator.
     - Query `accumulator.get()` to obtain the final aggregated term (encoded as a `usize`).
     - Construct the head triple using the group variables and the aggregated term.

---

## 6. Verification and Build Fixes

### Compiler Bug in `lib/src/oxrdf_adapter.rs`
During investigation, a critical build issue was identified in `lib/src/oxrdf_adapter.rs:93`:
```rust
        #[cfg(feature = "rdf-12")]
        oxrdf::Term::Triple(_) => panic!("RDF-star Triple terms are not supported by roxi"),
```
Because `roxi` itself does not have a feature named `rdf-12`, the match arm is compiled out. However, `oxrdf` is imported with the `rdf-12` feature enabled in `lib/Cargo.toml`, so `oxrdf::Term::Triple` is present in the dependency. This leads to a `non-exhaustive patterns` error during build.

**Recommended Fix**: Remove the `#[cfg(feature = "rdf-12")]` attribute from the `Triple` match arm in `oxrdf_adapter.rs` (or replace it with a wildcard `_ => panic!(...)` arm) so that it compiles successfully when `oxrdf` has `rdf-12` enabled.

### Test Execution Command
Once the changes are drafted, the suite can be verified with:
```bash
cargo test --workspace --lib --bins datalog
```
This will run the conformance tests in `lib/tests/datalog_negation.rs` and `lib/tests/datalog_conformance.rs` validating safe/unsafe rules, mutual recursion, stratification cycles, negation across strata, and aggregations.
