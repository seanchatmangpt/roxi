use minimal::shex::validate_shex;
use minimal::tripleindex::TripleIndex;
use minimal::parser::{Parser, Syntax};

fn build_data_index(data_str: &str) -> TripleIndex {
    let triples = Parser::parse_triples(data_str, Syntax::Turtle).unwrap();
    let mut index = TripleIndex::new();
    for t in triples {
        index.add(t);
    }
    index
}

#[test]
fn test_node_constraint_datatype() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/AgeShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/age",
              "valueExpr": {
                "type": "NodeConstraint",
                "datatype": "http://www.w3.org/2001/XMLSchema#integer"
              }
            }
          }
        }
      ]
    }"#;

    let data_str = r#"
        <http://example.org/Alice> <http://example.org/age> 30 .
        <http://example.org/Bob> <http://example.org/age> "thirty" .
    "#;

    let data = build_data_index(data_str);
    let shape_map = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/AgeShape".to_string()),
        ("http://example.org/Bob".to_string(), "http://example.org/AgeShape".to_string()),
    ];

    let report = validate_shex(&data, schema_json, &shape_map).unwrap();

    assert!(!report.conforms);
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].node.to_string(), "<http://example.org/Bob>");
    assert_eq!(report.failures[0].shape, "http://example.org/AgeShape");
}

#[test]
fn test_each_of_shape() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/UserShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "EachOf",
              "expressions": [
                {
                  "type": "TripleConstraint",
                  "predicate": "http://example.org/name",
                  "valueExpr": {
                    "type": "NodeConstraint",
                    "nodeKind": "literal"
                  }
                },
                {
                  "type": "TripleConstraint",
                  "predicate": "http://example.org/age",
                  "valueExpr": {
                    "type": "NodeConstraint",
                    "datatype": "http://www.w3.org/2001/XMLSchema#integer"
                  }
                }
              ]
            }
          }
        }
      ]
    }"#;

    let data_str = r#"
        <http://example.org/Alice> <http://example.org/name> "Alice" ;
                                   <http://example.org/age> 30 .
        <http://example.org/Bob> <http://example.org/name> "Bob" .
    "#;

    let data = build_data_index(data_str);
    let shape_map = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/UserShape".to_string()),
        ("http://example.org/Bob".to_string(), "http://example.org/UserShape".to_string()),
    ];

    let report = validate_shex(&data, schema_json, &shape_map).unwrap();

    assert!(!report.conforms);
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].node.to_string(), "<http://example.org/Bob>");
    assert_eq!(report.failures[0].shape, "http://example.org/UserShape");
}

#[test]
fn test_cardinality_on_triple_constraint() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/PhoneShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/phone",
              "min": 1,
              "max": 2,
              "valueExpr": {
                "type": "NodeConstraint",
                "nodeKind": "literal"
              }
            }
          }
        }
      ]
    }"#;

    let data_str = r#"
        <http://example.org/Alice> <http://example.org/phone> "111", "222" .
        <http://example.org/Bob> <http://example.org/phone> "111", "222", "333" .
        <http://example.org/Charlie> <http://example.org/name> "Charlie" .
    "#;

    let data = build_data_index(data_str);
    let shape_map = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/PhoneShape".to_string()),
        ("http://example.org/Bob".to_string(), "http://example.org/PhoneShape".to_string()),
        ("http://example.org/Charlie".to_string(), "http://example.org/PhoneShape".to_string()),
    ];

    let report = validate_shex(&data, schema_json, &shape_map).unwrap();

    assert!(!report.conforms);
    assert_eq!(report.failures.len(), 2);

    let failed_nodes: Vec<String> = report.failures.iter().map(|f| f.node.to_string()).collect();
    assert!(failed_nodes.contains(&"<http://example.org/Bob>".to_string()));
    assert!(failed_nodes.contains(&"<http://example.org/Charlie>".to_string()));
}

