## 2026-07-04T19:37:10-07:00
You are a Worker. Your task is to implement TICKET-001 in roxi.
Your working directory is: /Users/sac/roxi/.agents/teamwork_preview_worker_t1_1

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT
hardcode test results, create dummy/facade implementations, or
circumvent the intended task. A Forensic Auditor will independently
verify your work. Integrity violations WILL be detected and your
work WILL be rejected.

Scope & Strategy:
1. Read the Explorer's handoff report at `/Users/sac/roxi/.agents/teamwork_preview_explorer_t1_1/handoff.md` and the JIRA ticket specification at `/Users/sac/roxi/docs/jira/26.7.4/001-term-model-literal-blanknode.md`.
2. Implement the `Term`, `LiteralImpl`, `BlankNodeImpl` structures in `lib/src/triples.rs` and extend `VarOrTerm` to use `Term`.
3. Implement `EncodedValue` and update `Encoder` / `InternalEncoder` in `lib/src/encoding.rs` to tag each entry with its kind. Provide methods to add, decode, and roundtrip IRIs, Literals, and Blank Nodes correctly.
4. Update all match and field access sites throughout the codebase (including `tripleindex.rs`, `reasoner.rs`, `dred.rs`, `sparql.rs`, and any other file flagged by the compiler) to resolve compiler errors resulting from the changes.
5. In `sparql.rs`, update literal expression interning and evaluation to correctly parse/eval encoded literals.
6. Write the requested verification tests under `triples.rs` and `encoding.rs`:
   - `test_literal_term_roundtrip`
   - `test_blank_node_term_encoding`
   - `test_encoder_literal_vs_iri_distinct`
   - `test_literal_datatype_and_langtag_preserved`
7. Run `cargo test --lib --bins` to verify all tests compile and pass.
8. Document all implemented changes and compile/test output in `/Users/sac/roxi/.agents/teamwork_preview_worker_t1_1/handoff.md`.
