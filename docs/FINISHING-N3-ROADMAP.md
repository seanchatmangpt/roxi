# Finishing N3: current state, what's closed, what's left

This is the honest status of `roxi`'s N3 (Notation3) implementation as of this
writing, and a roadmap for the remaining work. It exists so nobody has to
re-derive "do we support N3?" from scratch by re-reading the whole codebase —
and so nobody rounds "we support most of it" up to "100%," which this
implementation explicitly is not.

## How to read this document

Every claim below is backed by a real, currently-passing test — either in
`lib/tests/n3_conformance/` (manifest-driven, see `manifest.json` and
`docs/jira/26.7.4/manifests/n3_manifest.md` for the generated pass-rate
table) or in a dedicated `lib/tests/n3_*.rs` file. Every gap below is backed
by a real, verified failure documented in `lib/tests/n3_conformance/
SKIPPED.md` — each entry there was confirmed by actually attempting to parse
or materialize the real upstream EYE source, not assumed. If a claim in this
document and a test disagree, trust the test and fix this document.

## What's solid

- **Core Turtle-N3 syntax**: prefixes, `@base`, SPARQL-style `PREFIX`/`BASE`,
  all four string-literal forms with real escape-sequence decoding, datatypes,
  language tags, RDF collections (including nested), `;`/`,` predicate/object
  sugar, `[ ... ]` anonymous blank-node property lists, `=`/`has`/`is ... of`
  sugar, path syntax (`!`/`^`), and now a bare literal as a triple's *subject*
  (not just object) — see `lib/tests/n3_literal_as_subject.rs`.
- **Forward chaining**: stratified fixpoint materialization, aggregates,
  `log:implies` dynamic rule reification (including chained/cross-rule
  reification and multiple independent `log:implies` literals in one rule
  body), `log:collectAllIn`, `log:notIncludes` (SNAF), denial rules
  (`=> false.`).
- **Backward chaining**: full SLD-style `solve()` (variable-goal, returns
  every binding row) plus the `prove()` ground-goal convenience wrapper,
  with list-pattern structural unification (Peano-style nested lists) and a
  recursion depth guard. **A real, severe bug here was found and fixed** —
  see "Recently fixed" below — so treat any code written against `prove`/
  `solve` before this fix with suspicion until re-tested.
- **Quantifier scoping**: `@forAll`/`@forSome` get real per-formula scoping,
  including inside a rule's own antecedent braces (a real grammar gap closed
  in the verification pass — see below). Deliberately narrower than the full
  N3 CG spec for *unquantified* (bare) variables — see "Permanent, documented
  limitations" below; this is a design decision, not an oversight.
- **Builtins**: broad `math:`/`string:`/`list:`/`log:` coverage plus `time:`,
  `crypto:`, and a `func:` (RIF) sampling — 100 conformance cases active (1
  deliberately ignored: `log:semantics`, see below), plus 31 adversarial
  edge-case tests in `lib/tests/n3_builtin_adversarial.rs` (division by zero,
  unicode, empty/nested lists, type mismatches) for the builtins that have
  received that depth of scrutiny (see the coverage caveat below).
- **19 real, byte-fetched EYE `reasoning/` corpus cases vendored and
  passing**, not hand-written approximations: `socrates`, `path`, `dog`,
  `snaf`, `deep_taxonomy`, `nixon_diamond`, `backward_math`, `good_cobbler`,
  `peano` (+ a variable-goal `solve()` variant), `skos`, `filter`,
  `basic_monadic`, `edt`, `entail`, `defeasable_logic`, `backward_more`,
  `restpath`, `graph`.
