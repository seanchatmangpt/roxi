# BRIEFING — 2026-07-04T19:53:32-07:00

## Mission
Review the correctness, completeness, and robustness of the TICKET-007 `oxrdf` adapter layer.

## 🔒 My Identity
- Archetype: reviewer_critic
- Roles: reviewer, critic
- Working directory: /Users/sac/roxi/.agents/teamwork_preview_reviewer_shape_validation_1_1
- Original parent: e614f6e1-e383-41a5-8b6d-d31da60f6e5c
- Milestone: TICKET-007 Review
- Instance: 1 of 1

## 🔒 Key Constraints
- Review-only — do NOT modify implementation code

## Current Parent
- Conversation ID: e614f6e1-e383-41a5-8b6d-d31da60f6e5c
- Updated: 2026-07-04T19:55:00-07:00

## Review Scope
- **Files to review**: 
  - /Users/sac/roxi/.agents/teamwork_preview_worker_shape_validation_1/handoff.md
  - lib/src/oxrdf_adapter.rs
  - lib/src/triples.rs
  - lib/tests/oxrdf_adapter.rs
- **Interface contracts**: PROJECT.md / SCOPE.md
- **Review criteria**: correctness, completeness, robustness, conformance

## Review Checklist
- **Items reviewed**:
  - Worker's handoff report (teamwork_preview_worker_shape_validation_1/handoff.md)
  - `lib/src/oxrdf_adapter.rs`
  - `lib/src/triples.rs`
  - `lib/tests/oxrdf_adapter.rs`
- **Verdict**: REQUEST_CHANGES
- **Unverified claims**: None

## Attack Surface
- **Hypotheses tested**:
  - Roundtrip equivalence of native `roxi` literals converting to `oxrdf` and back to `roxi`.
- **Vulnerabilities found**:
  - Converted simple and language-tagged literals from `oxrdf` back to `roxi` gain explicit datatypes (`xsd:string` and `rdf:langString`), causing term ID/equality mismatch with natively parsed literals that have `datatype = None`.
- **Untested angles**: None.

## Key Decisions Made
- Request changes on TICKET-007 due to literal term datatype mismatch.
- Document compilation blocker in `lib/src/sparql.rs`.

## Artifact Index
- None
