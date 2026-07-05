# Breakage Plan — Failure Points as Proof-of-Correctness (TPS/DFLSS)

## Purpose

This document inverts the usual "tests confirm it works" framing. Under a TPS (Toyota Production System) / DFLSS (Design for Lean Six Sigma) lens, a passing test you wrote to match your own implementation proves nothing — it's tautological. The only signal that actually counts is a **real, external, spec-derived test breaking** against your implementation: that's the andon cord, the moment a defect surfaces instead of being silently absorbed.

Every failure point below is evidence-backed: real file:line, real before/after code, real external test case (W3C data-shapes suite, W3C shexTest, or a hard `todo!()` panic), not a self-authored mock. Historical ones (1–4) are confirmed via `git show 40d52f5:<file>` (the pre-session baseline) diffed against current `HEAD`. The rest (5–9) were found and fixed within this visible session, with full diff detail.

## Failure-mode ledger

Each entry: **Failure Mode** (what silently misbehaved) → **Detection Mechanism** (the real, external signal that caught it) → **Root Cause** → **Countermeasure** (poka-yoke: the fix that makes the class of defect structurally harder to reintroduce) → **Standing Verification** (what re-runs this check today).

---

### 1. SHACL `sh:and` over-propagation

- **Failure Mode**: a focus node failing a `sh:and` conjunction reported one violation *per failing sub-shape* plus a top-level and-violation, instead of exactly one `AndConstraintComponent` result per non-conforming focus node.
- **Detection Mechanism**: W3C `data-shapes` suite case `node/and-001.ttl` — expects exactly 1 result; baseline logic (`shacl.rs:1001-1012` @ `40d52f5`) produced N+1.
- **Root Cause**: `validate_shape` recursed into each sub-shape, collected `sub_results` into a `Vec`, then did `results.extend(sub_results)` *in addition to* pushing the and-violation — conflating "why did it fail" (useful for `sh:not`, where sub-results matter) with "did it fail" (all `sh:and` needs).
- **Countermeasure**: replaced with `conforms_to_shape` (a boolean check, no result leakage) — `sub_shapes.iter().all(|&sub| conforms_to_shape(...))`, pushing exactly one result if `!conforms`. (`shacl.rs:1355-1362`, current.)
- **Standing Verification**: `shacl_conformance::test_w3c_core_constraint_component_suite` (node/and-001, node/and-002) + `shacl_stress::test_counterfactual_contradictory_and_never_conforms`.

### 2. SHACL `sh:datatype` missing lexical well-formedness

- **Failure Mode**: `"aldi"^^xsd:integer` — a literal whose *declared* datatype matched but whose *lexical form* was not a valid instance of that datatype — passed `sh:datatype` validation.
- **Detection Mechanism**: W3C suite `property/datatype-002.ttl`/`datatype-003.ttl` (ill-formed-lexical cases) expect a `DatatypeConstraintComponent` violation; baseline `check_datatype` (`shacl.rs:560-562` @ `40d52f5`) only checked datatype-IRI equality, never the lexical form.
- **Root Cause**: `check_datatype` was `get_datatype(x).map_or(false, |dt| dt == expected_dt)` — datatype tag comparison only, no XSD lexical-space validation.
- **Countermeasure**: added `is_lexically_valid_for_datatype` (`shacl.rs:597-627`) — real per-datatype lexical rules (integer family: digit-only after optional sign; decimal: digits+one `.`; double/float: `INF`/`-INF`/`NaN`/parseable; boolean: `true`/`false`/`1`/`0`), wired into `check_datatype` (`shacl.rs:633-641`).
- **Standing Verification**: `shacl_conformance` suite (property/datatype-001/002/003, node/datatype-001/002).

### 3. SHACL `sh:uniqueLang` counting duplicate *values* instead of duplicate *language tags*

