# Plan - TICKET-002 Implementation

## Steps
1. **Analyze backwardchaining.rs**: Re-read the file and verify structural compatibility.
2. **Implement eval_backward_inner**: Introduce an internal recursive helper function that takes `history: &mut HashSet<Triple>` and implements cycle detection.
3. **Wrap eval_backward**: Modify the public `eval_backward` to initialize `HashSet::new()` and call `eval_backward_inner`.
4. **Add test_cyclic_rules_terminate**: Write the unit test `test_cyclic_rules_terminate` inside `mod tests` of `lib/src/backwardchaining.rs`. Set `is_implemented = true` and verify termination. Use a dotless URI for example/foo rules to avoid the parser's dot-split bug.
5. **Verify with cargo test**: Run `cargo test --lib backwardchaining` to verify all tests in `backwardchaining` pass.
6. **Verify workspace-wide**: Run `cargo test` across the workspace to ensure no regressions.
7. **Document**: Write the handoff report in `/Users/sac/roxi/.agents/teamwork_preview_worker_t2_1/handoff.md`.
