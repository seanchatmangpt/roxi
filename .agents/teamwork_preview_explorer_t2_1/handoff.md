# Handoff Report — TICKET-002: Production Cycle Guard in Backward Chainer

This report synthesizes the investigation and proposed design for cycle-guarding backward chaining query evaluation in `roxi`.

## 1. Observation

* **Recursive call site in `eval_backward`**:
  In `lib/src/backwardchaining.rs`, `BackwardChainer::eval_backward` performs an unconditional recursive call to solve body subgoals at lines 27-29:
  ```rust
  //recursive call
  let recursive_bindings = Self::eval_backward(triple_index, rule_index, rule_atom);
  rule_bindings.combine(recursive_bindings);
  ```
  No history tracking or cycle detection exists on this path.

* **Cycle Guard pattern in `service_composition.rs`**:
  In `lib/src/service_composition.rs:17`, a test-only variant threads a history list through recursive calls:
  ```rust
  pub fn eval_backward_with_history(triple_index: &TripleIndex, rule_index: &RuleIndex, rule_head: &Triple, history:  &mut Vec<Triple>) -> Binding {
  ```
  However, this checks against `visited_triples` derived from database queries (lines 29-34) and does not protect against infinite rule loops when facts do not match, causing it to fail for pure cyclic rules with empty/unmatching data.

* **Failing test case**:
  Running `cargo test --lib backwardchaining` initially fails on `test_cyclic_rules_terminate` (skeleton defined at lines 124-176 of `lib/src/backwardchaining.rs`) with:
  ```
  thread 'backwardchaining::tests::test_cyclic_rules_terminate' (270505) panicked at lib/src/backwardchaining.rs:137:13:
  TICKET-002: Production Cycle Guard in Backward Chainer is not yet implemented. Set is_implemented to true once implemented.
  ```

* **Fragility in parser rule body splitting**:
  When formatting rule definitions like:
  ```rust
  let data = "{?a <http://example.com/foo> ?b.} => {?b <http://example.com/foo> ?a.}.";
  ```
  `Parser::parse` (defined in `lib/src/parser.rs:93`) splits rule bodies using `.split(".")` (line 105). Any dot in a URI (e.g. `.com` or `.org`) will split the URI itself into pieces, causing a parsing crash in `parse_triple` with:
  ```
  thread 'backward_chaining_cycle_test' panicked at lib/src/parser.rs:74:33:
  called `Option::unwrap()` on a `None` value
  ```
  This is avoided by formatting URIs without dots (e.g., `<http://example/foo>`) in the test case.

## 2. Logic Chain

1. Since `BackwardChainer::eval_backward` makes recursive calls to evaluate body atoms (`rule_atom`) of matched rules without checking the call history, evaluating cyclic rules (such as `?a foo ?b :- ?b foo ?a`) results in infinite recursion and a stack overflow.
2. A DFS cycle-detection visited-set (using a `HashSet<Triple>`) can record subgoals currently in the derivation path.
3. If a subgoal (`rule_head`) is already present in the active path `history` (i.e. `history.insert` returns `false`), a cycle has occurred. In this case, query evaluation should immediately backtrack and return empty bindings (`Binding::new()`) instead of recursing further.
4. When backtracking out of a subgoal evaluation, the subgoal must be removed from `history` so that sibling branches can evaluate it independently.
5. In order to use a `HashSet<Triple>`, `Triple` and its inner fields `VarOrTerm`, `Variable`, and `Term` must derive `Hash` and `Eq`. Checking `lib/src/triples.rs` confirms they all already derive `Debug, Clone, Eq, PartialEq, Hash`.
6. Refactoring the public `eval_backward` function to initialize a new `HashSet<Triple>` and delegating to an internal recursive helper function `eval_backward_inner` preserves the public API signature of the chainer while adding cycle safety.

## 3. Caveats

* **Variable Renaming across Rules**:
  The cycle guard checks structural equivalence of the `Triple` subgoals (variable IDs and terms). In Roxi, rules are parsed once and their variable IDs are fixed in the `RuleIndex`. Therefore, recursive visits to the same rule body atom will hit the exact same variable IDs, triggering cycle detection. 
  If a rule set dynamically constructs/adds rules with fresh variable IDs representing the same query pattern, simple `HashSet` matching might not match the IDs immediately. However, under the current static parsing and execution model, this is not an issue.

