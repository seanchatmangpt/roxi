# Handoff Report — SHACL Validation Robustness Challenge

## 1. Observation
We inspected `lib/src/shacl.rs` and the conformance tests in `lib/tests/shacl_conformance/main.rs`. We observed the following behaviors and code blocks:

* **Non-deterministic Blank Node IDs**:
  In `lib/src/shacl.rs` (lines 155-156 and 191-193), blank nodes for reports and results are generated dynamically using system time:
  ```rust
  let report_id = format!("report_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
  let report_term = VarOrTerm::new_blank_node(report_id);
  // ...
  let res_id = format!("result_{}_{}", i, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
  let res_term = VarOrTerm::new_blank_node(res_id);
  ```

* **Broken Graph Isomorphism Check for Violations**:
  In `lib/tests/shacl_conformance/main.rs` (lines 32-61), the `graphs_isomorphic` function only maps the main validation report subject:
  ```rust
  let a_report_subjs = get_subjects(a, rdf_type, sh_validation_report);
  let b_report_subjs = get_subjects(b, rdf_type, sh_validation_report);
  // ...
  for t in a_triples {
      let s_mapped = if t.s.to_encoded() == a_subj { b_subj } else { t.s.to_encoded() };
      let p_mapped = t.p.to_encoded();
      let o_mapped = if t.o.to_encoded() == a_subj { b_subj } else { t.o.to_encoded() };
      
      if !contains_triple(b, s_mapped, p_mapped, o_mapped) {
          return false;
      }
  }
  ```
  Any validation result blank nodes are left unmapped.

* **Complex Inverse Path Operand Traversal**:
  In `lib/src/shacl.rs` (lines 606-614), inverse path traversal retrieves the operand `inv` and directly queries subjects of that predicate:
  ```rust
  let sh_inverse_path = Encoder::get("<http://www.w3.org/ns/shacl#inversePath>").unwrap_or(0);
  let inverse_path = get_objects(shapes, path_node, sh_inverse_path);
  if !inverse_path.is_empty() {
      let mut results = Vec::new();
      for inv in inverse_path {
          results.extend(get_subjects(data, inv, focus_node));
      }
      return results;
  }
  ```
  If `inv` is a blank node representing a complex path (e.g. `(ex:knows ex:knows)`), `get_subjects` receives the blank node identifier instead of recursively traversing the inverse.

* **Missing Regex Flag**:
  In `lib/src/shacl.rs` (lines 467-476), the regex flag `'x'` is not matched.

* **Test Suite Success**:
  We ran `cargo test --workspace --test shacl_validation --test shacl_conformance` and successfully verified that all 10 tests (including our 5 new comprehensive edge-case tests) compile and pass.

---

## 2. Logic Chain
1. In `graphs_isomorphic`, only the subject `sh:ValidationReport` blank node (`a_subj` -> `b_subj`) is mapped.
2. In `ValidationReport::to_triples`, each `ValidationResult` is given a unique blank node `res_term` generated with a nanosecond timestamp.
3. Therefore, if a validation report contains `ValidationResult` objects (conforms is false), `a_triples` will contain triples like `result_term_a rdf:type sh:ValidationResult` and `a_report_subj sh:result result_term_a`.
4. Since `graphs_isomorphic` does not map `result_term_a` to any corresponding blank node in graph `b`, the comparison `contains_triple(b, ...)` will look for `result_term_a` in graph `b` and fail, returning `false` even if the graphs are isomorphic under renaming of the result blank nodes.
5. In `eval_path`, evaluating `sh:inversePath` passes the inverse operand directly to `get_subjects(data, inv, focus_node)`. If the operand is a complex path (e.g. sequence path or alternative path), it is represented as a blank node identifier. Since blank nodes representing paths are not present as predicates in the data graph, `get_subjects` will find zero matches. Therefore, nested/complex paths within `sh:inversePath` are not correctly supported.
6. The regex helper matches flags `i`, `m`, and `s` but ignores `x` (extended mode), which violates the SHACL specification requiring support for `x`.

