# TICKET-002 — Promote cycle-guard into production backward chaining

**Status**: Not started
**Size**: S
**Depends on**: 001 (needs `Hash`+`Eq` on `Triple`/`Term`)
**Blocks**: 003, 004

## Problem

`BackwardChainer::eval_backward` (`lib/src/backwardchaining.rs:12`) recurses through `find_subrules` (line 38) with an unconditional recursive call at line 28 and **no visited-set guard**. A cyclic rule set (e.g. `?a foo ?b :- ?b foo ?a`) causes infinite recursion / stack overflow — there is no protection today.

The fix pattern already exists, just not in production code: `service_composition.rs`'s test-only `eval_backward_with_history` (line 17) threads a `history` vector through recursive calls and refuses to revisit a query already in the current derivation path.

## Implementation approach

1. In `lib/src/backwardchaining.rs`, add a `history: &mut HashSet<Triple>` parameter to `eval_backward` (or an internal `_inner` variant with a public wrapper that starts with an empty set, to avoid breaking the existing public signature more than necessary).
2. At the recursive call site (line 28), before recursing: if the query triple is already in `history`, return no bindings (cycle detected) rather than recursing.
3. Insert/remove from `history` around the recursive call (standard DFS visited-set discipline — remove on backtrack so sibling branches aren't falsely blocked).
4. Requires `Triple` (and transitively `Term`/`VarOrTerm` from TICKET-001) to derive `Hash + Eq` for `HashSet` membership — confirm derive is present, add if not.

## Definition of Done

- New test (in `backwardchaining.rs`'s `#[cfg(test)]` module, or a new `lib/tests/backwardchaining_cycles.rs`): `test_cyclic_rules_terminate` — construct a self-referential rule set and assert the call returns (with correct — likely empty or fixpoint-consistent — bindings) rather than hanging or overflowing the stack. Use a bounded-time test harness (e.g. wrap in a thread with a timeout, or assert on recursion depth if a depth counter is added) so a regression fails loudly instead of hanging CI.
- `cargo test --workspace --lib --bins backwardchaining` passes.
- Existing backward-chaining tests (`test_eval_backward_rule` etc.) continue to pass unmodified.
