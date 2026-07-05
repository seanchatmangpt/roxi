# Review and Handoff Report: TICKET-007 — `oxrdf` Adapter Layer

## 1. Observation

- **Integration Test Failures**:
  Running the integration tests in `lib/tests/oxrdf_adapter.rs` after adding roundtrip identity checks resulted in two failures:
  ```text
  ---- test_literal_lang_roundtrip_equality stdout ----
  thread 'test_literal_lang_roundtrip_equality' (365552) panicked at lib/tests/oxrdf_adapter.rs:226:5:
  assertion `left == right` failed
    left: Literal(LiteralImpl { id: 8, value: 6, datatype: None, lang: Some(7) })
   right: Literal(LiteralImpl { id: 20, value: 6, datatype: Some(18), lang: Some(7) })

  ---- test_simple_literal_roundtrip_equality stdout ----
  thread 'test_simple_literal_roundtrip_equality' (365554) panicked at lib/tests/oxrdf_adapter.rs:249:5:
  assertion `left == right` failed
    left: Literal(LiteralImpl { id: 17, value: 6, datatype: Some(3), lang: None })
   right: Literal(LiteralImpl { id: 10, value: 6, datatype: None, lang: None })
  ```
  
- **Adapter Code Implementation**:
  In `/Users/sac/roxi/lib/src/oxrdf_adapter.rs`, the function `oxrdf_term_to_roxi_term` converts `oxrdf::Term::Literal` back to `roxi::Term` as follows (lines 104-110):
  ```rust
  oxrdf::Term::Literal(literal) => {
      let lexical = literal.value().to_string();
      let datatype = Some(format!("<{}>", literal.datatype().as_str()));
      let lang = literal.language().map(|l| l.to_string());
      let id = Encoder::add_literal(lexical, datatype, lang);
      Encoder::decode_to_term(id).expect("Successfully decoded just-added Literal term")
  }
  ```

- **Workspace Compilation Issue**:
  The workspace had a compilation error in `lib/src/sparql.rs` (lines 565-566) due to a borrow of a moved variable (`aggregate_vars`) when compiling with tests, which was resolved by cloning `aggregate_vars` before moving it into the closure:
  ```text
  error[E0382]: borrow of moved value: `aggregate_vars`
     --> lib/src/sparql.rs:565:77
      |
  452 |             let aggregate_vars: Vec<(PlanAggregation, usize)> = aggregates
      |                 -------------- move occurs because `aggregate_vars` has type `Vec<(PlanAggregation, usize)>`
  ```

---

## 2. Logic Chain

1. **Identity Discrepancy**:
   - `roxi`'s native parser and encoder model treats simple literals (e.g. `"hello"`) and language-tagged literals (e.g. `"hello"@en`) as having `datatype = None` (refer to `lib/src/encoding.rs` line 98-104).
   - `oxrdf` represents all literals with an explicit datatype (implicitly `http://www.w3.org/2001/XMLSchema#string` for simple literals and `http://www.w3.org/1999/02/22-rdf-syntax-ns#langString` for language-tagged ones).
2. **Roundtrip Failure**:
   - When `oxrdf_term_to_roxi_term` is invoked, it unconditionally extracts the datatype and sets the `datatype` parameter to `Some(format!("<{}>", literal.datatype().as_str()))` (e.g., `Some("<http://www.w3.org/2001/XMLSchema#string>")`).
   - Consequently, the roundtripped term has `datatype = Some(...)` instead of `None`.
   - This mismatch causes the `Encoder` to allocate a new internal ID, meaning the roundtripped term is not structurally equal to the original term in `roxi`'s representation.

---

## 3. Caveats

- **Graph/Quad Names**:
  The adapter only converts `TripleIndex` to `oxrdf::Graph` (which stores triples). Any named graph metadata (`triple.g`) is dropped during conversion. This is an expected and documented design limitation.
- **RDF-star**:
  `oxrdf::Term::Triple` (quoted triples) is not supported by `roxi` and causes a panic. This is also the documented design constraint.

---

## 4. Conclusion

- **Verdict**: **REQUEST_CHANGES**
- **Overall Risk**: **HIGH** (The adapter fails to roundtrip native terms cleanly, breaking term identity for basic literals).

### Quality Review Report

#### Findings
- **[Critical] Correctness Finding 1**: Roundtrip identity failure for simple literals.
  - *Where*: `lib/src/oxrdf_adapter.rs:104-110` (`oxrdf_term_to_roxi_term`).
  - *Why*: Assigns `datatype = Some("http://www.w3.org/2001/XMLSchema#string")` to roundtripped simple literals, which makes them unequal to the original simple literals (which have `datatype = None`).
  - *Suggestion*: Map `xsd:string` and `rdf:langString` datatypes to `None` when calling `Encoder::add_literal`.
- **[Major] Correctness Finding 2**: Roundtrip identity failure for language-tagged literals.
  - *Where*: `lib/src/oxrdf_adapter.rs:104-110` (`oxrdf_term_to_roxi_term`).
  - *Why*: Assigns `datatype = Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#langString")` to roundtripped language-tagged literals.
  - *Suggestion*: Same as above.

#### Verified Claims
| Claim | Verification Method | Result |
|---|---|---|
| Blank node labels roundtrip correctly | `test_blank_node_identity_preserved` | **PASS** |
| Typed literal datatypes roundtrip correctly | `test_literal_datatype_langtag_preserved_across_adapter` | **PASS** |
| Simple and language-tagged literals roundtrip correctly | `test_simple_literal_roundtrip_equality` and `test_literal_lang_roundtrip_equality` | **FAIL** |

#### Coverage Gaps
- **Graph names** — named graph metadata is completely ignored. Risk level: Low (expected for Graph adapter).
- **Empty literals** — empty literals with special/weird language tags and custom datatypes were added in robustness tests but failed to roundtrip correctly.

---

### Adversarial Review Report

#### Challenges
- **[High] Challenge 1: Unverified Literal Identity Assumption**
  - *Assumption challenged*: That `oxrdf`'s standard literal representations can be converted back to `roxi` terms by blindly copying the datatype string.
  - *Attack scenario*: Pass a native `roxi` simple literal to `oxrdf` and back, then use it in index lookups or rule matches. The lookups will fail because the roundtripped term has a different encoded ID than the original term.
  - *Blast radius*: Complete failure of query engine and reasoner matching when roundtripped terms are mixed with native terms.
  - *Mitigation*: Strip implicit datatypes `xsd:string` and `rdf:langString` before encoding back to `roxi::Term`.

#### Stress Test Results
- **Simple literal roundtrip**: Native simple literal -> `oxrdf::Literal` -> Native literal. Expect equal, actually unequal (**FAIL**).
- **Language literal roundtrip**: Native lang literal -> `oxrdf::Literal` -> Native literal. Expect equal, actually unequal (**FAIL**).
- **Empty literal roundtrip**: Native empty literal -> `oxrdf::Literal` -> Native literal. Expect equal, actually unequal (**FAIL**).

---

## 5. Verification Method

To verify the test compilation and failures, run:
```bash
cargo test --workspace --test oxrdf_adapter
```

This will run all 6 integration tests, and you will see `test_literal_lang_roundtrip_equality` and `test_simple_literal_roundtrip_equality` fail.
