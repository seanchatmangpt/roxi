# Handoff Report: TICKET-001 — RDF Term Model Expansion

## 1. Observation

In implementing TICKET-001 in `roxi`, the following files and structural configurations were directly observed and modified:

* **`lib/src/triples.rs`**:
  * Original definition of `VarOrTerm` and `TermImpl`:
    ```rust
    pub enum VarOrTerm {
        Var(Variable),
        Term(TermImpl),
    }
    pub struct TermImpl {
        pub(crate) iri: usize,
    }
    ```
  * Modified to define `Term` as an enum of sibling variants containing `TermImpl` (IRI), `LiteralImpl` (lexical value, datatype, and lang), and `BlankNodeImpl` (id):
    ```rust
    pub enum Term {
        Iri(TermImpl),
        Literal(LiteralImpl),
        BlankNode(BlankNodeImpl),
    }
    ```
  * `VarOrTerm::Term` was updated to hold `Term` instead of `TermImpl`.
  * Added tests `test_literal_term_roundtrip` and `test_blank_node_term_encoding`.

* **`lib/src/encoding.rs`**:
  * Implemented `EncodedValue` to tag each entry with its kind:
    ```rust
    pub enum EncodedValue {
        Iri(String),
        LiteralLexical(String),
        BlankNodeLabel(String),
        Literal {
            value: usize,
            datatype: Option<usize>,
            lang: Option<usize>,
        },
        Variable(String),
    }
    ```
  * Updated `InternalEncoder` mapping from `HashMap<String, usize>` / `HashMap<usize, String>` to `HashMap<EncodedValue, usize>` / `HashMap<usize, EncodedValue>`.
  * Added parsing inside `add` and `get` to support transparent auto-detection of variables (starts with `?`), blank nodes (starts with `_:`), literals (starts with `"`), and IRIs (fallback).
  * Exposed `add_iri`, `add_blank_node`, `add_literal`, and `decode_to_term`.
  * Added tests `test_encoder_literal_vs_iri_distinct` and `test_literal_datatype_and_langtag_preserved`.

* **`lib/src/tripleindex.rs`**:
  * Updated index maps `spo`, `pos`, and `osp` to hold `Option<Term>` instead of `Option<TermImpl>`.
  * Updated graph name accesses to retrieve the ID via `.id()` instead of `.iri`.

* **`lib/src/reasoner.rs`**:
  * Updated substitution and matching logic to use `Term` and `.id()`, replacing raw accesses to `.iri`.

* **`lib/src/dred.rs`**:
  * Line 118: changed `right_name.iri` to `right_name.id()`.

* **`lib/src/sparql.rs`**:
  * Changed `PlanExpression::Constant` to wrap `Term`.
  * Modified `extract_expression` for `Expression::Literal` to parse lexical value, datatype, and language tag, adding them to the encoder to construct a `Term::Literal`.
  * Modified `eval_expression` for `PlanExpression::Constant` to parse the integer value of `xsd:integer` literals for comparisons.

* **Tests output**:
  * Command: `cargo test --lib --bins -- --skip test_cyclic_rules_terminate --skip test_csprite_cycles_terminate`
  * Result: `test result: ok. 72 passed; 0 failed; 9 ignored; 2 filtered out; finished in 1.01s`

## 2. Logic Chain

1. **Need for RDF term representation**: The original `roxi` query engine lacked representation for RDF literals and blank nodes, which limits SPARQL 1.1, SHACL, and N3 compliance (Observation 1).
2. **Implementation path**: A sibling-enum design `Term` (`Iri`, `Literal`, `BlankNode`) was added to `lib/src/triples.rs`. To allow terms to be represented in the index tables via single `usize` identifiers, `VarOrTerm::to_encoded` delegates to `Term::id()`, which returns the interned ID of the term (Observation 1, 2).
3. **Tagged global encoding**: The global `Encoder` was restructured using `EncodedValue` to distinguish IRIs, Literal components, and Blank Nodes, preventing structural collision (e.g. between a literal `"http://example.com"` and IRI `http://example.com`) and preserving the literal lexical value, datatype, and language tag (Observation 2).
4. **Codebase-wide updates**: Match and field access sites throughout the codebase (in `tripleindex.rs`, `reasoner.rs`, `dred.rs`, and `sparql.rs`) were updated to use `.id()` and matching on `Term` instead of `TermImpl`. All other functions remain unchanged to respect the minimal change principle (Observation 3, 4, 5, 6).
5. **Expression evaluation integration**: In `sparql.rs`, literals are parsed into the tagged encoder, and `eval_expression` retrieves the lexical integer value when comparing `xsd:integer` literal types, ensuring full compatibility with existing SPARQL numeric filter tests (Observation 6).
6. **Correctness verification**: Running cargo test on all library modules succeeds with no errors or regressions on the 68 baseline tests plus the 4 new verification tests (Observation 7).

## 3. Caveats

* **Unimplemented Cycle Guard tests**: The tests `test_cyclic_rules_terminate` and `test_csprite_cycles_terminate` were skipped. They belong to TICKET-002 and TICKET-003, which are not implemented yet and are designed to fail/panic until those tickets are worked on.
* **SPARQL datatype support**: Only `xsd:integer` is explicitly parsed as a numeric literal inside `eval_expression` for comparison, maintaining the behavior of the original engine. String and boolean comparisons use standard formats.

## 4. Conclusion

TICKET-001 has been fully and successfully implemented. The term model now distinguishes between IRIs, Literals, and Blank Nodes via the `Term` enum, back-end structures, and the kind-tagged `Encoder` table. All code paths compile clean, and the test suite passes successfully.

## 5. Verification Method

1. Run the workspace checks to verify that compilation succeeds:
   ```bash
   cargo check --workspace
   ```
2. Run the test suite excluding unimplemented cycle guard tests to verify that all 72 tests pass:
   ```bash
   cargo test --lib --bins -- --skip test_cyclic_rules_terminate --skip test_csprite_cycles_terminate
   ```
3. Inspect `lib/src/triples.rs` for `test_literal_term_roundtrip` and `test_blank_node_term_encoding`.
4. Inspect `lib/src/encoding.rs` for `test_encoder_literal_vs_iri_distinct` and `test_literal_datatype_and_langtag_preserved`.
