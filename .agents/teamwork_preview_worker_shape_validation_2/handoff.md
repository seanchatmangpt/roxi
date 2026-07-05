# Handoff Report: TICKET-007 — `oxrdf` Adapter Layer Literal Roundtrip Bugfix

## 1. Observation

- **Modified File**: `/Users/sac/roxi/lib/src/oxrdf_adapter.rs`
- **Initial Verification & Error**:
  - Running `cargo test -p roxi --test oxrdf_adapter` initially failed with 2 errors in the roundtrip tests:
    ```text
    ---- test_literal_lang_roundtrip_equality stdout ----
    thread 'test_literal_lang_roundtrip_equality' (381156) panicked at lib/tests/oxrdf_adapter.rs:226:5:
    assertion `left == right` failed
      left: Literal(LiteralImpl { id: 2, value: 0, datatype: None, lang: Some(1) })
     right: Literal(LiteralImpl { id: 18, value: 0, datatype: Some(17), lang: Some(1) })

    ---- test_simple_literal_roundtrip_equality stdout ----
    thread 'test_simple_literal_roundtrip_equality' (381158) panicked at lib/tests/oxrdf_adapter.rs:249:5:
    assertion `left == right` failed
      left: Literal(LiteralImpl { id: 19, value: 0, datatype: Some(13), lang: None })
     right: Literal(LiteralImpl { id: 16, value: 0, datatype: None, lang: None })
    ```

- **Original Code**:
  - In `lib/src/oxrdf_adapter.rs`:
    ```rust
    oxrdf::Term::Literal(literal) => {
        let lexical = literal.value().to_string();
        let datatype = Some(format!("<{}>", literal.datatype().as_str()));
        let lang = literal.language().map(|l| l.to_string());
        let id = Encoder::add_literal(lexical, datatype, lang);
        Encoder::decode_to_term(id).expect("Successfully decoded just-added Literal term")
    }
    ```

- **Modified Code**:
  - In `lib/src/oxrdf_adapter.rs`:
    ```rust
    oxrdf::Term::Literal(literal) => {
        let lexical = literal.value().to_string();
        let lang = literal.language().map(|l| l.to_string());
        let dt = literal.datatype().as_str();
        let datatype = if dt == "http://www.w3.org/2001/XMLSchema#string"
            || dt == "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"
        {
            None
        } else {
            Some(format!("<{}>", dt))
        };
        let id = Encoder::add_literal(lexical, datatype, lang);
        Encoder::decode_to_term(id).expect("Successfully decoded just-added Literal term")
    }
    ```

- **Final Verification**:
  - Running `cargo test -p roxi --test oxrdf_adapter` completed successfully:
    ```text
    running 6 tests
    test test_literal_lang_roundtrip_equality ... ok
    test test_blank_node_identity_preserved ... ok
    test test_literal_datatype_langtag_preserved_across_adapter ... ok
    test test_oxrdf_adapter_robustness ... ok
    test test_simple_literal_roundtrip_equality ... ok
    test test_triple_index_to_oxrdf_graph_roundtrip ... ok

    test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    ```

---

## 2. Logic Chain

1. **Roxi Native Representation**:
   - Roxi encodes simple literals (e.g. `"hello"`) and language-tagged literals (e.g. `"hello"@en`) with `datatype = None` (see `lib/src/encoding.rs`).
2. **Oxigraph Representation**:
   - `oxrdf` represents all literals with explicit datatypes (implicitly `http://www.w3.org/2001/XMLSchema#string` for simple strings, and `http://www.w3.org/1999/02/22-rdf-syntax-ns#langString` for language-tagged strings).
3. **Mismatched Roundtrip ID Allocation**:
   - Converting the `oxrdf::Term` back to a Roxi `Term` without removing those implicit datatypes resulted in `datatype = Some(...)` instead of `None`.
   - This caused the `Encoder` to allocate a new internal ID, breaking the term identity and resulting in assertion failures in the roundtrip tests (`test_simple_literal_roundtrip_equality` and `test_literal_lang_roundtrip_equality`).
4. **Resolution via Normalization**:
   - By mapping `http://www.w3.org/2001/XMLSchema#string` and `http://www.w3.org/1999/02/22-rdf-syntax-ns#langString` to `None` in `oxrdf_term_to_roxi_term`, we ensure that simple and language-tagged literals roundtrip to the exact same native term representation (and thus the same ID).
   - This aligns the converted terms with the native Roxi parser/encoder behavior.

---

## 3. Caveats

- **RDF-star Quoted Triples**:
  - `oxrdf::Term::Triple` is not supported by Roxi and explicitly panics, as documented in `lib/src/oxrdf_adapter.rs`.
- **Named Graphs**:
  - Named graph components (`triple.g`) are discarded during conversion because `oxrdf::Graph` only stores triples. This is expected.

---

## 4. Conclusion

- The critical literal roundtrip bug in TICKET-007 has been successfully resolved by normalizing `xsd:string` and `rdf:langString` datatypes to `None` when translating from `oxrdf::Term::Literal` to Roxi `Term`.
- The fix restores the identity invariant for basic literals, enabling correct SPARQL query evaluations and SHACL/ShEx shape validation over roundtripped terms.

---

## 5. Verification Method

To verify the fix independently, execute:
```bash
cargo test -p roxi --test oxrdf_adapter
```
Check that all 6 tests in `lib/tests/oxrdf_adapter.rs` pass successfully.
