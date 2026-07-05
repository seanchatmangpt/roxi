// Runner for a vendored subset of the real W3C `data-shapes` SHACL test suite
// (core constraint components), fetched from:
//   https://github.com/w3c/data-shapes/tree/gh-pages/data-shapes-test-suite/tests/core
//
// Each vendored file is either:
//   - self-contained: the shapes graph, data graph, and expected
//     `sh:ValidationReport` (embedded inside an `mf:Manifest` /
//     `sht:Validate` test-case description) are all the same Turtle
//     document (this is what the manifest's `sht:dataGraph <>` /
//     `sht:shapesGraph <>` self-reference means), or
//   - split: the manifest/expected-report file references separate
//     `...-data.ttl` / `...-shapes.ttl` files (only `node/qualified-001.ttl`
//     in this vendored subset).
//
// Rather than fully parsing the W3C manifest ontology (mf:/sht:) to extract
// expected results, this runner does lightweight text scanning of the
// vendored file's embedded `mf:result [ a sh:ValidationReport ; ... ]` block:
// it reads off the expected `sh:conforms` boolean and counts the number of
// embedded `sh:ValidationResult` stanzas (== expected violation count, since
// every case in this vendored subset uses the default severity,
// sh:Violation). This is a deliberately minimal approach -- see the doc
// comment on `expected_from_manifest` -- but is sufficient to validate real
// conformance against the vendored suite's official expected results.

use std::fs;
use std::path::{Path, PathBuf};

use minimal::parser::{Parser, Syntax};
use minimal::shacl::{ShapesGraph, Validator};
use minimal::tripleindex::TripleIndex;

struct Case {
    /// Human-readable test id, matches the vendored file layout.
    name: &'static str,
    /// File containing the embedded manifest / expected sh:ValidationReport.
    meta: &'static str,
    /// File to load as the data graph (often == `meta`).
    data: &'static str,
    /// File to load as the shapes graph (often == `meta`).
    shapes: &'static str,
}

