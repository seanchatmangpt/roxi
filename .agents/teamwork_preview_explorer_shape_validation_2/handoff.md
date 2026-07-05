# Handoff Report: TICKET-008 - SHACL Validation & W3C Data-Shapes Conformance

This report details the exploration and analysis of SHACL validation integration for the `roxi` RDF engine, including compatibility checks on registry crates, the structure of conformance tests, and an actionable fallback implementation strategy.

---

## 1. Observation

### A. Cargo Registry Search and Dependency Conflicts
We searched the cargo registry for `shacl_validation` and `shacl_ast` and found the latest versions:
- `shacl_validation = "0.2.12"`
- `shacl_ast = "0.2.9"`

To verify their compilation, we created an isolated Cargo scratchpad using these dependencies. Running `cargo check` revealed a major SemVer break in the transitive dependencies of the `rudof` ecosystem:
```
error[E0308]: mismatched types
  --> /Users/sac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/shacl_ast-0.2.9/src/ast/component.rs:77:46
   |
77 |                 Self::write_iri(&IriRef::Iri(iri!(iri)), ShaclVocab::SH_DATATYPE, rdf_node, rdf)?;
   |                                  ----------- ^^^^^^^^^ expected `rudof_iri::iri::iris::IriS`, found `iri_s::IriS`
   |                                  |
   |                                  arguments to this enum variant are incorrect
   |
   = note: `iri_s::IriS` and `rudof_iri::iri::iris::IriS` have similar names, but are actually distinct types
```
And:
```
error[E0271]: expected `IntoIter<IriS>` to be an iterator that yields `IriS`, but it yields `IriS`
   --> /Users/sac/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/sparql_service-0.2.12/src/service_description.rs:157:28
    |
157 |             mie.add_graphs(graph_names.clone().into_iter());
    |                 ---------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `rudof_iri::iri::iris::IriS`, found `iri_s::IriS`
```

These errors arise because:
1. `shacl_ast v0.2.9` and `sparql_service v0.2.12` reference the `IriS` type from the `iri_s` crate.
2. The resolved latest minor versions of their transitive dependencies (`prefixmap v0.2.20` and `mie v0.2.20`) have migrated to reference `IriS` from the newer `rudof_iri` crate.
3. Due to pre-1.0 SemVer loose version constraints (e.g. `prefixmap = "^0.2.9"` in `shacl_ast`), Cargo automatically updates the transitive dependencies to `v0.2.20`, creating type conflicts that render these crates uncompilable in stable Rust. Locking `prefixmap` and other crates to version `0.2.9` still causes compilation failures in `sparql_service` because `mie` gets resolved to `v0.2.20` and conflicts.

### B. Current SHACL Test Suite and Conformance Directory
We inspected the `lib/tests/shacl_conformance/` directory:
- `lib/tests/shacl_conformance/main.rs` contains a placeholder mock test:
```rust
#[test]
fn test_shacl_conformance() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mock_dir = manifest_dir.join("tests/shacl_conformance/mock_suite");
    ...
    let conforms = true;
    assert!(conforms, "SHACL validation must conform");
    ...
    let manifest_report = format!(
        "# SHACL Conformance Pass-Rate Manifest\n\n\
        - **Dialect**: SHACL\n\
        - **Suite**: W3C data-shapes Test Suite (Mock Suite)\n\
        - **Total Tests**: 1\n\
        - **Passed**: 1\n..."
    );
    fs::write(manifests_dir.join("shacl_manifest.md"), manifest_report)...
}
```
- `lib/tests/shacl_conformance/mock_suite/` contains mock files:
  - `data.ttl`:
    ```turtle
    <http://example.org/User1> a <http://example.org/User> ;
        <http://example.org/name> "Alice" .
    ```
  - `shapes.ttl`:
    ```turtle
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    @prefix ex: <http://example.org/> .

    ex:UserShape a sh:NodeShape ;
        sh:targetClass ex:User ;
        sh:property [
            sh:path ex:name ;
            sh:minCount 1 ;
        ] .
    ```
  - `expected_report.ttl`:
    ```turtle
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    [ a sh:ValidationReport ;
      sh:conforms true
    ] .
    ```
- W3C `data-shapes` conformance tests are not currently vendored under `lib/tests/shacl_conformance/`.

---

## 2. Logic Chain

1. **Broken Registry Crates**: Because `shacl_validation` and `shacl_ast` are unusable due to the transitive `iri_s`/`rudof_iri` type mismatch in the public registry, **we cannot depend on them** for the SHACL validator implementation.
2. **Fallback Decision**: To satisfy JIRA TICKET-008 without external registry dependencies, the project must implement a **minimal, hand-rolled core SHACL validator** inside `lib/src/shacl.rs`.
3. **No Network Access**: In `CODE_ONLY` mode, we cannot fetch the W3C conformance suite at test time. Therefore, the official W3C `data-shapes` test suite ( Turtle files containing the test graphs and manifests) **must be vendored** locally under `lib/tests/shacl_conformance/w3c_suite/`.
4. **Adapter Layer Leverage**: The adapter layer (`oxrdf_adapter.rs`) converts roxi-native `TripleIndex` to `oxrdf::Graph`. A hand-rolled validator can either query this `oxrdf::Graph` or, more simply and efficiently, query the roxi-native `TripleIndex` directly using roxi's native `query_help` API, bypassing `oxrdf` entirely.

---

## 3. Caveats

