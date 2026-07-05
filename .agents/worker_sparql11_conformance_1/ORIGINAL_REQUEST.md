## 2026-07-05T02:50:39Z

You are the Worker agent for the SPARQL 1.1 Conformance milestone (TICKET-006).
Your working directory is `/Users/sac/roxi/.agents/worker_sparql11_conformance_1`.

Please implement the following:

1. Test Harness and Cases:
   - Set up a data-driven test runner in `lib/tests/sparql11_conformance/` by replacing the mock `main.rs` with the generic runner (`proposed_test_runner.rs` from `/Users/sac/roxi/.agents/explorer_sparql11_conformance_1/`). Note: You can add `serde` and `serde_json` to `[dev-dependencies]` in `lib/Cargo.toml` to support JSON deserialization in the test runner.
   - Copy/write `proposed_manifest.json` from the explorer's folder to `lib/tests/sparql11_conformance/manifest.json`.
   - Create the 10 test case subdirectories under `lib/tests/sparql11_conformance/` (`filter_ebv`, `filter_equality`, `filter_logical`, `bind_simple`, `optional_simple`, `optional_unbound`, `union_simple`, `minus_simple`, `subquery_simple`, `aggregate_multiple`) and populate them with their respective `data.ttl`, `query.rq`, and `expected.json` files as specified in the JIRA ticket and explorer analysis.

2. Code Fixes in `lib/src/sparql.rs`:
   - Fix variable prefixes: Standardize variable encoding in the `Encoder` so that lookups (e.g. `Encoder::get`) do not mismatch due to the presence/absence of the `?` prefix. For instance, you can strip `?` when registering or retrieving variables, or make sure they match consistently.
   - Fix Join logic:
     - Resolve the empty intersection panic (perform Cartesian product instead of returning empty/panicking).
     - Support joining on multiple shared variables (check match on all shared variables, not just `intersection.get(0)`).
   - Fix Aggregate logic:
     - Loop over all requested aggregates in `PlanNode::Aggregate`, keeping a list of accumulators and evaluating all of them.
     - Handle empty aggregates (plain GROUP BY with no COUNT/SUM/etc. projection).
   - Implement missing patterns in `PlanNode`:
     - Add `LeftJoin { left: Box<Self>, right: Box<Self>, expression: Option<PlanExpression> }` for `OPTIONAL`.
     - Add `Union { left: Box<Self>, right: Box<Self> }` for `UNION`.
     - Add `Minus { left: Box<Self>, right: Box<Self> }` for `MINUS`.
     - Map them in `extract_query_plan`.
   - Write corresponding iterators in `evaluate_plan` for `LeftJoin`, `Union`, and `Minus`.
     - Union: Flat map or chain left and right iterators.
     - LeftJoin: Evaluate left, and for each row, find matching rows in right (using join key intersection). If a match is found and passes any filter expression, output the merged binding. If no matching row exists, output the left binding (with right-only variables unbound/omitted).
     - Minus: For each row in left, filter out rows that have compatible bindings in right sharing at least one variable.
   - Fix `PlanNode::Extend` to append the evaluated binding to the existing binding vector instead of replacing it.
   - Support `Expression::Equal`, `Expression::NotEqual`, `Expression::And`, `Expression::Or`, `Expression::Not` in `extract_expression` and `eval_expression`. Apply Effective Boolean Value (EBV) using `to_bool` during filtering.

3. Compilation and Testing:
   - Run `cargo test --test sparql11_conformance` to verify your changes. Make sure all 10 conformance tests pass!
   - Write your handoff.md under `/Users/sac/roxi/.agents/worker_sparql11_conformance_1/`.

MANDATORY INTEGRITY WARNING:
> DO NOT CHEAT. All implementations must be genuine. DO NOT
> hardcode test results, create dummy/facade implementations, or
> circumvent the intended task. A Forensic Auditor will independently
> verify your work. Integrity violations WILL be detected and your
> work WILL be rejected.