#[test]
fn test_shape_ref_recursive() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/PersonShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/knows",
              "valueExpr": "http://example.org/PersonShape"
            }
          }
        }
      ]
    }"#;

    let data_str = r#"
        <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> .
        <http://example.org/Bob> <http://example.org/knows> <http://example.org/Alice> .
    "#;

    let data = build_data_index(data_str);
    let shape_map = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/PersonShape".to_string()),
        ("http://example.org/Bob".to_string(), "http://example.org/PersonShape".to_string()),
    ];

    let report = validate_shex(&data, schema_json, &shape_map).unwrap();

    assert!(report.conforms);
    assert_eq!(report.failures.len(), 0);
}

#[test]
fn test_shape_map_pass_fail() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/AgeShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/age",
              "valueExpr": {
                "type": "NodeConstraint",
                "datatype": "http://www.w3.org/2001/XMLSchema#integer"
              }
            }
          }
        }
      ]
    }"#;

    let data_str = r#"
        <http://example.org/Alice> <http://example.org/age> 30 .
        <http://example.org/Bob> <http://example.org/age> "thirty" .
    "#;

    let data = build_data_index(data_str);
    
    // Check clean pass
    let shape_map_pass = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/AgeShape".to_string()),
    ];
    let report_pass = validate_shex(&data, schema_json, &shape_map_pass).unwrap();
    assert!(report_pass.conforms);

    // Check fail
    let shape_map_fail = vec![
        ("http://example.org/Bob".to_string(), "http://example.org/AgeShape".to_string()),
    ];
    let report_fail = validate_shex(&data, schema_json, &shape_map_fail).unwrap();
    assert!(!report_fail.conforms);
}

#[test]
fn test_empty_and_invalid_schema() {
    let data = build_data_index("");
    let shape_map = vec![("http://example.org/Alice".to_string(), "http://example.org/AgeShape".to_string())];
    
    // Empty schema should fail parsing
    let res = validate_shex(&data, "", &shape_map);
    assert!(res.is_err());

    // Invalid JSON schema should fail parsing
    let res_invalid = validate_shex(&data, "invalid json", &shape_map);
    assert!(res_invalid.is_err());
}

#[test]
fn test_empty_shape_map() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/AgeShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/age",
              "valueExpr": {
                "type": "NodeConstraint",
                "datatype": "http://www.w3.org/2001/XMLSchema#integer"
              }
            }
          }
        }
      ]
    }"#;
    let data = build_data_index("<http://example.org/Alice> <http://example.org/age> 30 .");
    
    // Empty shape map: should conform since nothing is validated
    let report = validate_shex(&data, schema_json, &[]).unwrap();
    assert!(report.conforms);
    assert!(report.failures.is_empty());
}

#[test]
fn test_empty_graph() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/AgeShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/age",
              "valueExpr": {
                "type": "NodeConstraint",
                "datatype": "http://www.w3.org/2001/XMLSchema#integer"
              }
            }
          }
        }
      ]
    }"#;
    let data = build_data_index(""); // empty graph
    let shape_map = vec![("http://example.org/Alice".to_string(), "http://example.org/AgeShape".to_string())];
    
    // Empty graph: should NOT conform, because the required age property is missing
    let report = validate_shex(&data, schema_json, &shape_map).unwrap();
    assert!(!report.conforms);
    assert_eq!(report.failures.len(), 1);
}

#[test]
fn test_extremely_long_string_datatype() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/LongStringShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/text",
              "valueExpr": {
                "type": "NodeConstraint",
                "nodeKind": "literal"
              }
            }
          }
        }
      ]
    }"#;
    
    let long_str = "a".repeat(100_000);
    let data_str = format!(r#"<http://example.org/Alice> <http://example.org/text> "{}" ."#, long_str);
    let data = build_data_index(&data_str);
    let shape_map = vec![("http://example.org/Alice".to_string(), "http://example.org/LongStringShape".to_string())];
    
    let report = validate_shex(&data, schema_json, &shape_map).unwrap();
    assert!(report.conforms);
    assert_eq!(report.failures.len(), 0);
}