- **Failure Mode**: 3 values sharing the same language tag produced **2** violation results (one per duplicate-beyond-the-first), not the spec-correct **1** result per offending language tag.
- **Detection Mechanism**: W3C suite `property/uniqueLang-001.ttl` asserts an exact violation count; baseline (`shacl.rs:1244-1261` @ `40d52f5`) pushed a result inside the per-value loop on every repeat occurrence.
- **Root Cause**: the loop pushed a `ValidationResult` at the moment a *second* (or third, or Nth) value with an already-seen tag was encountered — a per-occurrence counter, not a per-distinct-tag counter.
- **Countermeasure**: two-pass approach — first tally `lang_counts: HashMap<tag, count>` across all value nodes, then push exactly one result per tag whose count exceeds 1 (`shacl.rs:1685-1706`, current).
- **Standing Verification**: `shacl_conformance` suite (property/uniqueLang-001).

### 4. SHACL `sh:conforms` severity semantics (found twice — once introduced, once caught)

- **Failure Mode**: a report with only `sh:Info`/`sh:Warning`-severity results (no `sh:Violation`) was computed as `conforms = true`.
- **Detection Mechanism**: W3C suite `misc/severity-001.ttl`/`severity-002.ttl` — a shape declaring `sh:severity sh:Warning` still expects `sh:conforms "false"`.
- **Root Cause**: SHACL Core (1.0) defines conforms as false whenever *any* result exists, regardless of severity (SHACL 1.2's `sh:conformanceDisallows` opt-in narrowing is a distinct, newer mechanism this validator doesn't implement). A "fix" applied earlier in this session's uncommitted work changed `conforms = results.is_empty()` to `conforms = !results.iter().any(|r| r.severity == sh:Violation)` — this was itself a regression, based on a plausible-but-wrong reading of the spec.
- **Countermeasure**: reverted to `conforms = results.is_empty()` (`shacl.rs`, `Validator::validate`) with a doc comment citing the real spec text and the two vendored `misc/severity-*` cases as evidence. A pre-existing hand-authored test (`shacl_validation.rs::test_severity_and_datatype`) that encoded the *wrong* assumption was corrected in the same pass — this is the sharpest example in the ledger of a self-authored test actively reinforcing a bug until a real external suite contradicted it.
- **Standing Verification**: `shacl_conformance` suite (misc/severity-001/002) + corrected `shacl_validation::test_severity_and_datatype`.

### 5. SHACL `xsd:dateTime` range comparison — silently always-fails

- **Failure Mode**: `sh:minInclusive`/`maxInclusive`/etc. against `xsd:dateTime` values *always* violated, even values that should conform, because the comparator only attempted `f64` parsing.
- **Detection Mechanism**: expanding the vendored W3C suite from 23→60 cases surfaced `node/minInclusive-002.ttl`/`minInclusive-003.ttl` (dateTime-with-timezone and dateTime-without-timezone cases) failing with *5 violations* where 3 were expected — i.e., real conforming values were being wrongly flagged.
- **Root Cause**: `compare_numeric` (`shacl.rs`) had no dateTime parsing path at all; `get_numeric_value` returning `None` for any non-numeric lexical silently fell through to "not comparable → violation" for every dateTime pair, including ones that should have compared fine.
- **Countermeasure**: added `parse_datetime` (ISO-8601 + optional timezone, civil-to-days algorithm) and extended `compare_numeric` to fall back to it; a timezone-qualified value compared against a timezone-less one is correctly treated as XSD's "indeterminate" (violates, per spec and the real test's expectation), not silently coerced either way.
- **Standing Verification**: `shacl_conformance` suite (node/minInclusive-002/003) + dedicated `shacl_stress::test_datetime_timezone_mismatch_is_indeterminate_and_violates` (isolates the exact behavior independent of the vendored suite).

### 6. SHACL blank-node string-facet leak (`sh:minLength`/`sh:maxLength`)

- **Failure Mode**: a blank node reached via `sh:targetClass` silently satisfied `sh:minLength`/`maxLength` because its *internal implementation-detail label* (e.g. `"b19019"`) was used as its "lexical form."
- **Detection Mechanism**: expanded suite case `node/minLength-001.ttl` expected 4 violations, got 3 — the blank-node case was missing.
- **Root Cause**: `get_lexical_form` returned `Some(label)` for `Term::BlankNode` unconditionally; the minLength/maxLength code paths treated any `Some(_)` as "has a real string value."
- **Countermeasure**: new `get_string_representation` helper returns `None` for blank nodes specifically (IRIs and literals still resolve normally); minLength/maxLength at both node- and property-level now treat `None` as "no string representation → always violates," per spec, instead of skipping or defaulting to length 0.
- **Standing Verification**: `shacl_conformance` suite (node/minLength-001).

