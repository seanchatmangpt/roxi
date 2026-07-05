//! Manifest-driven N3 conformance runner.
//!
//! Replaces the previous hardcoded `passed = true` stub: this reads
//! `manifest.json`, and for every `active` test case loads its `input` N3
//! document, materializes it, and checks that every triple in its `expected`
//! file is present among the derived facts. See README.md for the full
//! design (including why this is a subset check rather than exact equality).

use minimal::TripleStore;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Manifest {
    conformance_suites: Vec<Suite>,
}

#[derive(Debug, Deserialize)]
struct Suite {
    suite_name: String,
    test_cases: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
struct TestCase {
    id: String,
    name: String,
    input: String,
    expected: String,
    status: String,
    /// Present (and required) on any case whose `status` isn't `"active"`:
    /// a precise, specific explanation of exactly what this engine can't do
    /// yet that blocks the case, rather than a generic "not supported".
    #[serde(default)]
    blocked_reason: Option<String>,
}

struct CaseResult {
    suite_name: String,
    id: String,
    name: String,
    passed: bool,
    detail: String,
}

/// Parse a plain N3 fact file (no rules expected) and return each of its
/// triples decoded to its canonical "S P O" string form.
fn decoded_fact_set(base_dir: &Path, relative_path: &str) -> HashSet<String> {
    let content = fs::read_to_string(base_dir.join(relative_path))
        .unwrap_or_else(|e| panic!("failed to read {}: {}", relative_path, e));
    let store = TripleStore::from(&content);
    store
        .triple_index
        .triples
        .iter()
        .map(TripleStore::decode_triple)
        .collect()
}

fn run_case(base_dir: &Path, suite_name: &str, case: &TestCase) -> CaseResult {
    let input_content = fs::read_to_string(base_dir.join(&case.input))
        .unwrap_or_else(|e| panic!("failed to read {}: {}", case.input, e));

    let mut store = TripleStore::from(&input_content);
    let inferred = store.materialize();
    let derived: HashSet<String> = inferred.iter().map(TripleStore::decode_triple).collect();

    let expected = decoded_fact_set(base_dir, &case.expected);

    let missing: Vec<&String> = expected.iter().filter(|t| !derived.contains(*t)).collect();

    if missing.is_empty() {
        CaseResult {
            suite_name: suite_name.to_string(),
            id: case.id.clone(),
            name: case.name.clone(),
            passed: true,
            detail: format!("all {} expected triple(s) derived", expected.len()),
        }
    } else {
        CaseResult {
            suite_name: suite_name.to_string(),
            id: case.id.clone(),
            name: case.name.clone(),
            passed: false,
            detail: format!(
                "missing {} of {} expected triple(s): {:?}; derived {} triple(s): {:?}",
                missing.len(),
                expected.len(),
                missing,
                derived.len(),
                derived
            ),
        }
    }
}

#[test]
fn test_n3_conformance() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let base_dir = manifest_dir.join("tests/n3_conformance");

    let manifest_path = base_dir.join("manifest.json");
    let manifest_content = fs::read_to_string(&manifest_path).expect("Failed to read manifest.json");
    let manifest: Manifest = serde_json::from_str(&manifest_content).expect("Failed to parse manifest.json");

    // Dynamic-discovery guard (mirrors datalog_conformance.rs's directory-scan
    // pattern): every `X.n3` in syntax/builtins/vendored that has a matching
    // `X_result.n3` sibling must be referenced by *some* manifest case's
    // `input`, active or not. This means a newly vendored case can never
    // silently sit unrun -- forgetting to add its manifest entry (with either
    // `"status": "active"` or `"status": "ignored"` + `blocked_reason`) fails
    // the suite immediately instead of the file just never being picked up.
    let referenced_inputs: HashSet<String> = manifest
        .conformance_suites
        .iter()
        .flat_map(|s| s.test_cases.iter().map(|c| c.input.clone()))
        .collect();
    let mut undiscovered: Vec<String> = Vec::new();
    for dir in ["syntax", "builtins", "vendored"] {
        let dir_path = base_dir.join(dir);
        let Ok(entries) = fs::read_dir(&dir_path) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("n3") {
                continue;
            }
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            if stem.ends_with("_result") {
                continue;
            }
            let result_path = path.with_file_name(format!("{}_result.n3", stem));
            if !result_path.exists() {
                // No `_result.n3` sibling: not a runnable case (e.g. a
                // documentation-only stub for an out-of-scope corpus entry
                // like edt.n3/basic_monadic.n3/skos.n3/filter.n3).
                continue;
            }
            let relative = format!("{}/{}.n3", dir, stem);
            if !referenced_inputs.contains(&relative) {
                undiscovered.push(relative);
            }
        }
    }
    assert!(
        undiscovered.is_empty(),
        "found vendored/syntax/builtins .n3 case(s) with a _result.n3 sibling that aren't \
         referenced by any manifest.json test case (add one, with status \"active\" or \
         \"ignored\" + blocked_reason): {:?}",
        undiscovered
    );

    let mut results: Vec<CaseResult> = Vec::new();
    let mut ignored: Vec<(String, String, String)> = Vec::new(); // (id, name, reason)
    for suite in &manifest.conformance_suites {
        for case in &suite.test_cases {
            if case.status != "active" {
                let reason = case
                    .blocked_reason
                    .clone()
                    .unwrap_or_else(|| "(no blocked_reason given in manifest.json)".to_string());
                ignored.push((case.id.clone(), case.name.clone(), reason));
                continue;
            }
            results.push(run_case(&base_dir, &suite.suite_name, case));
        }
    }

    println!("-- {} ignored/blocked case(s) --", ignored.len());
    for (id, name, reason) in &ignored {
        println!("[IGNORED] {} ({}): {}", id, name, reason);
    }

    assert!(!results.is_empty(), "no active N3 conformance cases were found in manifest.json");

    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    for r in &results {
        println!(
            "[{}] {} ({}): {} -- {}",
            if r.passed { "PASS" } else { "FAIL" },
            r.id,
            r.suite_name,
            r.name,
            r.detail
        );
    }

    write_manifest_report(manifest_dir, &results, total, passed, failed);

    assert_eq!(
        failed, 0,
        "{} of {} N3 conformance case(s) failed -- see stdout above for details",
        failed, total
    );
}

fn write_manifest_report(
    manifest_dir: &Path,
    results: &[CaseResult],
    total: usize,
    passed: usize,
    failed: usize,
) {
    let manifests_dir: PathBuf = manifest_dir.join("../docs/jira/26.7.4/manifests");
    fs::create_dir_all(&manifests_dir).expect("Failed to create manifests directory");

    let pass_rate = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 };

    let mut rows = String::new();
    for r in results {
        rows.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            r.id,
            r.suite_name,
            if r.passed { "PASSED" } else { "FAILED" },
            r.detail.replace('|', "\\|"),
        ));
    }

    let manifest_report = format!(
        "# N3 Conformance Pass-Rate Manifest\n\n\
        - **Dialect**: Notation3 (N3)\n\
        - **Suite**: hand-authored syntax/builtins cases plus two vendored EYE reasoner cases (see lib/tests/n3_conformance/README.md)\n\
        - **Total Tests**: {total}\n\
        - **Passed**: {passed}\n\
        - **Failed**: {failed}\n\
        - **Pass Rate**: {pass_rate:.1}%\n\n\
        ## Test Details\n\n\
        | Test ID | Suite | Status | Details |\n\
        |---|---|---|---|\n\
        {rows}"
    );

    fs::write(manifests_dir.join("n3_manifest.md"), manifest_report)
        .expect("Failed to write N3 manifest report");
}