#[test]
fn test_nested_recursive_references_stress() {
    // Nested recursion: A refers to B, B refers to C, C refers to A
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/AShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/toB",
              "valueExpr": "http://example.org/BShape"
            }
          }
        },
        {
          "type": "ShapeDecl",
          "id": "http://example.org/BShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/toC",
              "valueExpr": "http://example.org/CShape"
            }
          }
        },
        {
          "type": "ShapeDecl",
          "id": "http://example.org/CShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/toA",
              "valueExpr": "http://example.org/AShape"
            }
          }
        }
      ]
    }"#;

    // Conformant cycle: Alice -> Bob -> Charlie -> Alice
    let data_str = r#"
        <http://example.org/Alice> <http://example.org/toB> <http://example.org/Bob> .
        <http://example.org/Bob> <http://example.org/toC> <http://example.org/Charlie> .
        <http://example.org/Charlie> <http://example.org/toA> <http://example.org/Alice> .
    "#;
    let data = build_data_index(data_str);
    let shape_map = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/AShape".to_string()),
        ("http://example.org/Bob".to_string(), "http://example.org/BShape".to_string()),
        ("http://example.org/Charlie".to_string(), "http://example.org/CShape".to_string()),
    ];

    let report = validate_shex(&data, schema_json, &shape_map).unwrap();
    assert!(report.conforms);

    // Non-conformant cycle: Alice -> Bob -> Charlie -> Dave (Dave does not have toA Alice)
    let bad_data_str = r#"
        <http://example.org/Alice> <http://example.org/toB> <http://example.org/Bob> .
        <http://example.org/Bob> <http://example.org/toC> <http://example.org/Charlie> .
        <http://example.org/Charlie> <http://example.org/toA> <http://example.org/Dave> .
    "#;
    let bad_data = build_data_index(bad_data_str);
    let report_bad = validate_shex(&bad_data, schema_json, &shape_map).unwrap();
    assert!(!report_bad.conforms);
}

#[test]
fn test_shape_map_failures_and_nonexistent_shape() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/AgeShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/age",
              "valueExpr": {
                "type": "NodeConstraint",
                "datatype": "http://www.w3.org/2001/XMLSchema#integer"
              }
            }
          }
        }
      ]
    }"#;
    let data = build_data_index("<http://example.org/Alice> <http://example.org/age> 30 .");

    // Case A: Invalid node syntax in shape map (e.g. empty node string)
    let shape_map_invalid_node = vec![("".to_string(), "http://example.org/AgeShape".to_string())];
    let res = validate_shex(&data, schema_json, &shape_map_invalid_node);
    assert!(res.is_err(), "Expected parsing error for empty node string, but got Ok");

    // Case B: Non-existent shape label in shape map
    let shape_map_nonexistent_shape = vec![("http://example.org/Alice".to_string(), "http://example.org/NonExistentShape".to_string())];
    
    // We want to see if this returns Ok or Err, and if conforms is true or false.
    // If it succeeds with conforms: true, it means it silently ignored the non-existent shape.
    let report_res = validate_shex(&data, schema_json, &shape_map_nonexistent_shape);
    match report_res {
        Ok(report) => {
            println!("Nonexistent shape report: {:?}", report);
            // We expect the library to either return conforms: false, or have a failure record.
            // Let's assert whatever is the actual current behavior, but we will document if it allows silent bypass.
            // Wait, let's first check if it conforms. Let's do assert!(report.conforms) or assert!(!report.conforms)
            // based on what it actually does. Let's write an assert that will print if it conforms.
            if report.conforms {
                println!("WARNING: Silently passed validation for non-existent shape!");
            }
        }
        Err(e) => {
            println!("Validation returned expected error for non-existent shape: {:?}", e);
        }
    }
}


