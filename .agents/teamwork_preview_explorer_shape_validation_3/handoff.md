# ShEx Validation & shexTest Conformance Analysis Report (TICKET-009)

## 1. Observation
The following source files, JIRA tickets, scope manifests, and cargo registries were examined to analyze TICKET-009:

- **JIRA Ticket `docs/jira/26.7.4/009-shex-validation-conformance.md`**:
  - Requires implementing a new module `lib/src/shex.rs` exposing ShEx parsing (ShExJ schema parser) and validation using `shex_validation` and `shex_ast` crates.
  - Dictates validating against the `oxrdf::Graph` produced by TICKET-007's adapter layer.
  - Requires vendoring the `shexTest` suite into `lib/tests/shex_conformance/` and running it end-to-end to target 100% conformance.
  
- **Cargo Registry Search (`cargo search` & `cargo info`)**:
  - Found `shex_validation` at version `0.3.6`.
  - Found `shex_ast` at version `0.3.6`.
  - Found `rudof_rdf` at version `0.3.6`.
  - Checking `Cargo.toml` for `shex_validation` and `rudof_rdf` showed a dependency on `oxrdf = "0.3.0"`.
  - Verified Roxi's own `lib/Cargo.toml` specifies `oxrdf = { version = "0.3.3", features = ["rdf-12"] }`, which is semver-compatible with `"0.3.0"` (both resolve to `0.3.3` in `Cargo.lock`).

- **Crate Source Inspection**:
  - In `shex_validation-0.3.6/src/validator.rs`, the validator accepts node validation requests via:
    ```rust
    pub fn validate_node_shape<S>(
        &mut self,
        node: &Node,
        shape: &ShapeLabel,
        rdf: &S,
        schema: &SchemaIR,
        maybe_nodes_prefixmap: &Option<PrefixMap>,
    ) -> Result<ResultShapeMap>
    where
        S: NeighsRDF + QueryRDF,
    ```
  - In `rudof_rdf-0.3.6/src/rdf_impl/oxigraph/in_memory.rs`, the type `OxigraphInMemory` implements `NeighsRDF` + `QueryRDF` as well as the `BuildRDF` trait, which provides:
    ```rust
    fn add_triple<S, P, O>(&mut self, subj: S, pred: P, obj: O) -> Result<(), Self::Err>
    ```
  - In `shex_ast-0.3.6/src/ast/schema.rs`, parsing is supported via:
    ```rust
    pub fn parse_schema(path: &Path) -> Result<Schema, SchemaJsonError>
    ```
    which deserializes the standard ShExJ schema format into the AST structure.
  - In `shex_ast-0.3.6/src/ir/ast2ir.rs`, the AST is compiled to IR:
    ```rust
    pub fn compile(
        &mut self,
        schema_ast: &SchemaAST,
        source_iri: &IriS,
        base: &Option<IriS>,
        compiled_schema: &mut SchemaIR,
        external_resolvers: &ExternalShapeResolverRegistry,
    ) -> CResult<()>
    ```

- **Roxi Tests & Mock Suite**:
  - `lib/tests/shex_conformance/mock_suite` exists and contains mock files: `data.ttl`, `schema.json` (mock ShExJ schema), `shape_map.json`, and `expected_result.json`.
  - `lib/tests/shex_conformance/main.rs` runs a mock check, writes a report to `docs/jira/26.7.4/manifests/shex_manifest.md` showing 100.0% pass rate.

- **Adapter Layer**:
  - `lib/tests/oxrdf_adapter.rs` references `oxrdf_term_to_roxi_term` and `triple_index_to_oxrdf_graph` as part of the planned TICKET-007 adapter layer.

---

## 2. Logic Chain
Based on the observations:

1. **Version Compatibility**: Both `shex_validation = "0.3.6"` and `shex_ast = "0.3.6"` are compatible with Roxi. They depend on `oxrdf = "0.3.0"`, which resolves to `oxrdf = "0.3.3"` under standard Rust semantic versioning, aligning perfectly with Roxi's dependency setup. No fallback decision or alternative versions are required.
2. **Data Model Bridge**:
   - Roxi stores triples in a native `TripleIndex` referencing native `Term`s.
   - `shex_validation` validates structures using traits `NeighsRDF` + `QueryRDF`.
   - `rudof_rdf::rdf_impl::OxigraphInMemory` implements `NeighsRDF` + `QueryRDF` and is backed by `oxrdf::Graph`.
   - Therefore, the integration strategy must convert native Roxi `TripleIndex` data to `OxigraphInMemory` using the adapter layer.
3. **Report Bridge**:
   - `shex_validation` returns `ResultShapeMap` containing `ValidationStatus` (which can be `Conformant`, `NonConformant`, etc.) for each checked node-shape association.
   - To report native errors referencing the original Roxi `Triple`s (as required by JIRA), we must extract the focus node's `oxrdf::Term` from `ResultShapeMap` and convert it back to Roxi's native `Term` using `oxrdf_adapter::oxrdf_term_to_roxi_term`.
4. **Harness Location**:
   - The mock suite is located in `lib/tests/shex_conformance/mock_suite`.
   - The full W3C `shexTest` conformance test suite files should be vendored into `lib/tests/shex_conformance/shexTest/` so they are isolated from other test runner assets.