/// The vendored subset, expanded from an initial 23-case pass to add real
/// depth in `core/property` (previously the most under-represented category
/// at 4/39 real cases) plus additional `core/node` and a new `core/misc`
/// category (severity/deactivated/message handling). `core/path` (complex
/// SPARQL-property-path `sh:path` expressions: sequence/inverse/alternative/
/// zeroOrMore/etc.) is deliberately NOT vendored here: `shacl.rs` only
/// supports simple single-predicate `sh:path`, so those cases would either
/// silently no-op or need real property-path evaluation to be implemented
/// first -- an honest, documented gap rather than vendoring files that would
/// sit unwired (see docs/jira/26.7.4/manifests/shacl_manifest.md).
const CASES: &[Case] = &[
    Case { name: "node/and-001", meta: "node/and-001.ttl", data: "node/and-001.ttl", shapes: "node/and-001.ttl" },
    Case { name: "node/and-002", meta: "node/and-002.ttl", data: "node/and-002.ttl", shapes: "node/and-002.ttl" },
    Case { name: "node/class-001", meta: "node/class-001.ttl", data: "node/class-001.ttl", shapes: "node/class-001.ttl" },
    Case { name: "node/class-002", meta: "node/class-002.ttl", data: "node/class-002.ttl", shapes: "node/class-002.ttl" },
    Case { name: "node/class-003", meta: "node/class-003.ttl", data: "node/class-003.ttl", shapes: "node/class-003.ttl" },
    Case { name: "node/closed-001", meta: "node/closed-001.ttl", data: "node/closed-001.ttl", shapes: "node/closed-001.ttl" },
    Case { name: "node/closed-002", meta: "node/closed-002.ttl", data: "node/closed-002.ttl", shapes: "node/closed-002.ttl" },
    Case { name: "node/datatype-001", meta: "node/datatype-001.ttl", data: "node/datatype-001.ttl", shapes: "node/datatype-001.ttl" },
    Case { name: "node/datatype-002", meta: "node/datatype-002.ttl", data: "node/datatype-002.ttl", shapes: "node/datatype-002.ttl" },
    Case { name: "node/disjoint-001", meta: "node/disjoint-001.ttl", data: "node/disjoint-001.ttl", shapes: "node/disjoint-001.ttl" },
    Case { name: "node/equals-001", meta: "node/equals-001.ttl", data: "node/equals-001.ttl", shapes: "node/equals-001.ttl" },
    Case { name: "node/hasValue-001", meta: "node/hasValue-001.ttl", data: "node/hasValue-001.ttl", shapes: "node/hasValue-001.ttl" },
    Case { name: "node/in-001", meta: "node/in-001.ttl", data: "node/in-001.ttl", shapes: "node/in-001.ttl" },
    Case { name: "node/languageIn-001", meta: "node/languageIn-001.ttl", data: "node/languageIn-001.ttl", shapes: "node/languageIn-001.ttl" },
    Case { name: "node/maxLength-001", meta: "node/maxLength-001.ttl", data: "node/maxLength-001.ttl", shapes: "node/maxLength-001.ttl" },
    Case { name: "node/minExclusive-001", meta: "node/minExclusive-001.ttl", data: "node/minExclusive-001.ttl", shapes: "node/minExclusive-001.ttl" },
    Case { name: "node/minInclusive-001", meta: "node/minInclusive-001.ttl", data: "node/minInclusive-001.ttl", shapes: "node/minInclusive-001.ttl" },
    Case { name: "node/minInclusive-002", meta: "node/minInclusive-002.ttl", data: "node/minInclusive-002.ttl", shapes: "node/minInclusive-002.ttl" },
    Case { name: "node/minInclusive-003", meta: "node/minInclusive-003.ttl", data: "node/minInclusive-003.ttl", shapes: "node/minInclusive-003.ttl" },
    Case { name: "node/minLength-001", meta: "node/minLength-001.ttl", data: "node/minLength-001.ttl", shapes: "node/minLength-001.ttl" },
    Case { name: "node/node-001", meta: "node/node-001.ttl", data: "node/node-001.ttl", shapes: "node/node-001.ttl" },
    Case { name: "node/nodeKind-001", meta: "node/nodeKind-001.ttl", data: "node/nodeKind-001.ttl", shapes: "node/nodeKind-001.ttl" },
    Case { name: "node/not-001", meta: "node/not-001.ttl", data: "node/not-001.ttl", shapes: "node/not-001.ttl" },
    Case { name: "node/not-002", meta: "node/not-002.ttl", data: "node/not-002.ttl", shapes: "node/not-002.ttl" },
    Case { name: "node/or-001", meta: "node/or-001.ttl", data: "node/or-001.ttl", shapes: "node/or-001.ttl" },
    Case { name: "node/pattern-001", meta: "node/pattern-001.ttl", data: "node/pattern-001.ttl", shapes: "node/pattern-001.ttl" },
    Case { name: "node/pattern-002", meta: "node/pattern-002.ttl", data: "node/pattern-002.ttl", shapes: "node/pattern-002.ttl" },
    Case { name: "node/xone-001", meta: "node/xone-001.ttl", data: "node/xone-001.ttl", shapes: "node/xone-001.ttl" },
    Case { name: "node/qualified-001", meta: "node/qualified-001.ttl", data: "node/qualified-001-data.ttl", shapes: "node/qualified-001-shapes.ttl" },
    Case { name: "property/datatype-001", meta: "property/datatype-001.ttl", data: "property/datatype-001.ttl", shapes: "property/datatype-001.ttl" },
    Case { name: "property/datatype-002", meta: "property/datatype-002.ttl", data: "property/datatype-002.ttl", shapes: "property/datatype-002.ttl" },
    Case { name: "property/datatype-003", meta: "property/datatype-003.ttl", data: "property/datatype-003.ttl", shapes: "property/datatype-003.ttl" },
    Case { name: "property/disjoint-001", meta: "property/disjoint-001.ttl", data: "property/disjoint-001.ttl", shapes: "property/disjoint-001.ttl" },
    Case { name: "property/equals-001", meta: "property/equals-001.ttl", data: "property/equals-001.ttl", shapes: "property/equals-001.ttl" },
    Case { name: "property/hasValue-001", meta: "property/hasValue-001.ttl", data: "property/hasValue-001.ttl", shapes: "property/hasValue-001.ttl" },
    Case { name: "property/in-001", meta: "property/in-001.ttl", data: "property/in-001.ttl", shapes: "property/in-001.ttl" },
    Case { name: "property/minCount-001", meta: "property/minCount-001.ttl", data: "property/minCount-001.ttl", shapes: "property/minCount-001.ttl" },
    Case { name: "property/minCount-002", meta: "property/minCount-002.ttl", data: "property/minCount-002.ttl", shapes: "property/minCount-002.ttl" },
    Case { name: "property/maxCount-001", meta: "property/maxCount-001.ttl", data: "property/maxCount-001.ttl", shapes: "property/maxCount-001.ttl" },
    Case { name: "property/maxCount-002", meta: "property/maxCount-002.ttl", data: "property/maxCount-002.ttl", shapes: "property/maxCount-002.ttl" },
    Case { name: "property/maxExclusive-001", meta: "property/maxExclusive-001.ttl", data: "property/maxExclusive-001.ttl", shapes: "property/maxExclusive-001.ttl" },
    Case { name: "property/maxInclusive-001", meta: "property/maxInclusive-001.ttl", data: "property/maxInclusive-001.ttl", shapes: "property/maxInclusive-001.ttl" },
    Case { name: "property/minExclusive-002", meta: "property/minExclusive-002.ttl", data: "property/minExclusive-002.ttl", shapes: "property/minExclusive-002.ttl" },
    Case { name: "property/minLength-001", meta: "property/minLength-001.ttl", data: "property/minLength-001.ttl", shapes: "property/minLength-001.ttl" },
    Case { name: "property/node-001", meta: "property/node-001.ttl", data: "property/node-001.ttl", shapes: "property/node-001.ttl" },
    Case { name: "property/not-001", meta: "property/not-001.ttl", data: "property/not-001.ttl", shapes: "property/not-001.ttl" },
    Case { name: "property/or-001", meta: "property/or-001.ttl", data: "property/or-001.ttl", shapes: "property/or-001.ttl" },
    Case { name: "property/pattern-001", meta: "property/pattern-001.ttl", data: "property/pattern-001.ttl", shapes: "property/pattern-001.ttl" },
    Case { name: "property/property-001", meta: "property/property-001.ttl", data: "property/property-001.ttl", shapes: "property/property-001.ttl" },
    Case { name: "property/lessThan-001", meta: "property/lessThan-001.ttl", data: "property/lessThan-001.ttl", shapes: "property/lessThan-001.ttl" },
    Case { name: "property/lessThan-002", meta: "property/lessThan-002.ttl", data: "property/lessThan-002.ttl", shapes: "property/lessThan-002.ttl" },
    Case { name: "property/lessThanOrEquals-001", meta: "property/lessThanOrEquals-001.ttl", data: "property/lessThanOrEquals-001.ttl", shapes: "property/lessThanOrEquals-001.ttl" },
    Case { name: "property/uniqueLang-001", meta: "property/uniqueLang-001.ttl", data: "property/uniqueLang-001.ttl", shapes: "property/uniqueLang-001.ttl" },
    Case { name: "targets/targetClass-001", meta: "targets/targetClass-001.ttl", data: "targets/targetClass-001.ttl", shapes: "targets/targetClass-001.ttl" },
    Case { name: "targets/targetNode-001", meta: "targets/targetNode-001.ttl", data: "targets/targetNode-001.ttl", shapes: "targets/targetNode-001.ttl" },
    Case { name: "misc/deactivated-001", meta: "misc/deactivated-001.ttl", data: "misc/deactivated-001.ttl", shapes: "misc/deactivated-001.ttl" },
    Case { name: "misc/deactivated-002", meta: "misc/deactivated-002.ttl", data: "misc/deactivated-002.ttl", shapes: "misc/deactivated-002.ttl" },
    Case { name: "misc/message-001", meta: "misc/message-001.ttl", data: "misc/message-001.ttl", shapes: "misc/message-001.ttl" },
    Case { name: "misc/severity-001", meta: "misc/severity-001.ttl", data: "misc/severity-001.ttl", shapes: "misc/severity-001.ttl" },
    Case { name: "misc/severity-002", meta: "misc/severity-002.ttl", data: "misc/severity-002.ttl", shapes: "misc/severity-002.ttl" },
];