#[test]
fn test_stress_empty_and_invalid_inputs() {
    let empty_data = TripleIndex::new();

    // 1. Empty schema string
    let result_empty = validate_shex(&empty_data, "", &[]);
    assert!(result_empty.is_err()); // should fail to parse JSON

    // 2. Invalid JSON schema string
    let result_invalid = validate_shex(&empty_data, "{invalid}", &[]);
    assert!(result_invalid.is_err());

    // 3. Schema JSON with no shapes
    let schema_no_shapes = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema"
    }"#;
    let _ = validate_shex(&empty_data, schema_no_shapes, &[]);

    let schema_empty_shapes = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": []
    }"#;
    let report = validate_shex(&empty_data, schema_empty_shapes, &[]);
    if let Ok(rep) = report {
        assert!(rep.conforms);
        assert_eq!(rep.failures.len(), 0);
    }

    // 4. Empty shape map with valid schema
    let schema_valid = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/S",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/p",
              "valueExpr": {
                "type": "NodeConstraint",
                "nodeKind": "literal"
              }
            }
          }
        }
      ]
    }"#;
    let report_empty_map = validate_shex(&empty_data, schema_valid, &[]).unwrap();
    assert!(report_empty_map.conforms);
    assert_eq!(report_empty_map.failures.len(), 0);

    // 5. Valid schema and shape map, but node is not in data
    let shape_map = vec![
        ("http://example.org/NonExistentNode".to_string(), "http://example.org/S".to_string())
    ];
    let report_missing_node = validate_shex(&empty_data, schema_valid, &shape_map).unwrap();
    assert!(!report_missing_node.conforms);
    assert_eq!(report_missing_node.failures.len(), 1);
}

#[test]
fn test_stress_extremely_long_strings() {
    // 1. Long string in literal values (e.g. 100,000 chars)
    let long_str = "A".repeat(100_000);
    let data_str = format!(
        r#"<http://example.org/Alice> <http://example.org/bio> "{}" ."#,
        long_str
    );
    let data = build_data_index(&data_str);

    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/BioShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/bio",
              "valueExpr": {
                "type": "NodeConstraint",
                "nodeKind": "literal"
              }
            }
          }
        }
      ]
    }"#;

    let shape_map = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/BioShape".to_string())
    ];

    let start = std::time::Instant::now();
    let report = validate_shex(&data, schema_json, &shape_map).unwrap();
    let duration = start.elapsed();

    assert!(report.conforms);
    assert_eq!(report.failures.len(), 0);
    println!("Extremely long string validation took: {:?}", duration);

    // 2. Extremely long shape label / IRI (e.g. 1,000 chars)
    let long_shape_name = "http://example.org/Shape".to_string() + &"B".repeat(1000);
    let schema_long_json = format!(r#"{{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {{
          "type": "ShapeDecl",
          "id": "{}",
          "shapeExpr": {{
            "type": "Shape",
            "expression": {{
              "type": "TripleConstraint",
              "predicate": "http://example.org/bio",
              "valueExpr": {{
                "type": "NodeConstraint",
                "nodeKind": "literal"
              }}
            }}
          }}
        }}
      ]
    }}"#, long_shape_name);

    let shape_map_long = vec![
        ("http://example.org/Alice".to_string(), long_shape_name)
    ];

    let report_long = validate_shex(&data, &schema_long_json, &shape_map_long).unwrap();
    assert!(report_long.conforms);
}

#[test]
fn test_stress_nested_recursive_shapes() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/PersonShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/worksFor",
              "valueExpr": "http://example.org/CompanyShape"
            }
          }
        },
        {
          "type": "ShapeDecl",
          "id": "http://example.org/CompanyShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/employs",
              "valueExpr": "http://example.org/PersonShape"
            }
          }
        }
      ]
    }"#;

    let data_str = r#"
        <http://example.org/Alice> <http://example.org/worksFor> <http://example.org/Acme> .
        <http://example.org/Acme> <http://example.org/employs> <http://example.org/Alice> .
    "#;

    let data = build_data_index(data_str);
    let shape_map = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/PersonShape".to_string()),
        ("http://example.org/Acme".to_string(), "http://example.org/CompanyShape".to_string()),
    ];

    let report = validate_shex(&data, schema_json, &shape_map).unwrap();
    assert!(report.conforms);
    assert_eq!(report.failures.len(), 0);
}