---

## 3. Caveats
- **Dependency on TICKET-007**: The integration strategy assumes the presence of `oxrdf_adapter.rs` providing conversion functions `oxrdf_term_to_roxi_term` and `triple_index_to_oxrdf_graph`. If TICKET-007 is delayed, a minimal mock implementation of `oxrdf_adapter` must be provided in `lib/src/oxrdf_adapter.rs`.
- **Negative Syntax & Structural Tests**: The JIRA ticket focuses heavily on positive and negative validation tests. Syntax parsing tests (e.g. `negativeSyntax` or structural errors) are handled by `shex_ast` out of the box, but their integration is scoped separately from standard data graph validation.

---

## 4. Conclusion
We recommend proceeding with `shex_validation = "0.3.6"` and `shex_ast = "0.3.6"`. The integration strategy consists of:

### Proposed Architecture & Code Layout
1. **Module Declaration**: Create `lib/src/shex.rs` and add `pub mod shex;` in `lib/src/lib.rs`.
2. **Native Reporting Types**:
   Define Roxi-native validation result structures in `lib/src/shex.rs`:
   ```rust
   pub struct ShexValidationReport {
       pub conforms: bool,
       pub failures: Vec<ShexValidationFailure>,
   }

   pub struct ShexValidationFailure {
       pub node: Term, // Roxi native Term
       pub shape: String, // Shape label
       pub reason: String,
   }
   ```
3. **Data Adaptation & Validation Execution**:
   Implement a validator wrapper function in `lib/src/shex.rs`:
   ```rust
   use crate::tripleindex::TripleIndex;
   use crate::oxrdf_adapter::{triple_index_to_oxrdf_graph, oxrdf_term_to_roxi_term};
   use shex_ast::{ast::Schema as SchemaAST, ir::ast2ir::AST2IR, ResolveMethod, ShapeExprLabel};
   use shex_ast::ir::{schema_ir::SchemaIR, semantic_actions_registry::SemanticActionsRegistry, map_state::MapState, shape_label::ShapeLabel};
   use shex_validation::{Validator, ValidatorConfig};
   use rudof_rdf::rdf_impl::OxigraphInMemory;

   pub fn validate_shex(
       data: &TripleIndex,
       schema_json_str: &str,
       shape_map: &[(String, String)], // Focus node IRI/string -> Shape Label string
   ) -> Result<ShexValidationReport, Box<dyn std::error::Error>> {
       // 1. Parse Schema AST from ShExJ JSON
       let ast: SchemaAST = serde_json::from_str(schema_json_str)?;
       
       // 2. Compile AST to SchemaIR
       let mut map_state = MapState::default();
       let mut registry = SemanticActionsRegistry::default();
       registry.set_map_state(&mut map_state);
       let mut compiler = AST2IR::new(&ResolveMethod::default(), map_state);
       let mut compiled_schema = SchemaIR::new(registry);
       let dummy_base = rudof_iri::IriS::new_unchecked("http://dummy.org/base");
       
       compiler.compile(&ast, &dummy_base, &Some(dummy_base.clone()), &mut compiled_schema, &Default::default())?;
       
       // 3. Convert Roxi TripleIndex to oxrdf::Graph
       let ox_graph = triple_index_to_oxrdf_graph(data);
       
       // 4. Wrap in OxigraphInMemory implementing NeighsRDF + QueryRDF
       let mut in_memory_rdf = OxigraphInMemory::new();
       for triple in ox_graph.iter() {
           in_memory_rdf.add_triple(triple.subject.clone(), triple.predicate.clone(), triple.object.clone())?;
       }
       
       // 5. Run validator
       let mut validator = Validator::new(&compiled_schema, &ValidatorConfig::default())?;
       let mut failures = Vec::new();
       let mut overall_conforms = true;

       for (node_str, shape_str) in shape_map {
           let focus_node = shex_ast::Node::parse(node_str, None)?;
           let shape_label = shex_ast::ir::shape_label::ShapeLabel::iri(rudof_iri::IriS::new_unchecked(shape_str));
           
           let result_map = validator.validate_node_shape(
               &focus_node,
               &shape_label,
               &in_memory_rdf,
               &compiled_schema,
               &Some(in_memory_rdf.prefixmap().clone())
           )?;

           if let Some(status) = result_map.get_info(&focus_node, &shape_label) {
               if !status.is_conformant() {
                   overall_conforms = false;
                   let native_node = oxrdf_term_to_roxi_term(&focus_node.as_object());
                   failures.push(ShexValidationFailure {
                       node: native_node,
                       shape: shape_str.clone(),
                       reason: status.to_string(),
                   });
               }
           }
       }
       
       Ok(ShexValidationReport {
           conforms: overall_conforms,
           failures,
       })
   }
   ```

---

## 5. Verification Method
Verify version compatibility and compilation of the integrated code:
1. Update `lib/Cargo.toml` with the ShEx dependencies.
2. Build the workspace with `cargo check --workspace --all-targets` to verify all traits and crate version features resolve cleanly.
3. Once implementation starts, run the test target `cargo test --test shex_conformance` to verify the conformance test runner output.