fn suite_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/shacl_conformance/w3c_suite")
}

/// The vendored manifest files use relative IRIs -- the same-document
/// self-reference `<>` (per `sht:dataGraph <>` / `sht:shapesGraph <>`) and
/// per-test-case fragment-like refs such as `<and-001>` (used as the subject
/// of the `sht:Validate` description and inside `mf:entries (...)`) -- to
/// refer to parts of the manifest document itself. Our Turtle parser is
/// invoked without a base IRI (see `Parser::parse_triples`), so bare relative
/// IRIs fail to resolve ("No scheme found in an absolute IRI").
///
/// Since these manifest/test-case triples are irrelevant to SHACL validation
/// (they're never a shape's subject/predicate/object in a way our validator
/// interprets) and every genuinely absolute IRI in these files has a
/// `scheme:` prefix (even the deliberately unusual `<a:b>` in
/// `maxLength-001.ttl`), rewriting any bracketed term with no `:` into a
/// fixed placeholder absolute IRI is a safe, meaning-preserving fix-up here.
fn resolve_relative_self_refs(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut rest = content;
    while let Some(start) = rest.find('<') {
        let (before, after_lt) = rest.split_at(start);
        out.push_str(before);
        let after_lt = &after_lt[1..]; // skip '<'
        if let Some(end) = after_lt.find('>') {
            let inner = &after_lt[..end];
            if inner.contains(':') {
                // Looks like an absolute IRI (has a scheme) -- leave as-is.
                out.push('<');
                out.push_str(inner);
                out.push('>');
            } else {
                out.push_str("<http://example.org/w3c-shacl-test-doc");
                if !inner.is_empty() {
                    out.push('#');
                    out.push_str(inner);
                }
                out.push('>');
            }
            rest = &after_lt[end + 1..];
        } else {
            // Unterminated '<' (shouldn't happen in well-formed Turtle) -- copy verbatim.
            out.push('<');
            rest = after_lt;
        }
    }
    out.push_str(rest);
    out
}

