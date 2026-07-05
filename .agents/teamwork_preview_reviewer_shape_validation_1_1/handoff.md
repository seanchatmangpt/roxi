# Review Report: TICKET-007 — `oxrdf` Adapter Layer Review

## Review Summary

**Verdict**: REQUEST_CHANGES

## 1. Observation

- **Implementation Location**:
  - Adapter layer: `/Users/sac/roxi/lib/src/oxrdf_adapter.rs`
  - VarOrTerm additions: `/Users/sac/roxi/lib/src/triples.rs`
  - Integration tests: `/Users/sac/roxi/lib/tests/oxrdf_adapter.rs`

- **Verbatim Error (Pre-existing Compilation Failure)**:
  - Running a full compilation via `cargo test --workspace` failed with:
    ```text
    error[E0026]: variant `PlanNode::Extend` does not have a field named `from`
       --> lib/src/sparql.rs:484:35
        |
    484 |         PlanNode::Extend { child, from, to } => {
        |                                   ^^^^
        |                                   |
        |                                   variant `PlanNode::Extend` does not have this field
        |                                   help: `PlanNode::Extend` has a field named `expression`

    error[E0027]: pattern does not mention field `expression`
       --> lib/src/sparql.rs:484:9
        |
    484 |         PlanNode::Extend { child, from, to } => {
        |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing field `expression`
    ```

- **Targeted Test Execution**:
  - Running `cargo test -p roxi --test oxrdf_adapter` (which bypasses compiling `lib/src/sparql.rs` using cached artifacts) compiles successfully and passes with 3 tests:
    ```text
    running 3 tests
    test test_blank_node_identity_preserved ... ok
    test test_triple_index_to_oxrdf_graph_roundtrip ... ok
    test test_literal_datatype_langtag_preserved_across_adapter ... ok

    test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    ```

- **Literal Representation Code**:
  - In `lib/src/oxrdf_adapter.rs`:
    ```rust
    oxrdf::Term::Literal(literal) => {
        let lexical = literal.value().to_string();
        let datatype = Some(format!("<{}>", literal.datatype().as_str()));
        let lang = literal.language().map(|l| l.to_string());
        let id = Encoder::add_literal(lexical, datatype, lang);
    ```
  - In `lib/src/encoding.rs` for `InternalEncoder::add` (which parses native literal strings):
    ```rust
    if suffix.starts_with('@') {
        lang = Some(suffix[1..].to_string());
    } else if suffix.starts_with("^^") {
        datatype = Some(suffix[2..].to_string());
    }
    self.add_literal(lexical, datatype, lang)
    ```

---

## 2. Logic Chain

1. **Native vs. Roundtripped Literals**:
   - A natively parsed language-tagged literal (e.g. `"hello"@en`) is processed via `InternalEncoder::add` which sets `datatype = None` and `lang = Some("en")`.
   - When converted from an `oxrdf::Term::Literal` containing `"hello"@en`, the `literal.datatype()` is automatically `http://www.w3.org/1999/02/22-rdf-syntax-ns#langString` in `oxrdf`.
   - `oxrdf_term_to_roxi_term` extracts `datatype` as `Some("<http://www.w3.org/1999/02/22-rdf-syntax-ns#langString>")` and `lang = Some("en")`.
   - Because `datatype` is `Some(...)` instead of `None`, the interned `EncodedValue::Literal` contains different fields.
   - Thus, the native literal and the roundtripped literal get assigned **different IDs** in the `InternalEncoder`, breaking term equality and matching/unification.

2. **Simple Literals**:
   - Similarly, a native simple literal `"hello"` has `datatype = None` and `lang = None`.
   - When roundtripped through `oxrdf`, `literal.datatype()` returns `http://www.w3.org/2001/XMLSchema#string`.
   - `oxrdf_term_to_roxi_term` registers it with `datatype = Some("<http://www.w3.org/2001/XMLSchema#string>")`, which assigns it a different interned ID, breaking equality.

---

## 3. Findings

### [Critical] Finding 1: Mismatch in roundtripping simple and language-tagged literals

- **What**: Converted simple/language-tagged literals from `oxrdf` back to `roxi` gain explicit datatypes (`xsd:string` and `rdf:langString` respectively), whereas `roxi`'s native parser/encoder leaves their datatypes as `None`.
- **Where**: `lib/src/oxrdf_adapter.rs` line 104–110.
- **Why**: This breaks term equality and term interning. A native literal `"hello"` (parsed with `datatype = None`) will have a different ID and fail to match against a roundtripped literal `"hello"^^xsd:string` returned from `oxrdf_term_to_roxi_term`. This breaks SPARQL queries and shape validation reports that expect the original native term.
- **Suggestion**: Map `datatype` to `None` if it is `xsd:string` (and `lang` is `None`), or if it is `rdf:langString` (and `lang` is `Some`).
  ```rust
  let lang = literal.language().map(|l| l.to_string());
  let datatype = if lang.is_some() {
      None
  } else {
      let dt = literal.datatype().as_str();
      if dt == "http://www.w3.org/2001/XMLSchema#string" {
          None
      } else {
          Some(format!("<{}>", dt))
      }
  };
  ```

### [Major] Finding 2: Pre-existing compilation errors in `lib/src/sparql.rs`

- **What**: The repository fails to compile when compiling the entire workspace due to a pattern mismatch on `PlanNode::Extend` in `lib/src/sparql.rs`.
- **Where**: `lib/src/sparql.rs` line 484.
- **Why**: `PlanNode::Extend` variant fields have changed or are mismatched, expecting `expression` instead of `from` and `to`. This blocks workspace-wide testing unless cached builds are used.
- **Suggestion**: Restoring compilation of `lib/src/sparql.rs` is required by the team working on Milestone 3.

### [Minor] Finding 3: Panic on Quoted Triples (`oxrdf::Term::Triple`)

- **What**: `oxrdf_term_to_roxi_term` panics when encountering `oxrdf::Term::Triple(_)`.
- **Where**: `lib/src/oxrdf_adapter.rs` line 111.
- **Why**: Panicking might crash a long-running server or shape validation process if an input contains a quoted triple (RDF-star).
- **Suggestion**: Explicitly document this constraint, or handle it gracefully (e.g. by returning an error instead of panicking, though that would require updating the function signature).

---

## 4. Verified Claims

- `triple_index_to_oxrdf_graph` correctly translates native terms to oxrdf terms → verified via `cargo test -p roxi --test oxrdf_adapter` → **PASS**
- Blank node identity is preserved across one-way conversion → verified via `test_blank_node_identity_preserved` → **PASS**
- Literal datatype and langtag are preserved across one-way conversion → verified via `test_literal_datatype_langtag_preserved_across_adapter` → **PASS**

---

## 5. Coverage Gaps

- **Roundtrip Literal Equality** — risk level: **HIGH** — recommendation: Investigate and apply the suggested fix in `oxrdf_term_to_roxi_term` so that simple and language-tagged literals keep `datatype = None` in alignment with native Roxi parser/encoder behavior.
- **Quoted Triples (RDF-Star)** — risk level: **LOW** — recommendation: Accept risk as a documented limitation since Roxi lacks RDF-star support.

---

## 6. Unverified Items

- Roundtripped tests using `oxrdf_term_to_roxi_term` in clean workspace builds — reason not verified: `lib/src/sparql.rs` pre-existing compilation error blocks clean workspace cargo builds.

---

## 7. Verification Method

To verify the test target specifically without triggering full compilation:
```bash
cargo test -p roxi --test oxrdf_adapter
```
To verify the compiler blocker on the entire workspace:
```bash
cargo test --workspace
```
