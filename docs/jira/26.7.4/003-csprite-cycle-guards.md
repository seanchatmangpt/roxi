# TICKET-003 — Apply the same cycle guard in csprite.rs

**Status**: Not started
**Size**: S
**Depends on**: 002 (reuses its guard type/pattern)
**Blocks**: none

## Problem

`lib/src/csprite.rs` has four unresolved cycle-safety TODOs at lines 119, 127, 153, 154 (`//TODO check cycles`) — the identical class of gap fixed in TICKET-002, surfacing in a different traversal path (csprite's rule/hierarchy composition).

## Implementation approach

Reuse the exact guard type and threading pattern established in TICKET-002 (do not invent a second cycle-detection mechanism in this file). At each of the four flagged call sites, thread the same `history`/visited-set guard through the recursive traversal and short-circuit on repeat.

## Definition of Done

- New tests exercising each of the four flagged sites with a cyclic input, asserting termination (bounded time/depth, not a hang).
- `cargo test --workspace --lib --bins csprite` passes, including existing `test_sprite_compute`, `test_sprite_compute_hierarchy`, `test_rewrite_hierarchy_csprite`.