fn build_index(content: &str, source: &str) -> TripleIndex {
    let resolved = resolve_relative_self_refs(content);
    let triples = Parser::parse_triples(&resolved, Syntax::Turtle)
        .unwrap_or_else(|e| panic!("failed to parse vendored file {source}: {e}"));
    let mut index = TripleIndex::new();
    for t in triples {
        index.add(t);
    }
    index
}

/// Extract `(expected_conforms, expected_violation_count)` from the embedded
/// W3C manifest test-case description via lightweight text scanning rather
/// than a full mf:/sht: manifest-ontology parse. This works because every
/// vendored case's `mf:result [ a sh:ValidationReport ; sh:conforms "..." ;
/// sh:result [ a sh:ValidationResult ; ... ] ... ]` block is textually
/// regular: exactly one `sh:conforms "true|false"` literal, and one
/// `rdf:type sh:ValidationResult` triple per expected result. All cases in
/// this vendored subset use the default severity (sh:Violation), so the
/// raw stanza count is directly comparable to `report.results.len()`.
fn expected_from_manifest(content: &str) -> (bool, usize) {
    let conforms = content.contains("sh:conforms \"true\"");
    debug_assert!(
        conforms || content.contains("sh:conforms \"false\""),
        "vendored file has no recognizable sh:conforms literal"
    );
    let violation_count = content.matches("rdf:type sh:ValidationResult").count()
        + content.matches("a sh:ValidationResult").count();
    (conforms, violation_count)
}

struct CaseOutcome {
    name: &'static str,
    passed: bool,
    detail: String,
}

