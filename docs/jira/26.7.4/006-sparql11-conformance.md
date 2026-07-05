# TICKET-006 — SPARQL 1.1: W3C conformance suite integration and gap closure

**Status**: Not started
**Size**: XL (initial run is a sizing spike — see below)
**Depends on**: 001 (literal/blank-node fidelity affects result-set comparison correctness)

## Problem

roxi has its own SPARQL evaluator (`lib/src/sparql.rs`, parsing delegated to `spargebra` 0.4.6) but it has **never been measured against the official W3C SPARQL 1.1 test suite**. Per the v26.7.4 zero-gaps commitment, SPARQL is explicitly in scope for 100% conformance alongside the other four dialects — it is not "assumed done" just because it predates this ticket set.

Known risk areas going in (from prior code reading, to be confirmed/quantified by the first suite run): `extract_query_plan` in `sparql.rs` has historically had catch-all fallback arms for unhandled `GraphPattern`/`Expression` variants; aggregate support (Count/Sum/Min/Max/Avg) was completed only recently; property paths, subqueries, federated `SERVICE`, and full `OPTIONAL`/`MINUS`/`EXISTS` semantics are plausible gap areas but must be confirmed empirically, not assumed.

## Implementation approach

1. **Vendor the suite**: pull the W3C `sparql11-test-suite` (query evaluation + update test manifests, data files, expected results) into `lib/tests/sparql11_conformance/`.
2. **Harness**: write a test runner that reads each test's manifest entry (query file, input data, expected result set or expected update effect), loads data via the existing `Parser`/`TripleStore` path, runs the query through `sparql::eval_query`/`evaluate_plan_and_debug`, and compares actual vs. expected results (accounting for SPARQL's unordered-by-default result semantics unless `ORDER BY` is present).
3. **Gap inventory (spike)**: run the full suite once against the current implementation before doing any fix work. Produce a categorized gap list (which language features fail, grouped by cause) — this determines the real remaining size of this ticket; do not commit to a fix plan before this data exists.
4. **Close gaps**: for each failing category, extend `PlanNode`/`PlanExpression`/`extract_query_plan`/`extract_expression` in `sparql.rs`, reusing existing plan-node variants wherever the feature is a variant of something already implemented, adding new ones only for genuinely missing features.
5. **Excluded tests**: SPARQL 1.1 itself marks some features optional (e.g. certain federated-query behaviors). Any test excluded from the 100% target must be justified in the manifest by citing the specific optional-feature clause — no silent exclusions.

## Definition of Done

- `lib/tests/sparql11_conformance/` runs the vendored suite and produces a pass-rate manifest (per test group and overall).
- 100% pass rate, or every excluded test has an explicit, spec-cited justification in the manifest.
- `cargo test --workspace --lib --bins sparql11_conformance` passes.

## Notes

Treat the first full suite run as the actual scoping exercise for this ticket — "XL" is a placeholder pending that data, not a final estimate.
