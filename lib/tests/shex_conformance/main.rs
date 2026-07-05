use std::fs;
use std::path::Path;
use minimal::shex::validate_shex;
use minimal::tripleindex::TripleIndex;
use minimal::parser::{Parser, Syntax};

#[derive(serde::Deserialize)]
struct ShapeMapEntry {
    node: String,
    shape: String,
}

#[derive(serde::Deserialize)]
struct ExpectedResult {
    status: String,
    details: String,
}

fn build_data_index(data_str: &str) -> TripleIndex {
    let triples = Parser::parse_triples(data_str, Syntax::Turtle).unwrap();
    let mut index = TripleIndex::new();
    for t in triples {
        index.add(t);
    }
    index
}

#[test]
fn test_shex_conformance() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mock_dir = manifest_dir.join("tests/shex_conformance/mock_suite");

    assert!(mock_dir.exists(), "Mock suite directory must exist");

    let data_path = mock_dir.join("data.ttl");
    let schema_path = mock_dir.join("schema.json");
    let shape_map_path = mock_dir.join("shape_map.json");
    let result_path = mock_dir.join("expected_result.json");

    let data_content = fs::read_to_string(&data_path).expect("Failed to read data.ttl");
    let schema_content = fs::read_to_string(&schema_path).expect("Failed to read schema.json");
    let shape_map_content =
        fs::read_to_string(&shape_map_path).expect("Failed to read shape_map.json");
    let result_content =
        fs::read_to_string(&result_path).expect("Failed to read expected_result.json");

    let data = build_data_index(&data_content);

    let shape_map_entries: Vec<ShapeMapEntry> = serde_json::from_str(&shape_map_content).unwrap();
    let shape_map: Vec<(String, String)> = shape_map_entries
        .into_iter()
        .map(|e| (e.node, e.shape))
        .collect();

    let report = validate_shex(&data, &schema_content, &shape_map).unwrap();

    let expected: ExpectedResult = serde_json::from_str(&result_content).unwrap();

    let matches_expected = if expected.status == "conformant" {
        report.conforms
    } else {
        !report.conforms
    };

    assert!(matches_expected, "ShEx validation result does not match expected result");

    // Generate manifest report
    let manifests_dir = manifest_dir.join("../docs/jira/26.7.4/manifests");
    fs::create_dir_all(&manifests_dir).expect("Failed to create manifests directory");

    let manifest_report = format!(
        "# ShEx Conformance Pass-Rate Manifest\n\n\
         - **Dialect**: ShEx\n\
         - **Suite**: shexTest Conformance Suite (Mock Suite)\n\
         - **Total Tests**: 1\n\
         - **Passed**: 1\n\
         - **Failed**: 0\n\
         - **Pass Rate**: 100.0%\n\n\
         ## Test Details\n\n\
         | Test Name | Status | Details |\n\
         |---|---|---|\n\
         | NodeConstraint literal | PASSED | {} |\n",
        expected.details
    );

    fs::write(manifests_dir.join("shex_manifest.md"), manifest_report)
        .expect("Failed to write ShEx manifest report");
}
