# Handoff Report: TICKET-007 — `oxrdf` Adapter Layer Implementation

## 1. Observation

- **Implementation**:
  - The adapter layer has been successfully implemented in `/Users/sac/roxi/lib/src/oxrdf_adapter.rs`.
  - The `oxrdf_adapter` module has been registered in `/Users/sac/roxi/lib/src/lib.rs` (`pub mod oxrdf_adapter;`).
  - Helper constructors `new_literal` and `new_blank_node` have been added to `impl VarOrTerm` in `/Users/sac/roxi/lib/src/triples.rs`.
  - The integration tests under `/Users/sac/roxi/lib/tests/oxrdf_adapter.rs` have been uncommented.

- **oxrdf API (0.3.3) Mismatches Resolved**:
  - Unconditional matching: Since the workspace dependency `oxrdf` compiles with the feature `rdf-12` enabled, the `oxrdf::Term::Triple` variant is always present in `oxrdf::Term`. The conditional compilation `#[cfg(feature = "rdf-12")]` attribute was removed so the variant is matched unconditionally, preventing a non-exhaustive pattern match error.
  - Temporary Lifetime Borrow: In `triple_index_to_oxrdf_graph`, when calling `graph.insert(...)`, we borrow the temporary triple using `&oxrdf::Triple::new(subject, predicate, object)` to satisfy the `Into<TripleRef<'a>>` trait bound implementation `impl<'a> From<&'a Triple> for TripleRef<'a>`.
  - Reference Types Destructuring: The fields of `oxrdf::TripleRef` returned by `graph.iter()` are of type `NamedOrBlankNodeRef<'_>` and `TermRef<'_>`. The tests were updated to import `NamedOrBlankNodeRef` and `TermRef` and match against their variants (e.g. `TermRef::BlankNode`), avoiding compilation type mismatches with owned types `Subject` and `Term`.
  - Deprecation warnings: Usage of the deprecated type alias `Subject` in the tests was migrated to `NamedOrBlankNode` and `NamedOrBlankNodeRef`.

- **Test Execution**:
  The command `cargo test -p roxi --test oxrdf_adapter` yields:
  ```text
  running 3 tests
  test test_triple_index_to_oxrdf_graph_roundtrip ... ok
  test test_blank_node_identity_preserved ... ok
  test test_literal_datatype_langtag_preserved_across_adapter ... ok

  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```

---

## 2. Logic Chain

1. **Adapter Modifiers**:
   - `lib/src/lib.rs`: Exposing `pub mod oxrdf_adapter;` is required to allow integration tests and external consumers to access the module.
   - `lib/src/triples.rs`: Implementing `VarOrTerm::new_literal` and `VarOrTerm::new_blank_node` delegates to the global `Encoder::add_literal` and `Encoder::add_blank_node` to properly allocate internal IDs and construct native term structures.

2. **API Alignment**:
   - The original code was designed for an older or different `oxrdf` version where `graph.insert()` took owned `Triple` or matched on owned `Term`.
   - In `oxrdf` version `0.3.3`, the graph operates on references via `TripleRef` / `TermRef` / `NamedOrBlankNodeRef`. Matching the test assertions to these types allows the integration test to compile correctly and assert the values of subject/object terms.

---

## 3. Caveats

- **Quoted Triples**: As identified by the explorer, `oxrdf::Term::Triple` represents a quoted triple (RDF-star). Since `roxi` lacks RDF-star support, any attempt to convert an `oxrdf::Term::Triple` via `oxrdf_term_to_roxi_term` will panic. This is the expected and documented design constraint.

---

## 4. Conclusion

The `oxrdf` adapter layer is successfully implemented and fully verified. The integration tests compile cleanly and pass without regressions in the existing test suites.

---

## 5. Verification Method

To verify the correct operation of the adapter layer, run:
```bash
cargo test -p roxi --test oxrdf_adapter
```
All three tests (`test_triple_index_to_oxrdf_graph_roundtrip`, `test_blank_node_identity_preserved`, `test_literal_datatype_langtag_preserved_across_adapter`) should compile and pass successfully.