- **Unsupported-builtin predicates now fail loudly, not silently.** Any
  predicate from a recognized N3/EYE builtin namespace (`log:`/`math:`/
  `string:`/`list:`/`time:`/`crypto:`/`func:`/`e:`) that isn't registered —
  `log:callWithCut`, `log:repeat`, `e:findall`, etc. — now panics with a
  clear message instead of the rule silently never firing. Ordinary
  user-defined predicates with no matching facts are completely unaffected
  (that's normal Datalog behavior, not an error). See
  `builtins::reject_if_unsupported_builtin` and
  `lib/tests/n3_unsupported_builtin_guard.rs`.

## Recently fixed (worth knowing about even though it's closed)

An independent, fresh-context adversarial review pass (no prior knowledge of
this implementation) found two real bugs, both now fixed with regression
tests:

1. **A grammar gap**: `@forAll`/`@forSome` could not be declared inside a
   rule's own antecedent braces, only at the document root or inside a
   formula-as-term — a hard parse error, not a semantic limitation. Fixed in
   `n3.pest`/`n3rule_parser.rs`.
2. **A severe correctness bug in backward chaining**: `TripleStore::prove`/
   `solve` could report a goal as *proven* even when its underlying ground
   builtin check was actually false (e.g. `prove(3 :moreInterestingThan 5)`
   — false, since 3 is not greater than 5 — incorrectly returned `true`).
   Root cause: `Binding::len() == 0` is ambiguous between "zero rows"
   (failure) and "one row, zero columns" (a ground success), and
   `BackwardChainer::solve_inner` conflated the two. Fixed by changing
   `solve_inner` to return `Option<Binding>`, tracking success/failure
   explicitly. See `backwardchaining_test.rs`'s
   `test_prove_rejects_ground_goal_whose_body_constraint_is_actually_false`.
   **This was found by testing, not by reading the code** — the function's
   own doc comment already described the *correct* intended behavior; the
   implementation just didn't match it. Independent, executed verification
   matters more than code review here.

Also closed in a follow-up pass: `math:notEqualTo`, `list:notMember`,
`list:firstRest` builtins (each closes a specific catalogued EYE-corpus
gap — `kaprekar`/`goldbach`/`gcd-bezout-identity` and `path-discovery`
respectively), and the literal-as-subject grammar gap mentioned above.

## Permanent, documented limitations (not bugs, not TODOs)

- **`log:semantics` is deliberately unimplemented.** It requires network I/O
  to fetch a remote N3 document by IRI; this engine does no network access.
  One conformance case stays `ignored` for this, on purpose.
- **Bare-variable auto-scoping is intentionally not spec-accurate.** Only
  variables an author explicitly declares with `@forAll`/`@forSome` get real
  per-formula scoping. Unquantified variables keep this engine's flat,
  name-based identity across formula boundaries, because `log:implies`'s
  dynamic rule reification is structurally built on that name-matching. This
  was a deliberate tradeoff discovered while implementing scoping, documented
  in `n3rule_parser.rs`'s module doc comment — auto-scoping every bare
  variable would break `log:implies` chaining. If you need fully spec-literal
  scoping semantics, this engine does not provide them.
- **Adversarial testing depth is uneven across builtin modules.** `math.rs`
  builtins received real execution-verified edge-case review (division by
  zero, boundary values, malformed input). `string.rs`, `list.rs`, `func.rs`,
  `crypto.rs`, `time.rs`, and the backward-chaining engine's non-`math:`
  paths have **not** received the same depth — they pass their one-fixture
  conformance case each, which is a thinner evidence bar. Don't cite them as
  "adversarially verified."

## What's not supported at all

Every directory below was checked against the **real** fetched upstream `.n3`
source (never assumed from a directory name) — see
`lib/tests/n3_conformance/SKIPPED.md` for the full, line-by-line reasoning.
Categories of gap, roughly by how much work each would take to close:

**Grammar gaps** (would require real `n3.pest`/`n3rule_parser.rs` work):
- A rule (`=>`/`<=`) nested *inside* another rule's consequent (`bnode-scope`).
- A bare numeric literal as a member *inside* a `List` production, not just
  as a bare Subject (`kaprekar`, distinct from the now-fixed
  literal-as-subject gap).
- `($ $)` EYE "anything"-wildcard list pattern; a quoted rule as a rule's
  consequent (`n3plus1`).
- RDF-star / N3-star / TriG-style blank-node-labeled named-graph blocks
  (`n3-star`, `rdf-star`, `rdf-star-reasoning`, parts of `n3c`/`n3gl`) — not
  attempted at all; this is a genuinely different term model (quoted triples
  as first-class terms), not a small grammar patch.
- `=>` used as an ordinary predicate rather than a rule connective, in some
  EYE idioms.

**Missing builtins** (each individually small, but real feature work,
not just registry entries — cut semantics and `findall`-style aggregation
are control-flow/aggregation features, not simple predicates):
- `log:callWithCut` — Prolog-style cut, changes rule evaluation control flow.
- `log:repeat` — iteration/repetition semantics, unclear scope without a
  concrete upstream spec to pin down.
- `e:findall` (the `eulersharp.../log-rules#` namespace's aggregate, distinct
  from `log:collectAllIn`/`log:forAllIn`) — needs a formula-valued generator
  argument and exclusion-list semantics.

**A real reasoning-engine limitation, not a parser/builtin gap**:
- Existential-variable skolemization interacting with formula-pattern-vs-
  asserted-content unification in at least one corpus case — documented in
  `SKIPPED.md`, not yet root-caused to a specific fix.

**A disclosed-but-deferred vendoring candidate**: `path-discovery`'s *only*
documented blocker (the `list:notMember`/`list:firstRest` builtins) is now
closed — see "Recently fixed" above — so this directory is a real candidate
for vendoring, just not yet done (its 4.8 MB real-world payload would also
need truncation, same as `deep_taxonomy`/`basic_monadic`/`edt`).

## What "day to day N3" coverage actually means here

If you hand this engine an ordinary hand-written N3 ruleset — the kind that
uses forward/backward rules, common `math:`/`string:`/`list:` builtins,
negation, `log:implies`, lists, quoted formulas — it will very likely work.
Roughly 80-90% of common real-world N3 usage, by the reasoning above. The
remaining 10-15% clusters specifically around: RDF-star/N3-star syntax, a
handful of named builtins (`callWithCut`, `repeat`, `findall`), literal
members inside lists (as opposed to literal subjects, now fixed), and rule-
in-rule nesting. None of that is silently wrong anymore — it either doesn't
parse (a real `Result::Err`) or panics with a clear "unsupported builtin"
message. Nothing in the unsupported 10-15% produces a plausible-looking wrong
answer.

## Roadmap: recommended order for closing the rest

1. **Vendor `path-discovery`** (cheap now that its only builtin blocker is
   closed) — truncate the 4.8 MB payload the same way `deep_taxonomy` was.
2. **Adversarially test the unreached builtin modules** (`string.rs`,
   `list.rs`, `func.rs`, `crypto.rs`, `time.rs`) and the backward-chaining
   engine's non-`math:` paths, to the same depth `math.rs` already got —
   this is the single highest-leverage trust-building move left, since it's
   the explicitly-flagged thin spot in current evidence.
3. **`log:callWithCut`** — closes `ackermann`, `goldbach`'s cut guard, the
   `nk`-style corpus cases; requires an actual control-flow change to rule
   body evaluation (short-circuit remaining alternatives once cut fires),
   not just a new procedural builtin. Scope this as its own investigation
   before committing to an implementation approach.
4. **Literal members inside `List`** (not just as a bare Subject) — closes
   `kaprekar` and similar; a grammar change plus checking `parse_list`
   handles the resulting `Literal` pairs (it likely already does, via the
   same `term_from_pair` dispatch the Subject fix relied on — verify before
   assuming new logic is needed).
5. **`e:findall`** — a genuinely new aggregation feature (formula-valued
   generator, exclusion-list semantics); scope this against the real
   `medic.n3` upstream source before implementing, since its exact semantics
   need to be pinned down from a concrete example, not guessed from the name.
6. **RDF-star/N3-star support** — the largest single item, a different term
   model (triples as first-class terms in subject/object position), not
   attempted at all yet. Treat as its own multi-phase plan, not a quick add.
7. **`log:repeat`** and any other builtin surfaced by (2)'s adversarial
   sweep, on a case-by-case basis once a concrete failing example exists.

Each item above should be closed the same way everything in this document
was: fetch the real upstream source (verbatim, never AI-paraphrased), write
a failing test against it first, fix the root cause, and only then update
`manifest.json`/`SKIPPED.md` — never mark something closed on the strength of
a claim alone.
