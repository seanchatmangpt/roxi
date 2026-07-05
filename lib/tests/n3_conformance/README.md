# N3 Conformance Tests

This directory contains manifest-driven N3 conformance test cases for this
crate's N3 rule engine (parser + forward-chaining reasoner + built-in
predicates).

There is no official W3C Recommendation test suite for N3, so this suite is a
mix of:

- **`vendored/`** -- cases lifted (with light adaptation, documented in each
  file's header comment) from the [EYE reasoner](https://github.com/eyereasoner/eye)'s
  own public `reasoning/` test corpus. Most of that corpus exercises EYE
  features well beyond this crate's scope (`log:collectAllIn`,
  `log:notIncludes` SNAF guards, xsd date/duration arithmetic, N3-star,
  dependent types, ...); `socrates` and `path` were the simplest cases that
  run against this engine's supported subset.
- **`syntax/`** and **`builtins/`** -- hand-authored cases covering this
  grammar's syntax surface (multi-triple heads, literals, blank nodes, RDF
  lists, `@forAll`/`@forSome`, quoted graphs, backward implication `<=`) and
  its N3 built-in predicates (`log:equalTo`, `log:implies`, `math:sum`,
  `math:greaterThan`, `list:in`, `list:length`, `string:concat`,
  `string:length`).

## Folder Structure

```text
lib/tests/n3_conformance/
├── README.md               # This file
├── manifest.json           # JSON index of all conformance test cases
├── main.rs                 # The manifest-driven test runner
├── syntax/                 # N3 syntax and structure test cases
│   ├── multi_head.n3 / multi_head_result.n3
│   ├── literals.n3 / literals_result.n3
│   └── ...
├── builtins/               # Built-ins (log:, math:, list:, string:) test cases
│   ├── log_equal_to.n3 / log_equal_to_result.n3
│   ├── math_sum.n3 / math_sum_result.n3
│   └── ...
└── vendored/                # Adapted EYE reasoner test cases
    ├── socrates.n3 / socrates_result.n3
    └── path.n3 / path_result.n3
```

Each case is an `(input, expected)` pair of `.n3` files: `input` is a
complete N3 document (facts + rules), and `expected` is a plain N3 fact list
naming the triples that must appear among the derived (materialized) facts.

## Running Conformance Tests

`main.rs`'s `test_n3_conformance` test:
1. Parses `manifest.json`.
2. For each test case with status `active`:
   - Loads the `input` N3 file via `TripleStore::from` (facts + rules).
   - Runs `store.materialize()` to compute the forward-chaining fixpoint.
   - Decodes every inferred triple to its `"S P O."` string form.
   - Asserts every triple listed in `expected` is present among them.

This is a **subset/entailment check**, not exact-equality: a forward-chaining
fixpoint often derives more than any one hand-picked expectation captures,
and two of the vendored EYE cases (`path`) were themselves checked by EYE
against only a narrow, non-generic query rather than the full closure -- see
`manifest.json`'s `runner_semantics` field and each case's own header
comments for the reasoning behind its specific `expected` file.

Running just this suite:

```sh
cargo test -p roxi --test n3_conformance
```

Regenerating the pass-rate manifest at
`docs/jira/26.7.4/manifests/n3_manifest.md` happens as a side effect of
running the test (see `main.rs`).