### 7. SHACL nested `sh:property` — entirely unimplemented

- **Failure Mode**: a property shape whose value itself has its own nested property shape (`sh:property [ sh:path ex:address ; sh:property ex:AddressCityShape ]`) produced **zero** violations regardless of data — the nested constraint was never evaluated at all.
- **Detection Mechanism**: expanded suite case `property/property-001.ttl` expected `conforms=false` with 2 violations, got `conforms=true` with 0.
- **Root Cause**: `validate_shape`'s `sh:property` loop had no recursion for a property shape's own `sh:property` objects.
- **Countermeasure**: extracted the per-property-shape body into a standalone `validate_property_shape` function and added recursion: each value reached by the outer path is passed as the new focus node into the nested property shape.
- **Standing Verification**: `shacl_conformance` suite (property/property-001) + `shacl_stress::test_three_level_nested_property_shape` (stresses 3 levels of depth, beyond the suite's 2-level case).

### 8. N3 builtins — hard `todo!()` panics (the clearest possible failure signal)

- **Failure Mode**: `math:sum`, `math:greaterThan`, `string:concat`, `string:length`, `list:in`, `list:length`, `log:equalTo`, `log:implies`, and 10 further builtins didn't subtly misbehave — they panicked the process outright.
- **Detection Mechanism**: any conformance run touching these predicates crashed rather than reported a wrong answer — maximally loud, impossible to silently ship.
- **Root Cause**: builtin dispatch for the `math:`/`string:`/`list:`/`log:` namespaces didn't exist in `SimpleQueryEngine::query`, and `log:implies` (requiring runtime quoted-graph reification) had no data model to reify against.
- **Countermeasure**: added a full RDF-list + quoted-graph term model (`triples.rs`: `LIST_REGISTRY`/`FORMULA_REGISTRY`), builtin dispatch (functional + row-filter modes) in `queryengine.rs`, and dynamic rule reification for `log:implies`/`log:collectAllIn`/`log:notIncludes` in `reasoner.rs`.
- **Standing Verification**: `n3_builtins.rs` (12 tests, all previously-panicking predicates), `n3_conformance` manifest (30 cases, 22 active), `n3_scoping.rs` (nested-formula isolation, chained implication, chained `log:implies → log:implies`).

### 9. Datalog stratification — untested against non-adjacent/reconvergent cycle topologies

- **Failure Mode**: not an observed defect (the whole-graph Bellman-Ford relaxation in `datalog.rs` was already structurally sound), but an **unverified-trust gap**: only linear-chain-plus-far-edge and simple self-negation cycles had test coverage, leaving reconvergent (diamond) and multiple-simultaneous cycle topologies unproven.
- **Detection Mechanism**: none broke — but that's exactly the point of a poka-yoke audit: absence of a failure doesn't mean absence of a gap, and this is the one failure point in the ledger that was closed *before* it could ever produce a real breakage.
- **Root Cause**: N/A (preventive, not corrective).
- **Countermeasure**: added `test_diamond_reconvergence_rejected` (4-predicate cycle closed entirely through negation, reached via reconvergent paths) and `test_multiple_disjoint_cycles_rejected` (two independent unstratifiable cycles plus one safe rule, confirming both are caught).
- **Standing Verification**: `datalog_stress.rs`.

## What this ledger deliberately excludes

Per DFLSS discipline against manufactured confidence: this ledger does **not** include any case where a self-authored test was written and passed on the first try without ever having failed against real behavior first (e.g. most of the counterfactual/stress tests added this session that passed immediately — those *demonstrate* correctness but were never breakage events). Item 9 is flagged explicitly as the one preventive (non-corrective) entry, not conflated with the eight real corrective ones.

## Next: dynamic andon workflow

See the accompanying `Workflow` run — a parallel regression-andon check that re-verifies all 9 failure points concurrently against the current `HEAD`, so a future regression on any of these nine specific historical defects is caught immediately rather than silently reintroduced.
