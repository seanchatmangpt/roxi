# TICKET-010 — Cross-dialect conformance CI gate

**Status**: Not started
**Size**: M
**Depends on**: 004, 005, 006, 008, 009 (all five conformance suites must exist first)

## Problem

"100% specification-validated, no gaps" is only true at a single point in time unless it's continuously enforced. Without a gate, any future change to `sparql.rs`, `shacl.rs`, `shex.rs`, `n3rule_parser.rs`/`reasoner.rs`, or `datalog.rs` could silently regress conformance and no one would notice until a user hit it. There is currently no CI at all in this repo (`.github/workflows/` does not exist).

## Implementation approach

1. Add `.github/workflows/conformance.yml` running on every push/PR to the default branch, executing:
   - `cargo test --workspace --lib --bins sparql11_conformance`
   - `cargo test --workspace --lib --bins shacl_conformance`
   - `cargo test --workspace --lib --bins shex_conformance`
   - `cargo test --workspace --lib --bins n3_conformance`
   - `cargo test --workspace --lib --bins datalog_conformance`
   Fail the build on any failure in any of the five.
2. Publish each suite's pass-rate manifest (produced by tickets 004/005/006/008/009) as a build artifact.
3. Aggregate all five manifests into a single top-level status doc, `docs/jira/26.7.4/CONFORMANCE-STATUS.md`, as a table: dialect | suite | pass rate | last-updated | link to manifest.

## Definition of Done

- `.github/workflows/conformance.yml` exists and is green on the default branch.
- `docs/jira/26.7.4/CONFORMANCE-STATUS.md` exists and aggregates all five pass-rate manifests.
- `cargo test --workspace` (full suite, all five conformance modules included) passes in CI.