#[test]
fn test_w3c_core_constraint_component_suite() {
    let dir = suite_dir();
    let mut outcomes = Vec::with_capacity(CASES.len());

    for case in CASES {
        let meta_path = dir.join(case.meta);
        let meta_content = fs::read_to_string(&meta_path)
            .unwrap_or_else(|e| panic!("missing vendored file {}: {e}", meta_path.display()));

        let data_content = if case.data == case.meta {
            meta_content.clone()
        } else {
            fs::read_to_string(dir.join(case.data)).unwrap_or_else(|e| panic!("missing {}: {e}", case.data))
        };
        let shapes_content = if case.shapes == case.meta {
            meta_content.clone()
        } else if case.shapes == case.data {
            data_content.clone()
        } else {
            fs::read_to_string(dir.join(case.shapes)).unwrap_or_else(|e| panic!("missing {}: {e}", case.shapes))
        };

        let (expected_conforms, expected_violations) = expected_from_manifest(&meta_content);

        let data_index = build_index(&data_content, case.data);
        let shapes = ShapesGraph::parse(&resolve_relative_self_refs(&shapes_content))
            .unwrap_or_else(|e| panic!("failed to parse vendored shapes {}: {e}", case.shapes));
        let report = Validator::validate(&data_index, &shapes);

        let actual_violations = report.results.len();
        let passed = report.conforms == expected_conforms && actual_violations == expected_violations;
        outcomes.push(CaseOutcome {
            name: case.name,
            passed,
            detail: format!(
                "expected conforms={expected_conforms} violations={expected_violations}, got conforms={} violations={actual_violations}",
                report.conforms
            ),
        });
    }

    let total = outcomes.len();
    let passed = outcomes.iter().filter(|o| o.passed).count();
    let failed = outcomes.iter().filter(|o| !o.passed).collect::<Vec<_>>();

    write_manifest_report(total, passed, &outcomes);

    assert!(
        failed.is_empty(),
        "W3C SHACL core-constraint-component suite failures ({}/{} failed):\n{}",
        failed.len(),
        total,
        failed.iter().map(|o| format!("  {}: {}", o.name, o.detail)).collect::<Vec<_>>().join("\n")
    );
}

fn write_manifest_report(total: usize, passed: usize, outcomes: &[CaseOutcome]) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/jira/26.7.4/manifests");
    fs::create_dir_all(&manifest_dir).expect("Failed to create manifests directory");

    let pass_rate = if total == 0 { 0.0 } else { (passed as f64) * 100.0 / (total as f64) };
    let failed = total - passed;

    let mut report = String::new();
    report.push_str("# SHACL Conformance Pass-Rate Manifest\n\n");
    report.push_str("- **Dialect**: SHACL\n");
    report.push_str("- **Suite**: W3C data-shapes Test Suite (vendored core-constraint-component subset: node/, property/, targets/, misc/)\n");
    report.push_str("- **Source**: https://github.com/w3c/data-shapes/tree/gh-pages/data-shapes-test-suite/tests/core\n");
    report.push_str("- **Coverage note**: `core/path/` (complex SPARQL-property-path `sh:path` expressions -- sequence/inverse/alternative/zeroOrMore/etc.) is NOT vendored: `shacl.rs` only supports simple single-predicate `sh:path`, so those ~17 real cases are an honest, documented gap, not silently dropped.\n");
    report.push_str(&format!("- **Total Tests**: {total}\n"));
    report.push_str(&format!("- **Passed**: {passed}\n"));
    report.push_str(&format!("- **Failed**: {failed}\n"));
    report.push_str(&format!("- **Pass Rate**: {pass_rate:.1}%\n\n"));
    report.push_str("## Test Details\n\n");
    report.push_str("| Test Name | Status | Details |\n");
    report.push_str("|---|---|---|\n");
    for o in outcomes {
        let status = if o.passed { "PASSED" } else { "FAILED" };
        report.push_str(&format!("| {} | {} | {} |\n", o.name, status, o.detail));
    }

    fs::write(manifest_dir.join("shacl_manifest.md"), report).expect("Failed to write SHACL manifest report");
}
