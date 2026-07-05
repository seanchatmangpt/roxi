# roxi v26.7.4: One engine, five fully-conformant dialects

*Working-backwards press release — internal, dated 2026-07-04*

## Headline

**roxi becomes the first embeddable, wasm-capable RDF engine to natively and fully implement SPARQL, SHACL, ShEx, N3, and Datalog reasoning against a single shared graph — with 100% specification-validated conformance across all five, enforced continuously by CI.**

## Summary

Today, teams that need query (SPARQL), validation (SHACL and/or ShEx), and reasoning (N3/Datalog) over RDF data run separate tools glued together with lossy translation layers, or they pick one paradigm and live without the others. Heavyweight server-based stores (Jena, GraphDB, Stardog) target the "big server you query" model; even oxigraph, the project roxi is closest to in spirit, does not attempt validation or rule-based reasoning at all.

With v26.7.4, roxi closes that gap completely for its own scope: SPARQL 1.1, SHACL, ShEx, N3, and Datalog all operate over the same in-memory `Triple`/`TripleIndex` representation, each measured against a real specification test suite (the W3C SPARQL 1.1 suite, the W3C `data-shapes` SHACL suite, the `shexTest` suite, and the EYE/N3 community corpus as the closest thing N3 has to one — plus a comprehensive hand-authored suite for Datalog, which has no single official spec to defer to), and a CI gate that fails the build the moment any of the five regresses.

## Why this matters

Because all five dialects share one substrate instead of five, capabilities become possible that don't really exist as shipped products today:

- **Validation-as-reasoning, interleaved.** Instead of validate → reason → validate again as three disconnected passes, N3/Datalog rules can derive new facts while SHACL/ShEx shapes constrain what's derivable — catching an inference engine going somewhere invalid mid-materialization, not after the fact.
- **Shape-directed static analysis.** Because SHACL/ShEx describe expected shapes and N3/Datalog describe derivation rules against the same term model, it becomes possible to ask "given this rule set, can it ever produce data that violates this shape?" as a compile-time question, not something you only find out empirically after running the reasoner.
- **Incremental, streaming shape validation.** roxi already has RSP (stream processing) and DRed (incremental delete/rederive). With SHACL/ShEx native rather than bolted on, shape validation can re-check only the delta as facts are asserted or retracted in real time — instead of re-running a full validator over a static snapshot, which is what every existing SHACL/ShEx engine assumes.
- **Frictionless SHACL ↔ ShEx interop.** Enterprise data (SHACL) and bioinformatics/library-science data (ShEx) can be reasoned over with the same rule engine without a lossy conversion step, because both validators run against the same graph and the same term model.
- **Embedded, not server-based.** Because it's one coherent Rust crate (wasm-target-capable, no separate services to run), roxi can be embedded directly into mobile apps, desktop tools, and CI pipelines doing schema/rule validation on every commit — a shape of deployment that server-based triple stores structurally cannot match.

## What "done" looks like

Not a demo. Not "mostly works." Every one of the five dialects passes its respective conformance suite in full, continuously, gated by CI:

| Dialect | Conformance bar | Suite location |
|---|---|---|
| SPARQL 1.1 | W3C `sparql11-test-suite` | `lib/tests/sparql11_conformance/` |
| SHACL | W3C `data-shapes` test suite | `lib/tests/shacl_conformance/` |
| ShEx | `shexTest` (shex.io/shex-test) | `lib/tests/shex_conformance/` |
| N3 | EYE/N3 community test corpus | `lib/tests/n3_conformance/` |
| Datalog | Comprehensive hand-authored suite (safety, stratification, recursion, aggregation) | `lib/tests/datalog_conformance/` |

Aggregated status lives in `docs/jira/26.7.4/CONFORMANCE-STATUS.md`, kept current by the CI gate defined in ticket 010.

## The ten tickets that get us there

001 (term model) → {002 → 003} cycle safety, plus 006 (SPARQL) and 007 (oxrdf adapter) in parallel → 004 (Datalog) → 005 (N3, coordinated with 004) → 008/009 (SHACL, ShEx) → 010 (CI gate makes it permanent). Full detail, file-level implementation notes, and Definitions of Done for each are in `docs/jira/26.7.4/00N-*.md`.

## A note on framing

The capabilities described above — interleaved validation-and-reasoning, shape-directed static analysis, streaming incremental validation, embedded deployment — are the motivating "why." They are not a checklist for this release; the checklist is the conformance-suite table above. This release earns the right to make those larger claims credibly precisely because it does the unglamorous work first: get the term model right, get every dialect to 100% against a real spec test suite, and enforce it in CI so it stays true.
