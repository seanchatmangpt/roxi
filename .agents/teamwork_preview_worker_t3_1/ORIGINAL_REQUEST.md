## 2026-07-05T02:49:35Z
You are a Worker. Your task is to implement TICKET-003 in roxi.
Your working directory is: /Users/sac/roxi/.agents/teamwork_preview_worker_t3_1

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.

Scope & Strategy:
1. Read the Explorer's handoff report at `/Users/sac/roxi/.agents/teamwork_preview_explorer_t3_1/handoff.md` and the patch at `/Users/sac/roxi/.agents/teamwork_preview_explorer_t3_1/csprite.patch`.
2. Implement the cycle guards in `lib/src/csprite.rs`:
   - Extend `eval_backward_csprite` to instantiate a `HashSet<Triple>` called `history`.
   - Update `eval_backward_csprite_helper` to take `history` as a mutable reference and perform DFS visited-set insertion/removal.
   - Refactor `eval_backward_csprite_helper_with_stack` to use a `StackFrame` enum (with `Enter` and `Exit` frames) and track `history` to prevent cycle-based loops.
3. Update `test_csprite_cycles_terminate` in `lib/src/csprite.rs` to set `is_implemented = true` and ensure the test case compiles and passes.
4. Run `cargo test --lib csprite` to verify that all csprite tests compile and pass successfully.
5. Document all changes and test outputs in `/Users/sac/roxi/.agents/teamwork_preview_worker_t3_1/handoff.md`.