## 4. Conclusion & Implementation Proposal

We propose implementing the production cycle guard in `BackwardChainer::eval_backward` as follows:

### API Refactoring & Wrap Strategy
Keep the existing public signature and introduce an internal recursive helper:

```rust
impl BackwardChainer {
    pub fn eval_backward(triple_index: &TripleIndex, rule_index: &RuleIndex, rule_head: &Triple) -> Binding {
        let mut history = std::collections::HashSet::new();
        Self::eval_backward_inner(triple_index, rule_index, rule_head, &mut history)
    }

    fn eval_backward_inner(
        triple_index: &TripleIndex,
        rule_index: &RuleIndex,
        rule_head: &Triple,
        history: &mut std::collections::HashSet<Triple>,
    ) -> Binding {
        // Cycle detection check
        if !history.insert(rule_head.clone()) {
            return Binding::new();
        }
        
        let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> = Self::find_subrules(rule_index, rule_head);
        let mut all_bindings = Binding::new();
        for (sub_rule, var_subs) in sub_rules.into_iter() {
            debug!("Backchainging rule: {:?}", TripleStore::decode_rule(&sub_rule));
            let mut rule_bindings = Binding::new();
            for rule_atom in &sub_rule.body {
                debug!("Matching body: {:?}", TripleStore::decode_triple(rule_atom));

                if let Some(result_bindings) = triple_index.query(rule_atom, None) {
                    debug!("   Found matching body: {:?}", TripleStore::decode_bindings(&result_bindings));
                    rule_bindings = rule_bindings.join(&result_bindings);
                }
                
                // Recurse with history
                let recursive_bindings = Self::eval_backward_inner(triple_index, rule_index, rule_atom, history);
                rule_bindings.combine(recursive_bindings);
            }
            // Rename variables
            let renamed = rule_bindings.rename(var_subs);
            all_bindings.combine(renamed);
        }
        
        // Remove from history on backtrack
        history.remove(rule_head);
        all_bindings
    }
}
```

### Test case update in `lib/src/backwardchaining.rs`
1. Reformat the rule data string to avoid parsing errors (remove dots from URIs and remove leading spaces).
2. Set `is_implemented` to `true`.
3. The test uses a timeout of 500ms via `mpsc::channel` to ensure termination of cyclic rules.

```rust
    #[test]
    fn test_cyclic_rules_terminate() {
        let data = "{?a <http://example.com/foo> ?b.}=>{?b <http://example.com/foo> ?a.}";
        // Convert URI to dotless version to avoid Parser::parse bug:
        let data = "{?a <http://example/foo> ?b.}=>{?b <http://example/foo> ?a.}";

        let is_implemented = true;
        
        let (tx, rx) = std::sync::mpsc::channel();
        let builder = std::thread::Builder::new()
            .name("backward_chaining_cycle_test".to_string());

        let data_str = data.to_string();
        let handle = builder.spawn(move || {
            let store = TripleStore::from(&data_str);
            let backward_head = Triple::from(
                "?x".to_string(),
                "<http://example/foo>".to_string(),
                "?y".to_string(),
            );
            let bindings = BackwardChainer::eval_backward(
                &store.triple_index,
                &store.rules_index,
                &backward_head,
            );
            tx.send(bindings.len()).unwrap();
        }).expect("failed to spawn evaluation thread");

        match rx.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok(len) => {
                assert!(len >= 0);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                panic!("Test failed: Backward chainer evaluation hung / did not terminate within timeout.");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let join_res = handle.join();
                panic!("Test failed: Backward chainer evaluation thread crashed (likely stack overflow): {:?}", join_res);
            }
        }
    }
```

## 5. Verification Method

To verify the implementation of this ticket:
1. Apply the patch `/Users/sac/roxi/.agents/teamwork_preview_explorer_t2_1/backwardchaining.patch`.
2. Run:
   ```bash
   cargo test --lib backwardchaining
   ```
   All tests (`test`, `test_eval_backward_rule`, and `test_cyclic_rules_terminate`) should compile and pass successfully.
