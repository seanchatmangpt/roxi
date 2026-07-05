# Handoff Report — oxrdf Adapter Layer Robustness Challenge

## 1. Observation

### Observation A: Workspace compilation failure
When attempting to run workspace tests after clean (`cargo clean && cargo test --workspace --lib`), the compilation fails with the following errors in `lib/src/sparql.rs`:
```
error[E0026]: variant `PlanNode::Extend` does not have a field named `from`
   --> lib/src/sparql.rs:509:35
    |
509 |         PlanNode::Extend { child, from, to } => {
    |                                   ^^^^
    |                                   |
    |                                   variant `PlanNode::Extend` does not have this field
    |                                   help: `PlanNode::Extend` has a field named `expression`

error[E0027]: pattern does not mention field `expression`
   --> lib/src/sparql.rs:509:9
    |
509 |         PlanNode::Extend { child, from, to } => {
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing field `expression`
```

### Observation B: PlanNode::Extend Definition
The `PlanNode` enum in `lib/src/sparql.rs:74-78` defines the `Extend` variant as:
```rust
    Extend {
        child: Box<Self>,
        expression: PlanExpression,
        to: Variable,
    },
```

### Observation C: oxrdf Literal Translation to roxi Literal
In `lib/src/oxrdf_adapter.rs:104-110`:
```rust
        oxrdf::Term::Literal(literal) => {
            let lexical = literal.value().to_string();
            let datatype = Some(format!("<{}>", literal.datatype().as_str()));
            let lang = literal.language().map(|l| l.to_string());
            let id = Encoder::add_literal(lexical, datatype, lang);
            Encoder::decode_to_term(id).expect("Successfully decoded just-added Literal term")
        }
```

### Observation D: Native roxi Literal Parsing
In `lib/src/encoding.rs:92-108`:
```rust
        } else if s.starts_with('"') {
            let last_quote = s.rfind('"');
            if let Some(end_lex) = last_quote {
                if end_lex > 0 {
                    let lexical = s[1..end_lex].to_string();
                    let suffix = &s[end_lex + 1..];
                    let mut datatype = None;
                    let mut lang = None;
                    if suffix.starts_with('@') {
                        lang = Some(suffix[1..].to_string());
                    } else if suffix.starts_with("^^") {
                        datatype = Some(suffix[2..].to_string());
                    }
                    self.add_literal(lexical, datatype, lang)
```

---

## 2. Logic Chain

1. **Workspace Compilation Failure**: 
   - Based on *Observation A*, compiling `roxi` fails.
   - Based on *Observation B*, the enum variant `PlanNode::Extend` contains `expression` (of type `PlanExpression`) and `to` (of type `Variable`), but no field `from`.
   - The match pattern at line 509 tries to destructure `PlanNode::Extend { child, from, to }`. This causes compiler errors `E0026` and `E0027`. This indicates parallel work in TICKET-006 (SPARQL 1.1 Conformance) refactored the definition of `PlanNode::Extend` but did not update the `evaluate_plan` function.

2. **Term Identity Mismatches**:
   - In `roxi`'s native literal parsing (*Observation D*), a simple literal like `"hello"` is parsed into `EncodedValue::Literal` with `datatype = None` and `lang = None`.
   - A language-tagged literal like `"hello"@en` is parsed into `EncodedValue::Literal` with `datatype = None` and `lang = Some("en")`.
   - However, in the `oxrdf` adapter layer (*Observation C*), simple literals and language-tagged literals are converted by passing both `datatype` and `lang` directly from the `oxrdf::Literal` representation.
   - In `oxrdf`, a simple literal has `datatype() == xsd:string` (`http://www.w3.org/2001/XMLSchema#string`), so the adapter passes `datatype = Some("<http://www.w3.org/2001/XMLSchema#string>")` and `lang = None`.
   - In `oxrdf`, a language-tagged literal has `datatype() == rdf:langString` (`http://www.w3.org/1999/02/22-rdf-syntax-ns#langString`), so the adapter passes both `datatype = Some("<http://www.w3.org/1999/02/22-rdf-syntax-ns#langString>")` and `lang = Some("en")`.
   - Because `EncodedValue` in the encoder utilizes structural equality, these different inputs lead to distinct `EncodedValue` variants (e.g. `datatype = None` vs `datatype = Some(...)`).
   - Consequently, the same literal (e.g. `"hello"` or `"hello"@en`) will have two different IDs: one for the native term parsed by `roxi`, and one for the term converted back from `oxrdf`. This breaks term identity round-trips and matching in triple indexes / queries.

3. **Validation Bypassing**:
   - The adapter uses `NamedNode::new_unchecked` and `BlankNode::new_unchecked` without validation. If `roxi`'s triple index contains any malformed IRIs (like `<invalid uri`) or blank nodes with invalid identifiers, these are passed directly to `oxrdf` which can cause logic bugs or crashes downstream during SHACL/ShEx validation.

4. **Robustness of RDF-star Handling**:
   - Encountering an RDF-star nested triple (`oxrdf::Term::Triple`) in `oxrdf_term_to_roxi_term` causes a direct process-level panic. This makes the validation layer fragile against graphs with RDF-star features.

---

## 3. Caveats

- Since we are restricted to a review-only role and must not modify implementation code, we could not verify these findings by executing a fixed build of the tests.
- SHACL and ShEx validation conformance suites (`lib/tests/shacl_conformance/main.rs` and `lib/tests/shex_conformance/main.rs`) are currently mocks and do not test live validation behavior against the underlying crates.

---

## 4. Conclusion

- **Overall assessment**: **HIGH RISK**.
- The workspace compilation error in `lib/src/sparql.rs` blocks all tests and must be resolved by the TICKET-006 implementer/orchestrator.
- The `oxrdf` adapter layer has a critical logic bug causing term ID mismatches on round-trips for simple and language-tagged literals, which will break matching inside the engine.
- Minor issues include unchecked named/blank node creation and process panics on RDF-star.

---

## 5. Verification Method

To verify the compilation failure:
1. Run `cargo clean && cargo test --workspace --lib` in the workspace root.
2. Confirm the build errors out on `lib/src/sparql.rs` at line 509.

To verify term identity mismatches (once compilation is fixed):
1. Add a test in `lib/tests/oxrdf_adapter.rs`:
   ```rust
   #[test]
   fn test_term_identity_roundtrip() {
       let native_lit = VarOrTerm::new_literal("hello".to_string(), None, Some("en".to_string()));
       let native_term = native_lit.as_term();
       
       let mut index = TripleIndex::new();
       let s = VarOrTerm::convert("http://example.org/s".to_string());
       let p = VarOrTerm::convert("http://example.org/p".to_string());
       index.add(Triple { s, p, o: native_lit, g: None });
       
       let graph = triple_index_to_oxrdf_graph(&index);
       let oxrdf_triple = graph.iter().next().unwrap();
       let converted_term = oxrdf_term_to_roxi_term(&oxrdf_triple.object);
       
       assert_eq!(native_term, converted_term, "Native term and converted term must match");
   }
   ```
2. Run `cargo test -p roxi --test oxrdf_adapter`. The test will fail because the term IDs differ.
