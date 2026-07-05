# TICKET-005 — N3: full grammar, built-ins, quantifiers, and EYE conformance suite

**Status**: Not started
**Size**: XL
**Depends on**: 001 (literals), 004 (negation reuse, `Rule` shape coordination)

## Problem

The N3 grammar (`lib/src/parser/n3.pest`) and parser (`lib/src/parser/n3rule_parser.rs`) support only: `@prefix` declarations, triple patterns inside `{}`, the `a` (rdf:type) shorthand, `?Var` variables, `<iri>` and `prefix:local` terms, and a single `{body}=>{head}` implication restricted to **exactly one** head triple (the grammar's `Head` production is `TP`, not `TP+`).

Missing, all now **in scope** per the v26.7.4 zero-gaps commitment (previously considered candidates for deferral — that is no longer the plan):
- Literals (strings, numbers, langtags, datatypes) and blank nodes (`_:x`, `[...]`)
- Multi-triple rule heads
- `@forAll`/`@forSome` quantifiers with real scoping semantics
- Quoted graphs as terms (`{...}` used as a term, required for `log:` built-ins reasoning about formulae)
- RDF lists (`(...)`)
- Built-in predicates: `log:`, `math:`, `list:`, `string:`
- Backward implication `<=`
- Comments (`#`)
- Negation

Additionally, `n3rule_parser.rs:71` calls `.expect("Unable to read")` on the parse result — a malformed rules file panics the process instead of returning a `Result`.

## Implementation approach

1. **Parser robustness (do first, low risk)**: change `parse()`'s signature to `Result<Rule, ParseError>` (new error type), replace the `.expect(...)` at line 71 and any other panicking paths.
2. **Multi-triple heads**: rewrite `Head`/`Body` grammar productions from `TP` to `TP+`. Decompose a multi-triple head into N separate `Rule`s **at parse time** (each sharing the same body) — this is the simplest correct approach since it reuses `Reasoner`/`RuleIndex` unchanged rather than teaching the engine genuine multi-head rules. Coordinate with TICKET-004, which also touches `Rule`'s shape (adding `BodyLiteral` polarity) — land 004 first or in the same review window.
3. **Literals/blank nodes/lists**: add grammar productions for quoted strings, langtag suffix, `^^datatype` suffix, numeric literals, `_:label` blank nodes, and `(...)` lists — these feed TICKET-001's new `Term::Literal`/`Term::BlankNode` variants.
4. **Quantifiers**: parse `@forAll`/`@forSome` with real scoping (each quantified variable's scope is the rule it's declared in) — not parsed-and-ignored, per the zero-gaps commitment.
5. **Quoted graphs as terms**: extend the term grammar so `{...}` can appear as a subject/object, needed for `log:includes`/`log:notIncludes`-style built-ins.
6. **Built-ins**: extend `SimpleQueryEngine`/`Reasoner` to special-case a known set of built-in predicate IRIs (`log:`, `math:`, `list:`, `string:` namespaces) during rule-body matching, evaluating them procedurally instead of treating them as ordinary triple-pattern lookups against `TripleIndex`.
7. **Backward implication / comments**: `<=` as sugar swapping body/head at parse time; `#` to end-of-line as grammar-level whitespace/comment.
8. **Negation**: rule bodies reuse TICKET-004's `BodyLiteral { negated, pattern }` — do not invent a second negation mechanism for N3.
9. **Conformance suite**: vendor the public EYE reasoner N3 test corpus into `lib/tests/n3_conformance/` (there is no W3C REC test suite for N3 itself, so this is the community-standard bar) with a manifest documenting suite provenance and per-case coverage.

## Definition of Done

- `lib/tests/n3_parser.rs`: `test_parse_multi_triple_head`, `test_parse_literal_terms`, `test_parse_blank_nodes`, `test_parse_lists`, `test_parse_quantifiers`, `test_parse_quoted_graphs`, `test_parse_backward_implication`, `test_parse_comments_ignored`, `test_parse_error_returns_result_not_panic`.
- `lib/tests/n3_builtins.rs`: one test per supported `log:`/`math:`/`list:`/`string:` built-in.
- `lib/tests/n3_conformance/`: the vendored EYE/N3 test cases running end-to-end through parse + reason, zero `#[ignore]`d, with a manifest.
- `cargo test --workspace --lib --bins n3` passes.
