# BRIEFING — 2026-07-04T19:49:30-07:00

## Mission
Explore and analyze TICKET-007: `oxrdf` adapter layer.

## 🔒 My Identity
- Archetype: explorer
- Roles: Teamwork explorer
- Working directory: /Users/sac/roxi/.agents/teamwork_preview_explorer_shape_validation_1
- Original parent: e614f6e1-e383-41a5-8b6d-d31da60f6e5c
- Milestone: TICKET-007

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Network mode: CODE_ONLY

## Current Parent
- Conversation ID: e614f6e1-e383-41a5-8b6d-d31da60f6e5c
- Updated: not yet

## Investigation State
- **Explored paths**:
  - `PROJECT.md`
  - `.agents/sub_orch_shape_validation/SCOPE.md`
  - `docs/jira/26.7.4/007-oxrdf-adapter-layer.md`
  - `lib/src/triples.rs`
  - `lib/src/encoding.rs`
  - `lib/src/tripleindex.rs`
  - `lib/Cargo.toml`
  - `lib/tests/oxrdf_adapter.rs`
  - Local cargo registry for `oxrdf-0.3.3` (especially `named_node.rs`, `blank_node.rs`, `literal.rs`, `triple.rs`, `graph.rs`, `lib.rs`)
- **Key findings**:
  - Completed term model changes (TICKET-001) in `triples.rs` and `encoding.rs` represent IRI, Literal, and BlankNode as first-class enum variants, which aligns with `oxrdf::Term`.
  - `oxrdf` version `0.3.3` is available with feature `rdf-12` enabled. We mapped all required conversion types: `NamedNode`, `Literal`, `BlankNode`, `NamedOrBlankNode`, `Term`, `Triple`, and `Graph`.
  - Simple helper functions `clean_iri` and `clean_blank_node` are required to strip `<...>` and `_:` prefixes during conversions from Roxi's internal string representation (stored inside the global `Encoder`) to `oxrdf` strings.
  - In the reverse conversion, the adapter must format IRIs with `<...>` and BlankNodes with `_:` before interning in the `Encoder`.
- **Unexplored areas**: None.

## Key Decisions Made
- Formulate the adapter module code inside `proposed_oxrdf_adapter.rs`.
- Propose additions to `triples.rs` (`VarOrTerm::new_literal` and `VarOrTerm::new_blank_node`) and `lib.rs` module registration inside `proposed_changes.patch`.
- Recommend uncommenting the tests in `lib/tests/oxrdf_adapter.rs` as-is.

## Artifact Index
- `proposed_oxrdf_adapter.rs` — Proposed implementation of the `oxrdf_adapter` module.
- `proposed_changes.patch` — Proposed patch for `lib/src/lib.rs`, `lib/src/triples.rs`, and `lib/tests/oxrdf_adapter.rs`.
