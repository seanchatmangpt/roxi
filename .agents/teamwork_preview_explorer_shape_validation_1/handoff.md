# Handoff Report: TICKET-007 — `oxrdf` Adapter Layer

## 1. Observation

- **Dependency Configuration**:
  In `/Users/sac/roxi/lib/Cargo.toml` (line 22), `oxrdf` is declared as:
  ```toml
  oxrdf = { version = "0.3.3", features = ["rdf-12"] }
  ```
  It compiles successfully in the current workspace.
  
- **Roxi Term Model**:
  In `lib/src/triples.rs` (lines 96-100), the native term model defines:
  ```rust
  pub enum Term {
      Iri(TermImpl),
      Literal(LiteralImpl),
      BlankNode(BlankNodeImpl),
  }
  ```
  In `lib/src/encoding.rs` (lines 197-213), the global `Encoder` decodes these from `usize` keys into `Term`:
  ```rust
  pub fn decode_to_term(&self, id: usize) -> Option<Term> {
      match self.decoded.get(&id)? {
          EncodedValue::Iri(_) => Some(Term::Iri(TermImpl { iri: id })),
          EncodedValue::BlankNodeLabel(_) => Some(Term::BlankNode(BlankNodeImpl { id })),
          EncodedValue::Literal { value, datatype, lang } => Some(Term::Literal(LiteralImpl {
              id,
              value: *value,
              datatype: *datatype,
              lang: *lang,
          })),
          _ => None,
      }
  }
  ```

- **OxRDF API (0.3.3)**:
  By inspecting the cached crates under `/Users/sac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/oxrdf-0.3.3/src/`, we observed:
  - `NamedNode::new` (validates and builds a node, returning `Result<Self, IriParseError>`) and `NamedNode::new_unchecked` (builds a node directly).
  - `BlankNode::new` (validates and builds, returning `Result<Self, BlankNodeIdParseError>`) and `BlankNode::new_unchecked` (builds directly).
  - `Literal::new_simple_literal`, `Literal::new_typed_literal`, and `Literal::new_language_tagged_literal_unchecked` (or `Literal::new_language_tagged_literal`).
  - Methods to read literals: `Literal::value` (returns `&str`), `Literal::datatype` (returns `NamedNodeRef`), and `Literal::language` (returns `Option<&str>`).
  - Enums `Term` (variants: `NamedNode`, `BlankNode`, `Literal`, and `Triple` if `rdf-12` is enabled) and `NamedOrBlankNode` (formerly aliased to `Subject`).
  - `Graph::insert` (inserts an `oxrdf::Triple`) and `Graph::iter` (yields `TripleRef`).

- **Test Suite Requirements**:
  In `/Users/sac/roxi/lib/tests/oxrdf_adapter.rs` (which is currently commented out), the following constructors are called on `VarOrTerm`:
  - `VarOrTerm::new_literal(value, datatype, lang)` (lines 49-53, 56-60)
  - `VarOrTerm::new_blank_node(label)` (line 94, 96)
  These constructors are not yet defined in `lib/src/triples.rs`.

---

## 2. Logic Chain

1. **Delimiters in Internal Encoding**:
   Roxi's `Encoder` stores IRIs wrapped in angle brackets (e.g. `"<http://example.org/s>"`) and blank nodes with `_:` prepended (e.g. `"_:b0"`). `oxrdf::NamedNode` and `oxrdf::BlankNode` expect raw strings (e.g. `"http://example.org/s"`, `"b0"`). Therefore, `triple_index_to_oxrdf_graph` must strip these delimiters during conversion using `clean_iri` and `clean_blank_node` helper functions before calling `oxrdf` constructors.
   
2. **Reverse Conversions**:
   To convert from `oxrdf::Term` back to Roxi's `Term`, we must restore the delimiters so they are registered correctly in the global `Encoder`:
   - `NamedNode(node)`: wrapped as `format!("<{}>", node.as_str())`, added to `Encoder`, and decoded.
   - `BlankNode(node)`: wrapped as `format!("_:{}", node.as_str())`, added to `Encoder`, and decoded.
   - `Literal(literal)`: extracted via `value()`, datatype formatted with `<...>`, and language tag retrieved. These are passed to `Encoder::add_literal` before decoding.

3. **Exhaustive Matching on OxRDF Term**:
   Since the `rdf-12` feature is enabled on `oxrdf` inside `Cargo.toml`, `oxrdf::Term` contains a `Triple` variant for RDF-star quoted triples. Matching on `oxrdf::Term` must handle `oxrdf::Term::Triple` to compile; since Roxi does not support quoted triples, the adapter must panic if one is encountered.

4. **Exposing API and Compiling Tests**:
   - `oxrdf_adapter` module must be declared in `lib/src/lib.rs`.
   - `new_literal` and `new_blank_node` helpers must be added to `impl VarOrTerm` block in `lib/src/triples.rs` to allow the integration tests to compile.
   - The test block in `lib/tests/oxrdf_adapter.rs` must be uncommented.

---

## 3. Caveats

- **RDF-star (RDF 1.2) Unsupported**:
  If downstream shape-validators (SHACL/ShEx) pass graphs containing RDF-star triple terms to the reverse adapter, a panic will occur. This is an acceptable constraint since Roxi's core graph engine does not support RDF-star triples.
- **W3C SHACL/ShEx Validation Packages**:
  `shacl_validation` and `shex_validation` crates are not yet direct dependencies in `lib/Cargo.toml` and are not in `Cargo.lock`. They will be added in TICKET-008 and TICKET-009, so this investigation is strictly scoped to the `oxrdf` adapter.

---

## 4. Conclusion

We propose the following integration strategy:
1. Create `lib/src/oxrdf_adapter.rs` exactly as specified in the artifact `proposed_oxrdf_adapter.rs`.
2. Apply the patch `proposed_changes.patch` to update `lib/src/lib.rs` (module declaration), `lib/src/triples.rs` (adding literal and blank node constructors on `VarOrTerm`), and `lib/tests/oxrdf_adapter.rs` (uncommenting the test suite).

All artifacts are written to the working directory:
- `/Users/sac/roxi/.agents/teamwork_preview_explorer_shape_validation_1/proposed_oxrdf_adapter.rs`
- `/Users/sac/roxi/.agents/teamwork_preview_explorer_shape_validation_1/proposed_changes.patch`

---

## 5. Verification Method

To verify the adapter layer:
1. Copy the proposed adapter file to the codebase:
   ```bash
   cp /Users/sac/roxi/.agents/teamwork_preview_explorer_shape_validation_1/proposed_oxrdf_adapter.rs /Users/sac/roxi/lib/src/oxrdf_adapter.rs
   ```
2. Apply the proposed patch:
   ```bash
   git apply /Users/sac/roxi/.agents/teamwork_preview_explorer_shape_validation_1/proposed_changes.patch
   ```
3. Run the unit and integration test command:
   ```bash
   cargo test --workspace --test oxrdf_adapter
   ```
   All three tests (`test_triple_index_to_oxrdf_graph_roundtrip`, `test_literal_datatype_langtag_preserved_across_adapter`, and `test_blank_node_identity_preserved`) should compile and pass.

**Invalidation conditions**:
- Modifying the internal format of `VarOrTerm` or changing how `Encoder::add` wraps IRIs or BlankNodes.
- Updating `oxrdf` dependency to a version that alters the signatures of `NamedNode::new_unchecked`, `Literal::new_typed_literal`, or `Literal::destruct`.
