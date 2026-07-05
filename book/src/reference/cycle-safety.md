# Cycle Safety & Visited Guards in Roxi

## 1. The Recursion Problem in Backward Chainers

Backward-chaining reasoning engines compute facts on-demand by starting with a query goal and recursively evaluating rules whose heads match that goal. While this approach avoids the upfront memory costs of forward materialization, it faces a fundamental challenge: **infinite recursion**.

In semantic graphs, recursive relationships are common (e.g., family trees, networks, class hierarchies). Consider a simple transitive rule:
$$\text{connectedTo}(?X, ?Y) \leftarrow \text{linkedTo}(?X, ?Z), \text{connectedTo}(?Z, ?Y)$$

If the input graph contains a circular relationship (for example, $:A \xrightarrow{\text{linkedTo}} :B$ and $:B \xrightarrow{\text{linkedTo}} :A$), evaluating the goal query `?x :connectedTo :C` leads to the following execution sequence:

1. Match rule head: `?x :connectedTo :C` $\to$ resolve body subgoal `?z :connectedTo :C`.
2. First bind `?z` to `:B` (using `:A :linkedTo :B`). Resolve subgoal `:B :connectedTo :C`.
3. Match rule head for `:B :connectedTo :C` $\to$ resolve body subgoal `?z2 :connectedTo :C`.
4. Bind `?z2` to `:A` (using `:B :linkedTo :A`). Resolve subgoal `:A :connectedTo :C`.
5. The goal `:A :connectedTo :C` is identical to the step 1 goal, creating an infinite loop.

Without termination guards, this recursive loop causes a stack overflow (SIGSEGV) or hangs the process indefinitely.

---

## 2. Visited-Set History Guard Implementation

To guarantee termination and ensure stack safety, Roxi implements a **Visited-Set History Guard** in [lib/src/backwardchaining.rs](file:///Users/sac/roxi/lib/src/backwardchaining.rs).

During backward resolution, the engine threads a mutable `history` context containing all subgoals currently active in the execution stack.

```
       [Goal: ?x :connectedTo :C]
                  |
                  | (history: {})
                  v
       [Goal: :B :connectedTo :C]
                  |
                  | (history: {?x :connectedTo :C})
                  v
       [Goal: :A :connectedTo :C]
                  |
                  | (history: {?x :connectedTo :C, :B :connectedTo :C})
                  v
       Check History: Goal matches '?x :connectedTo :C' (Bound to :A)
                  |
                  +---> DETECTED CYCLE! Abort recursion & return empty bindings
```

---

## 3. Step-by-Step Traversal Protocol

1. **Subgoal Interception**: When a subgoal `G` is scheduled for evaluation, the engine verifies if `G` matches any goal already present in `history` (accounting for variable binding states).
2. **Short-Circuit**: If `G` is detected in `history`, the engine terminates the branch immediately and returns an empty set of bindings, preventing infinite loops.
3. **Push Step**: If `G` is not present, it is added to the `history` set, and evaluation proceeds recursively.
4. **Pop (Backtrack) Step**: When the recursive evaluation of `G` completes and the engine backtracks to explore alternative rule paths, `G` is removed from the `history` set. This ensures that independent execution branches are not blocked.

---

## 4. Rust Implementation Reference

Below is the Rust implementation of the cycle guard:

```rust
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Triple {
    pub s: usize,
    pub p: usize,
    pub o: usize,
}

pub struct Bindings {
    pub rows: Vec<Vec<usize>>,
}

pub struct BackwardChainer {
    // Rule index and store references
}

impl BackwardChainer {
    /// Public entry point for goal resolution
    pub fn resolve_query(&self, goal: &Triple) -> Bindings {
        let mut history = HashSet::new();
        self.resolve_goal_inner(goal, &mut history)
    }

    /// Internal recursive evaluation carrying the visited history set
    fn resolve_goal_inner(&self, goal: &Triple, history: &mut HashSet<Triple>) -> Bindings {
        // 1. Cycle Check
        if history.contains(goal) {
            // Cycle detected: short-circuit recursion and return empty bindings
            return Bindings { rows: Vec::new() };
        }

        // 2. Register current goal in history
        history.insert(goal.clone());

        let mut results = Bindings { rows: Vec::new() };

        // 3. Evaluate matching rules
        let matching_rules = self.find_matching_rules(goal);
        for rule in matching_rules {
            let subgoal_results = self.resolve_rule_body(&rule, history);
            results.extend(subgoal_results);
        }

        // 4. Remove goal from history during backtrack step
        history.remove(goal);

        results
    }

    fn find_matching_rules(&self, _goal: &Triple) -> Vec<Rc<Rule>> {
        // Mock implementation for reference
        Vec::new()
    }

    fn resolve_rule_body(&self, _rule: &Rule, _history: &mut HashSet<Triple>) -> Bindings {
        // Mock implementation for reference
        Bindings { rows: Vec::new() }
    }
}

pub struct Rule {
    pub body: Vec<Triple>,
    pub head: Triple,
}
```

---

## 5. Thread-Sandboxed Timeout Guards in CI

In the test runner [lib/tests/backwardchaining_cycles.rs](file:///Users/sac/roxi/lib/tests/backwardchaining_cycles.rs), cycle guards are validated using a thread-sandboxed safety mechanism. If an implementation error introduces an infinite recursion loop, the test thread is aborted after a 500ms timeout instead of hanging the entire CI pipeline:

```rust
#[test]
fn test_cyclic_rules_terminate() {
    let (tx, rx) = std::sync::mpsc::channel();
    
    // Spawn evaluation in a sandboxed thread
    std::thread::spawn(move || {
        let store = setup_cyclic_store();
        let result = store.query_backward(":A", ":connectedTo", "?y");
        let _ = tx.send(result);
    });

    // Enforce a strict 500ms timeout
    let test_timeout = std::time::Duration::from_millis(500);
    match rx.recv_timeout(test_timeout) {
        Ok(_) => {} // Test passed: evaluation terminated in bounded time
        Err(_) => panic!("Timeout: Cycle guard failed to terminate recursive loop!"),
    }
}
```