- **SPARQL-based Constraints**: TICKET-008 mentions "SPARQL-based constraint components". Implementing a complete SPARQL-based SHACL validator requires integrating the validator with our SPARQL evaluation engine (`sparql.rs`). Since SPARQL evaluation is implemented in the repository, we can delegate SPARQL constraints to the existing SPARQL engine.
- **Recursion and Cycled Shapes**: The fallback SHACL validator assumes non-recursive shapes. Deeply nested or recursive shapes (e.g. shapes referencing themselves via `sh:property/sh:node`) must be validated carefully to avoid infinite loops, which can be handled using a simple visited-set recursion prevention mechanism.

---

## 4. Conclusion

### Proposed Integration Strategy (Fallback Implementation)

We propose implementing the SHACL validation framework from scratch in `lib/src/shacl.rs` using roxi-native data structures:

#### A. Public Interface
```rust
pub struct ShapesGraph {
    pub shapes: Vec<Shape>,
}

pub struct ValidationReport {
    pub conforms: bool,
    pub results: Vec<ValidationResult>,
}

pub struct ValidationResult {
    pub focus_node: Term,
    pub result_path: Option<Term>,
    pub value: Option<Term>,
    pub source_constraint_component: Term,
    pub source_shape: Term,
    pub severity: Term,
}

impl ShapesGraph {
    pub fn parse(shapes_str: &str) -> Result<Self, String> {
        // Use Parser::parse_triples(shapes_str, Syntax::Turtle) to load shapes into a TripleIndex.
        // Traverse the shapes TripleIndex to build a structured representation of NodeShapes and PropertyShapes.
    }
}

pub struct Validator;
impl Validator {
    pub fn validate(data: &TripleIndex, shapes: &ShapesGraph) -> ValidationReport {
        // Evaluate each target declaration in shapes against the data TripleIndex.
        // For each focus node, evaluate constraints and collect validation results.
    }
}
```

#### B. Shape Parsing and Target Retrieval
A shape targets nodes in the data graph using:
- **`sh:targetClass`**: Find all `(S, sh:targetClass, C)` in the shapes index, then find all `(F, rdf:type, C)` in the data index.
- **`sh:targetNode`**: Focus node is explicitly declared via `(S, sh:targetNode, F)`.
- **`sh:targetSubjectsOf`**: Find all `(F, P, _)` where `(S, sh:targetSubjectsOf, P)` is in shapes.
- **`sh:targetObjectsOf`**: Find all `(_, P, F)` where `(S, sh:targetObjectsOf, P)` is in shapes.

#### C. Validation Logic (Core Constraints)
For each focus node `F` targeted by shape `S`:
1. **Property constraints** (`sh:property`):
   - Locate property shape `PS` and path `P` from `(PS, sh:path, P)`.
   - Retrieve all value nodes `V` matching `(F, P, v)` in the data graph.
   - Evaluate constraints on the set `V`:
     - **`sh:minCount N`**: Verify `|V| >= N`.
     - **`sh:maxCount N`**: Verify `|V| <= N`.
     - **`sh:datatype D`**: For each `v` in `V`, check if `v` is a literal with datatype `D`.
     - **`sh:class C`**: For each `v` in `V`, check if `(v, rdf:type, C)` exists in the data graph.
     - **`sh:pattern Pat`**: Check if the string representation of each `v` matches regex `Pat`.
     - **`sh:in List`**: Check if `v` is in the RDF list `List` (traverse `rdf:first`/`rdf:rest` chains in shapes).
2. **Logical constraints** on focus node `F`:
   - **`sh:and List`**: `F` must conform to all shapes in `List`.
   - **`sh:or List`**: `F` must conform to at least one shape in `List`.
   - **`sh:not Shape`**: `F` must NOT conform to the given `Shape`.

#### D. Translating ValidationReport back to Roxi Triples
To export/convert the `ValidationReport` to RDF triples (e.g. for serialization or comparison in tests):
- Generate a blank node `_:report` of type `sh:ValidationReport`.
- Add `(_:report, sh:conforms, true/false)` to a result list.
- For each violation, generate a blank node `_:result` of type `sh:ValidationResult`.
- Attach `sh:focusNode`, `sh:resultPath`, `sh:value`, `sh:sourceConstraintComponent`, `sh:sourceShape`, and `sh:resultSeverity` to `_:result`.
- Link `(_:report, sh:result, _:result)`.

#### E. Test Suite Vendoring and Execution
1. Extract the official W3C SHACL test cases (e.g., `core/node/` and `core/property/`) into `lib/tests/shacl_conformance/w3c_suite/`.
2. Rewrite `lib/tests/shacl_conformance/main.rs` to:
   - Walk the `w3c_suite/` directory to locate manifest files (`manifest.ttl`).
   - Parse each manifest to extract test definitions (`action` containing data/shapes graphs, and `result` containing the expected validation report).
   - Load graphs into `TripleIndex`, invoke our `Validator::validate`, and compare the resulting report graph against the expected report graph.
   - Generate and write `docs/jira/26.7.4/manifests/shacl_manifest.md` detailing the test run and 100% pass rate.

---

## 5. Verification Method

### How to Verify
1. **Locate Code Files**:
   - Verify `lib/src/shacl.rs` contains the validator module.
   - Verify `lib/src/lib.rs` declares `pub mod shacl;`.
   - Verify `lib/tests/shacl_validation.rs` contains integration tests for core constraints (`test_min_max_count_violation`, etc.).
   - Verify `lib/tests/shacl_conformance/w3c_suite/` contains the vendored W3C Turtle test files.
2. **Run Tests**:
   - Execute the test suite using Cargo:
     ```bash
     cargo test --test shacl_conformance
     cargo test --test shacl_validation
     ```
3. **Inspect Output Manifest**:
   - Confirm that `docs/jira/26.7.4/manifests/shacl_manifest.md` was created and records a `100.0%` pass rate across the vendored W3C tests.
