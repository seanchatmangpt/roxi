# TICKET-005 — N3: full grammar, built-ins, quantifiers, and EYE conformance suite

**Status**: Done — verified 2026-07-05 (`cargo test --workspace` clean apart from 3 pre-existing, unrelated `imars_window.rs` doctest failures (crate-name mismatch, not touched by this ticket): `n3_parser.rs` 9/9, `n3_builtins.rs` 8/8, `n3_conformance` manifest-driven runner 14/14 *active* cases passing + 8 *ignored* cases each with a precise documented blocking reason (22 total manifest entries), `cargo test -p roxi --lib` 90/90 (7 pre-existing unrelated ignores))
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

- [x] `lib/tests/n3_parser.rs`: `test_parse_multi_triple_head`, `test_parse_literal_terms`, `test_parse_blank_nodes`, `test_parse_lists`, `test_parse_quantifiers`, `test_parse_quoted_graphs`, `test_parse_backward_implication`, `test_parse_comments_ignored`, `test_parse_error_returns_result_not_panic` — all 9 present and passing.
- [x] `lib/tests/n3_builtins.rs`: one test per supported `log:`/`math:`/`list:`/`string:` built-in — 8 tests covering `log:equalTo`, `log:implies`, `math:sum`, `math:greaterThan`, `list:in`, `list:length`, `string:concat`, `string:length`; all passing.
- [x] `lib/tests/n3_conformance/`: manifest-driven runner (`manifest.json` + `main.rs`) covering 7 syntax cases + 10 built-in cases + 5 *active* vendored EYE reasoner cases + 8 *ignored* (documented-blocked) vendored EYE reasoner cases = 30 manifest entries total, 22/22 active cases passing (independently re-verified by direct `cargo test --workspace --test n3_conformance -- --nocapture` run, not just the agent's self-report), manifest documented in `lib/tests/n3_conformance/README.md`. A dynamic-discovery guard in `main.rs` (mirroring `datalog_conformance.rs`'s directory-scan pattern) fails the suite if any `syntax/`/`builtins/`/`vendored/` `.n3` file with a `_result.n3` sibling isn't referenced by some manifest case, so newly vendored files can't silently go unrun.
- [x] `lib/tests/n3_builtins.rs` grown from 8 to 12 tests: added `test_math_builtins_difference_product_quotient_remainder`, `test_math_builtins_comparison_constraints`, `test_log_builtin_not_equal_to`, `test_list_builtin_append` — closing a gap where the 10 new arithmetic/comparison/list builtins added below were implemented but had zero unit-level test coverage (only exercised, thinly, via 2 of the vendored/builtin manifest cases). All 12 independently re-run and confirmed passing.
- [x] `cargo test --workspace --lib --bins n3` passes — confirmed.