#[test]
fn test_stress_deeply_nested_recursion() {
    let n = 30;
    let mut shapes_json = Vec::new();
    for i in 1..=n {
        let next_id = if i == n { 1 } else { i + 1 };
        let shape_decl = format!(r#"{{
          "type": "ShapeDecl",
          "id": "http://example.org/Shape{}",
          "shapeExpr": {{
            "type": "Shape",
            "expression": {{
              "type": "TripleConstraint",
              "predicate": "http://example.org/next",
              "valueExpr": "http://example.org/Shape{}"
            }}
          }}
        }}"#, i, next_id);
        shapes_json.push(shape_decl);
    }
    let schema_json = format!(r#"{{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [{}]
    }}"#, shapes_json.join(",\n"));

    let mut data_str = String::new();
    for i in 1..=n {
        let next_node = if i == n { 1 } else { i + 1 };
        data_str.push_str(&format!("<http://example.org/node{}> <http://example.org/next> <http://example.org/node{}> .\n", i, next_node));
    }

    let data = build_data_index(&data_str);
    let shape_map = vec![
        ("http://example.org/node1".to_string(), "http://example.org/Shape1".to_string())
    ];

    let report = validate_shex(&data, &schema_json, &shape_map).unwrap();
    assert!(report.conforms);
    assert_eq!(report.failures.len(), 0);
}

#[test]
fn test_stress_shape_map_failures() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/S",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/p",
              "valueExpr": {
                "type": "NodeConstraint",
                "nodeKind": "literal"
              }
            }
          }
        }
      ]
    }"#;

    let data_str = r#"<http://example.org/Alice> <http://example.org/p> "hello" ."#;
    let data = build_data_index(data_str);

    // 1. Focus node with invalid IRI format
    let shape_map_invalid_node = vec![
        ("invalid_iri".to_string(), "http://example.org/S".to_string())
    ];
    let result_invalid_node = validate_shex(&data, schema_json, &shape_map_invalid_node);
    assert!(result_invalid_node.is_err() || !result_invalid_node.unwrap().conforms);

    // 2. Shape label that does not exist in the schema
    let shape_map_invalid_shape = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/NonExistentShape".to_string())
    ];
    let report_invalid_shape = validate_shex(&data, schema_json, &shape_map_invalid_shape);
    if let Ok(rep) = report_invalid_shape {
        assert!(!rep.conforms);
        assert_eq!(rep.failures.len(), 1);
        assert_eq!(rep.failures[0].shape, "http://example.org/NonExistentShape");
    } else {
        // An Err is also acceptable — it must mention the unknown shape IRI to be informative
        let err_msg = report_invalid_shape.err().unwrap().to_string();
        assert!(
            err_msg.contains("NonExistentShape") || err_msg.contains("unknown shape") || err_msg.contains("not found"),
            "Error for unknown shape must reference the shape IRI; got: {err_msg}"
        );
    }
}


#[test]
fn test_stress_missing_properties() {
    let schema_json = r#"{
      "@context": "http://www.w3.org/ns/shex.jsonld",
      "type": "Schema",
      "shapes": [
        {
          "type": "ShapeDecl",
          "id": "http://example.org/RequiredPropShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/required",
              "min": 1,
              "max": 1,
              "valueExpr": {
                "type": "NodeConstraint",
                "nodeKind": "literal"
              }
            }
          }
        },
        {
          "type": "ShapeDecl",
          "id": "http://example.org/OptionalPropShape",
          "shapeExpr": {
            "type": "Shape",
            "expression": {
              "type": "TripleConstraint",
              "predicate": "http://example.org/optional",
              "min": 0,
              "max": 1,
              "valueExpr": {
                "type": "NodeConstraint",
                "nodeKind": "literal"
              }
            }
          }
        }
      ]
    }"#;

    let data_str = r#"<http://example.org/Alice> <http://example.org/name> "Alice" ."#;
    let data = build_data_index(data_str);

    // 1. Required property missing -> fail
    let shape_map_req = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/RequiredPropShape".to_string())
    ];
    let report_req = validate_shex(&data, schema_json, &shape_map_req).unwrap();
    assert!(!report_req.conforms);
    assert_eq!(report_req.failures.len(), 1);

    // 2. Optional property missing -> pass
    let shape_map_opt = vec![
        ("http://example.org/Alice".to_string(), "http://example.org/OptionalPropShape".to_string())
    ];
    let report_opt = validate_shex(&data, schema_json, &shape_map_opt).unwrap();
    assert!(report_opt.conforms);
    assert_eq!(report_opt.failures.len(), 0);
}

