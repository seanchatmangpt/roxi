# SHACL FMEA Matrix — constraint type × failure-mode category

## Purpose

Failure Mode and Effects Analysis (FMEA), TPS/Six-Sigma discipline applied to this session's real breakage ledger (`011-breakage-plan-tps-dflss.md`): before generating combinatorial test cases, name the *specific* plausible defect for every (constraint type, failure-mode category) pairing. This makes the NK combinatorial study (`lib/tests/shacl_nk_study.rs`) deliberate coverage, not just volume — each generated case is chosen because it targets a named, real failure mode, calibrated against defects this session actually found (not hypothetical ones).

## Failure-mode categories

- **Boundary/off-by-one**: inclusive vs exclusive edge mishandled.
- **Type coercion**: a comparison silently succeeds/fails because of how a lexical form was parsed (the dateTime bug, item #5 in the breakage ledger, lived here).
- **Cardinality mismatch**: count-based logic (minCount/maxCount/qualifiedMinCount) miscounts or double-counts.
- **Severity/conforms leakage**: a constraint's violation result doesn't correctly propagate into the report's `conforms` boolean (breakage ledger item #4).
- **No-string-representation**: a blank node or other non-literal value silently satisfies a string-shaped facet (breakage ledger item #6).
- **Vacuous truth**: the constraint is silently never evaluated (empty list, missing wiring) so anything "passes" by omission rather than by genuinely satisfying the rule.
- **Interaction-order-dependence**: evaluating constraint A before B (or vice versa) changes the result — should never happen for a conjunction of independent constraints, but only combinatorial (K≥2) testing can catch it if it does.

## Matrix

| Constraint | Boundary/off-by-one | Type coercion | Cardinality mismatch | Severity leakage | No-string-rep | Vacuous truth | Order-dependence |
|---|---|---|---|---|---|---|---|
| `sh:minCount`/`maxCount` | count exactly at the bound wrongly rejected/accepted | N/A | **primary risk**: miscounts multi-valued property (breakage ledger #7 nested-property recursion touched this) | violation not flipping conforms | N/A | constraint silently skipped if `sh:path` unresolved | count computed before vs after a sibling constraint filters values |
| `sh:minInclusive`/`maxInclusive`/`minExclusive`/`maxExclusive` | inclusive/exclusive boundary swap | **primary risk**: dateTime/numeric coercion (breakage ledger #5) | N/A | violation not flipping conforms | comparing a non-comparable term silently "passes" instead of violating | comparator returns `None` silently treated as pass instead of fail | N/A |
| `sh:pattern`+`sh:flags` | regex anchoring off-by-one | lexical form extraction wrong for typed literals | N/A | violation not flipping conforms | blank node's internal label matched instead of a real string | empty pattern silently matches everything | flags applied before/after value resolution |
| `sh:datatype` | N/A | **primary risk**: lexical well-formedness not checked (breakage ledger #2) | N/A | violation not flipping conforms | N/A | datatype IRI comparison skipped if literal untyped | N/A |
| `sh:class` | N/A | N/A | N/A | violation not flipping conforms | blank node/literal wrongly treated as having an rdf:type | `rdfs:subClassOf` transitivity silently not walked | class-hierarchy walk order affecting result |
| `sh:hasValue`/`sh:in` | N/A | value-equality using wrong term-comparison (encoded id vs lexical) | N/A | violation not flipping conforms | N/A | empty `sh:in` list silently treated as "any value allowed" instead of "no value allowed" | N/A |
| `sh:nodeKind` | N/A | N/A | N/A | violation not flipping conforms | blank node classified as neither IRI nor Literal nor BlankNode (fallthrough bug) | unknown nodeKind IRI silently treated as always-true | N/A |
| `sh:languageIn` | N/A | untagged literal wrongly treated as matching | N/A | violation not flipping conforms | N/A | empty language list silently treated as "any language allowed" | N/A |
| `sh:uniqueLang` | **primary risk (fixed this session, breakage ledger #3)**: counting duplicate values instead of duplicate tags | tag case-sensitivity mishandled | count of violations vs count of offending tags conflated | violation not flipping conforms | N/A | N/A | tag-counting order affecting which "first" occurrence is exempted |
| `sh:equals`/`sh:disjoint`/`sh:lessThan`/`sh:lessThanOrEquals` | boundary for lessThanOrEquals vs lessThan | numeric vs lexical comparison mismatch | comparing against zero values of the other property (vacuous?) | violation not flipping conforms | N/A | comparison property with zero values silently treated as satisfying equals/disjoint | evaluating the comparison property's values before vs after they're derived by another rule |
| `sh:qualifiedValueShape`+`sh:qualifiedMinCount`/`sh:qualifiedMaxCount` | boundary at exactly qualifiedMinCount | N/A | **primary risk**: interacts with outer `sh:maxCount` (breakage ledger's own item #11 in the impossible-catalog) | violation not flipping conforms | N/A | N/A | qualifying-shape evaluation recursion depth/order |
| `sh:minLength`/`sh:maxLength` | boundary at exact length | length computed on wrong representation (encoded id length vs lexical length) | N/A | violation not flipping conforms | **primary risk (fixed this session, breakage ledger #6)**: blank node's internal label used as string | N/A | N/A |
| `sh:and`/`sh:or`/`sh:not`/`sh:xone` (logical combinators) | N/A | N/A | N/A | **primary risk (fixed this session, breakage ledger #1)**: sub-shape violations leaking into report instead of one combinator-level result | N/A | zero-length sub-shape list silently treated as trivially satisfied | sub-shape evaluation order affecting which violation is reported first |
| `sh:node` (nested/property recursion) | N/A | N/A | N/A | violation not flipping conforms | N/A | **primary risk (fixed this session, breakage ledger #7)**: nested property shape never evaluated at all | recursion depth limits silently truncating deep chains |

## How this feeds the NK study

Cells marked **primary risk** are where a real defect already lived this session — the combinatorial harness (`lib/tests/shacl_nk_study.rs`) deliberately generates parameter/data combinations that stress these specific cells *in combination with a second (or third) constraint*, since the single-constraint version of each primary risk is already closed and tested (breakage ledger + impossible-catalog). The open question this study answers: does combining two previously-fixed-in-isolation constraints reintroduce any of these failure modes through their *interaction* (e.g., does `sh:minLength` + `sh:class` on the same property still correctly treat a blank node as a no-string-rep AND non-class violation simultaneously, or does one code path short-circuit the other)?