---

## 3. Caveats
* We assumed that the SHACL implementation only targets core constraints (`sh:minCount`, `sh:maxCount`, `sh:datatype`, `sh:class`, `sh:pattern`, `sh:in`, and logical operands). We did not investigate SPARQL-based constraints or advanced targeting mechanism components (e.g., target validation using query patterns).

---

## 4. Conclusion

### Challenge Summary

**Overall risk assessment**: MEDIUM

### Challenges

#### [High] Challenge 1: Non-deterministic report generation and broken graph isomorphism check for violations

* **Assumption challenged**: The conformance comparison helper `graphs_isomorphic` in `lib/tests/shacl_conformance/main.rs` assumes that only the main `sh:ValidationReport` blank node needs to be mapped during comparison.
* **Attack scenario**: If a conformance test case produces a validation report with one or more validation results (non-conforming dataset), the blank nodes generated for each `ValidationResult` (using `SystemTime` nanoseconds) will not be remapped. Consequently, the isomorphism check will fail because the generated result blank nodes in graph A will not match the result blank nodes in graph B.
* **Blast radius**: Conformance testing of failing test cases (which represent the majority of real-world SHACL test suites) is currently broken and cannot pass when using `graphs_isomorphic`.
* **Mitigation**: Update `graphs_isomorphic` to compute a full blank node mapping (e.g. signature-based isomorphism) or structure the blank node generator in `to_triples()` to use deterministic IDs (e.g., hash-based or sequence-based) instead of `SystemTime`.

#### [Medium] Challenge 2: Inability to evaluate complex paths inside `sh:inversePath`

* **Assumption challenged**: The implementation of `sh:inversePath` assumes the operand `inv` is a simple predicate IRI.
* **Attack scenario**: If a shape uses a complex path inside `sh:inversePath`, such as an alternative path or sequence path:
  `sh:path [ sh:inversePath (ex:parent ex:ancestor) ]`
  the evaluator will call `get_subjects(data, inv, focus_node)` with the blank node representing the sequence path. Because the sequence path blank node is not a predicate in the data graph, the evaluation will return an empty set of nodes, leading to incorrect validation results.
* **Blast radius**: Complex inverse paths are ignored or evaluated as empty.
* **Mitigation**: Implement a generic `eval_inverse_path(data, shapes, focus_node, path_node)` that computes the inverse traversal of arbitrary paths recursively.

#### [Low] Challenge 3: Lack of `sh:x` flag support in regular expressions

* **Assumption challenged**: The regular expression matcher assumes only `i`, `m`, and `s` flags are used.
* **Attack scenario**: If a shape defines a constraint using `sh:flags "x"`, the validator ignores it. Any pattern relying on extended mode (whitespace/comments ignored) will fail to match correctly.
* **Blast radius**: Conformance failures for regular expressions using the `x` flag.
* **Mitigation**: Implement support for the `x` flag by mapping it to the `regex` builder's appropriate configuration (or stripping whitespace/comments manually if the crate doesn't support it directly).

---

## 5. Verification Method
1. Run the test suite:
   ```bash
   cargo test --workspace --test shacl_validation --test shacl_conformance
   ```
2. Inspect the newly added tests in `lib/tests/shacl_validation.rs` covering:
   - `test_empty_dataset` (empty data behavior)
   - `test_invalid_turtle_shapes` (syntax error propagation)
   - `test_recursive_shapes` (recursive resolution and cycle safety)
   - `test_property_paths_comprehensive` (alternative paths, sequence paths, zero-or-more paths, one-or-more paths, zero-or-one paths, simple inverse paths)
   - `test_severity_and_datatype` (severity field propagation)
3. To reproduce the broken isomorphism check, write a test case in `shacl_conformance` that expects a non-conforming result report and compare using `graphs_isomorphic`.
