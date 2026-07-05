## 2026-07-05T02:34:58Z

You are an Explorer. Investigate the roxi codebase for TICKET-001.
Your working directory is: /Users/sac/roxi/.agents/teamwork_preview_explorer_t1_1
Specifically:
1. Inspect `lib/src/triples.rs` and `lib/src/encoding.rs`. Analyze the definitions of `VarOrTerm`, `TermImpl`, `Encoder`, and how terms are encoded/interned/decoded.
2. Locate all current usage sites of `VarOrTerm`, `TermImpl`, and `Encoder` in the codebase (e.g. `lib/src/tripleindex.rs`, `lib/src/ruleindex.rs`, `lib/src/queryengine.rs`, `lib/src/sparql.rs`, `lib/src/dred.rs`, `lib/src/parser/` etc.) to understand the impact of the proposed changes.
3. Propose a clear, detailed strategy for implementing TICKET-001, including:
   - The precise structures for `Term`, `LiteralImpl`, and `BlankNodeImpl`.
   - How `Encoder` should be extended to store and retrieve IRIs, Literals, and Blank Nodes (e.g., using a tagged kind, or separate maps, or prefixing interned strings, but keeping in mind the ticket requirements).
   - A list of all files and code blocks that will need to be updated due to matches on `VarOrTerm` and `TermImpl`.
4. Document your findings in `/Users/sac/roxi/.agents/teamwork_preview_explorer_t1_1/handoff.md`.
