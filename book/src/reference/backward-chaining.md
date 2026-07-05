# Backward Chainer

## On-Demand Inference

Roxi's backward chainer, located in [lib/src/backwardchaining.rs](file:///Users/sac/roxi/lib/src/backwardchaining.rs), executes on-demand reasoning. Instead of materializing all possible facts, it starts with a query goal and evaluates only the rules needed to resolve it.

This approach is highly memory-efficient, making it ideal for systems with large base datasets but specific, targeted queries.

---

## SLD Resolution

The engine uses SLD-style resolution to resolve subgoals:

1. **Goal Matching**: Matches the query goal (e.g., `?x :ancestorOf :bob`) against rule heads.
2. **Subgoal Decomposition**: Replaces the goal with the rule body patterns, turning them into new subgoals.
3. **Index Evaluation**: Looks up positive subgoals directly in the `TripleIndex`.
4. **Binding Union**: Joins the resulting bindings and propagates them back up the recursion tree.