### Builtins added (all in `lib/src/queryengine.rs`'s `builtins` module unless noted)
`math:difference`, `math:product` (now n-ary, folds over the whole list — see below), `math:quotient`, `math:remainder`, `math:notLessThan`, `math:notGreaterThan`, `math:lessThan`, `math:equalTo`, `log:notEqualTo`, `list:append` (from an earlier pass on this ticket, still verified working), plus this pass's additions:
- `log:collectAllIn` and `log:notIncludes` — both dispatched from `lib/src/reasoner.rs` (`Reasoner::materialize`'s rule-body special-casing, alongside the pre-existing `log:implies` handling), not from the ordinary per-row `queryengine::builtins` module, because both need a genuine sub-query against the live `TripleIndex` rather than a one-shot check over already-accumulated bindings. Both call the same shared helper, `Reasoner::eval_embedded_formula_against_store`, to avoid duplicating that sub-query logic.
- `math:sum` and `math:product` were changed from a hardcoded 2-operand form to a proper n-ary fold over the whole list (per the actual N3 built-ins spec), needed to make the vendored `dog` case's 5-element list sum correctly; this is a strict superset of the old behavior (a 2-element list folds identically) and doesn't change either builtin's existing conformance test.
- The N3 grammar (`lib/src/parser/n3.pest`) also gained one change in support of `log:collectAllIn`: `ListItem` (an RDF list's member alternatives) now includes `Formula`, since `log:collectAllIn`'s subject is `(?Var {quoted graph} ?List)` — a list containing a quoted graph. Previously `ListItem` explicitly excluded `Formula`.

### Honest gaps / notes
- **5 vendored EYE cases run and pass**: `socrates`, `path` (from the prior pass), plus this pass's `dog` (`log:collectAllIn` + n-ary `math:sum` + `math:greaterThan`), `snaf` (`log:notIncludes`), and `deep-taxonomy` (truncated from EYE's 10000-level/~30000-triple scalability benchmark down to a 3-level `rdfs:subClassOf` chain, to exercise the same backward-chaining transitivity rule as a fast conformance check rather than a scale test).
- **8 more EYE cases are vendored but `status: "ignored"`** in `manifest.json`, each with a `blocked_reason` field and a matching header comment in its own `.n3` file, precisely explaining the gap rather than a generic "not supported":
  - `nixon-diamond` — needs **rule-priority ordering**: `Reasoner::materialize` evaluates every rule in a stratum against one shared per-iteration `TripleIndex` snapshot and only merges all newly-derived triples in a single batch after the whole iteration finishes, so a `log:notIncludes` guard in one rule can never observe a fact just asserted by a higher-priority rule of the same stratum within the same iteration.
  - `backward` (the `math:greaterThan`-via-`<=` example) and `peano` — need **goal-directed/backward-chained query evaluation** seeded by a concrete query goal; this engine is purely forward-chaining with no query/goal-injection mechanism.
  - `good-cobbler` and `peano` — need **general list-term pattern unification** (matching a rule pattern list containing a variable, e.g. `(:good ?Y)`, against a separately-parsed ground list, e.g. `(:good :Cobbler)`); this engine represents list terms as opaque synthetic blank-node ids (`triples.rs`'s `LIST_REGISTRY`) and only ever compares them for id equality in `TripleIndex::query`, with no structural/element-wise unification. The explicit list built-ins (`list:in`/`length`/`append`/`sum`/`concat`/`collectAllIn`) work fine since they dereference list members procedurally instead of relying on pattern matching.
  - `skos` — needs a bodyless **`=> false.` denial-rule head** (this engine's `Rule.head` is a plain `Triple`, with no boolean/denial variant), `;`-shared-subject predicate-object-list syntax, and the **`func:` RIF-builtin-function namespace** (`func:lang-from-PlainLiteral`) — none in scope for this ticket.
  - `filter` — needs **`string:lessThan`**, a `string:` builtin not yet implemented (only `string:length`/`string:concat` exist); otherwise its `log:collectAllIn` usage is exactly what this ticket's implementation supports.
  - `basic-monadic` — needs the same list-term unification gap as `good-cobbler`/`peano` (to detect a repeated variable closing an 11-element cycle-list pattern), and its fixture data is randomly-generated 10^n-scale benchmark data rather than a small illustrative fact set.
  - `edt` — exercises the exact same `rdfs:subClassOf` backward rule as `deep-taxonomy` (which does run, truncated), just at full unpractical scale (30002 triples, 10000 levels); skipped as a scale-only duplicate, not a new feature gap.
- **Still entirely out of scope, not vendored at all**: N3-star (RDF-star-in-N3), xsd date/duration arithmetic, the `func:` namespace beyond the one case noted above, Euler-sharp (`e:`) extensions, `log:impliesAnswer` (EYE's query/answer-extraction predicate — this test suite only ever calls `materialize()` and inspects derived facts directly, so there's no notion of a "query" to answer), and any backward/goal-directed query evaluation mode in general (this remains a pure forward-chaining fixpoint engine).
- `path`'s `expected` fixture was itself checked by the upstream EYE test against a narrow, non-generic query rather than full closure equality (see `manifest.json`'s `runner_semantics` field) — the roxi test asserts a subset/entailment check (every expected triple is present among derived facts), not exact equality with EYE's output.
