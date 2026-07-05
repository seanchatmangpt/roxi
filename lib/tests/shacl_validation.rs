use minimal::shacl::{Validator, ShapesGraph};
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
fn test_min_max_count_violation() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:property [
                sh:path ex:name ;
                sh:minCount 2 ;
                sh:maxCount 3 ;
            ] .
    "#;

    let data_str = r#"
        @prefix ex: <http://example.org/> .

        ex:Alice a ex:Person ;
            ex:name "Alice" . # Only 1 name, violates minCount 2

        ex:Bob a ex:Person ;
            ex:name "Bob" , "Robert" , "Bobby" , "Rob" . # 4 names, violates maxCount 3

        ex:Charlie a ex:Person ;
            ex:name "Charlie" , "Chuck" . # 2 names, passes
    "#;

    let data = build_data_index(data_str);
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);

    assert!(!report.conforms);
    assert_eq!(report.results.len(), 2);

    let alice_results: Vec<_> = report.results.iter().filter(|r| r.focus_node.to_string().contains("Alice")).collect();
    assert_eq!(alice_results.len(), 1);
    assert_eq!(alice_results[0].source_constraint_component.to_string(), "<http://www.w3.org/ns/shacl#MinCountConstraintComponent>");

    let bob_results: Vec<_> = report.results.iter().filter(|r| r.focus_node.to_string().contains("Bob")).collect();
    assert_eq!(bob_results.len(), 1);
    assert_eq!(bob_results[0].source_constraint_component.to_string(), "<http://www.w3.org/ns/shacl#MaxCountConstraintComponent>");
}

#[test]
fn test_datatype_constraint_pass_fail() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:property [
                sh:path ex:age ;
                sh:datatype xsd:integer ;
            ] .
    "#;

    let data_str = r#"
        @prefix ex: <http://example.org/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:Alice a ex:Person ;
            ex:age 30 . # passes

        ex:Bob a ex:Person ;
            ex:age "thirty" . # fails (string instead of integer)
    "#;

    let data = build_data_index(data_str);
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);

    assert!(!report.conforms);
    let bob_results: Vec<_> = report.results.iter().filter(|r| r.focus_node.to_string().contains("Bob")).collect();
    assert_eq!(bob_results.len(), 1);
    assert_eq!(bob_results[0].source_constraint_component.to_string(), "<http://www.w3.org/ns/shacl#DatatypeConstraintComponent>");
}

#[test]
fn test_class_constraint() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:property [
                sh:path ex:knows ;
                sh:class ex:Person ;
            ] .
    "#;

    let data_str = r#"
        @prefix ex: <http://example.org/> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        ex:Student rdfs:subClassOf ex:Person .

        ex:Alice a ex:Person ;
            ex:knows ex:Bob , ex:Charlie , ex:Rex .

        ex:Bob a ex:Person . # directly matches Person
        ex:Charlie a ex:Student . # subclass of Person (should pass subclass check)
        ex:Rex a ex:Animal . # not Person or subclass (should fail)
    "#;

    let data = build_data_index(data_str);
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);

    assert!(!report.conforms);
    let rex_results: Vec<_> = report.results.iter().filter(|r| r.value.as_ref().map(|v| v.to_string().contains("Rex")).unwrap_or(false)).collect();
    assert_eq!(rex_results.len(), 1);
    assert_eq!(rex_results[0].source_constraint_component.to_string(), "<http://www.w3.org/ns/shacl#ClassConstraintComponent>");
}

#[test]
fn test_and_or_not_logical_constraints() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:AndShape a sh:NodeShape ;
            sh:targetNode ex:AliceAnd ;
            sh:and (
                [ sh:property [ sh:path ex:name; sh:minCount 1 ] ]
                [ sh:property [ sh:path ex:age; sh:minCount 1 ] ]
            ) .

        ex:OrShape a sh:NodeShape ;
            sh:targetNode ex:BobOr ;
            sh:or (
                [ sh:property [ sh:path ex:name; sh:minCount 1 ] ]
                [ sh:property [ sh:path ex:age; sh:minCount 1 ] ]
            ) .

        ex:NotShape a sh:NodeShape ;
            sh:targetNode ex:CharlieNot ;
            sh:not [
                sh:property [ sh:path ex:name; sh:minCount 1 ]
            ] .
    "#;

    let data_str = r#"
        @prefix ex: <http://example.org/> .

        # AliceAnd has name and age -> passes AndShape
        ex:AliceAnd ex:name "Alice" ; ex:age 30 .

        # BobOr has only age -> passes OrShape
        ex:BobOr ex:age 30 .

        # CharlieNot has no name -> passes NotShape
        ex:CharlieNot ex:age 30 .
    "#;

    let data = build_data_index(data_str);
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);

    assert!(report.conforms);
    assert_eq!(report.results.len(), 0);

    // Let's modify the data to trigger violations
    let data_str_violating = r#"
        @prefix ex: <http://example.org/> .

        # AliceAnd has only name -> fails AndShape
        ex:AliceAnd ex:name "Alice" .

        # BobOr has neither -> fails OrShape
        ex:BobOr a ex:Something .

        # CharlieNot has name -> fails NotShape
        ex:CharlieNot ex:name "Charlie" .
    "#;

    let data_violating = build_data_index(data_str_violating);
    let report_violating = Validator::validate(&data_violating, &shapes);

    assert!(!report_violating.conforms);
    assert_eq!(report_violating.results.len(), 3);
}

#[test]
fn test_conforms_true_for_valid_graph() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:property [
                sh:path ex:name ;
                sh:minCount 1 ;
            ] .
    "#;

    let data_str = r#"
        @prefix ex: <http://example.org/> .

        ex:Alice a ex:Person ;
            ex:name "Alice" .
    "#;

    let data = build_data_index(data_str);
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);

    assert!(report.conforms);
    assert_eq!(report.results.len(), 0);
}

#[test]
fn test_empty_dataset() {
    // 1. Shapes with targetClass but data is completely empty. Should conform because no focus nodes.
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:property [
                sh:path ex:name ;
                sh:minCount 1 ;
            ] .
    "#;

    let data = build_data_index("");
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);
    assert!(report.conforms);
    assert_eq!(report.results.len(), 0);

    // 2. Shapes with targetNode and empty data. Should violate minCount.
    let shapes_str_node = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetNode ex:Alice ;
            sh:property [
                sh:path ex:name ;
                sh:minCount 1 ;
            ] .
    "#;

    let shapes_node = ShapesGraph::parse(shapes_str_node).unwrap();
    let report_node = Validator::validate(&data, &shapes_node);
    assert!(!report_node.conforms);
    assert_eq!(report_node.results.len(), 1);
    assert_eq!(report_node.results[0].focus_node.to_string(), "<http://example.org/Alice>");
    assert_eq!(report_node.results[0].source_constraint_component.to_string(), "<http://www.w3.org/ns/shacl#MinCountConstraintComponent>");
}

#[test]
fn test_invalid_turtle_shapes() {
    // Missing semicolon or dot, invalid syntax
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PersonShape a sh:NodeShape
            sh:targetClass ex:Person
    "#;
    let shapes = ShapesGraph::parse(shapes_str);
    assert!(shapes.is_err());
}

#[test]
fn test_recursive_shapes() {
    // Recursive validation using sh:and pointing back to the node shape itself.
    // ex:Alice is a Person, knows ex:Bob.
    // ex:Bob is a Person, knows ex:Alice.
    // ex:Alice has name "Alice".
    // ex:Bob does not have a name (violates minCount 1).
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:property [
                sh:path ex:name ;
                sh:minCount 1 ;
            ] ;
            sh:property [
                sh:path ex:knows ;
                sh:and ( ex:PersonShape ) ;
            ] .
    "#;

    let data_str = r#"
        @prefix ex: <http://example.org/> .

        ex:Alice a ex:Person ;
            ex:name "Alice" ;
            ex:knows ex:Bob .

        ex:Bob a ex:Person ;
            ex:knows ex:Alice .
    "#;

    let data = build_data_index(data_str);
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);

    // Should fail because Bob does not have a name, and Bob is validated recursively because Alice knows Bob.
    // Bob should have a minCount violation for ex:name.
    // Let's check results.
    assert!(!report.conforms);
    
    // We expect a violation for Bob missing name, and possibly Bob failing the `and` constraint for knows Alice (wait, does Alice fail anything? No, Alice conforms).
    // Let's see what focus nodes failed: Bob should have a minCount violation on name.
    let bob_name_viol: Vec<_> = report.results.iter()
        .filter(|r| r.focus_node.to_string().contains("Bob") && r.source_constraint_component.to_string().contains("MinCountConstraintComponent"))
        .collect();
    assert!(!bob_name_viol.is_empty());
}

#[test]
fn test_property_paths_comprehensive() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .

        ex:PathShape a sh:NodeShape ;
            sh:targetNode ex:Alice ;
            
            # 1. Alternative path: ex:father | ex:mother
            sh:property [
                sh:path [ sh:alternativePath (ex:father ex:mother) ] ;
                sh:minCount 2 ;
            ] ;
            
            # 2. Inverse path: ^ex:child
            sh:property [
                sh:path [ sh:inversePath ex:child ] ;
                sh:minCount 1 ;
            ] ;
            
            # 3. Zero-or-more path: ex:knows*
            sh:property [
                sh:path [ sh:zeroOrMorePath ex:knows ] ;
                sh:minCount 3 ;
            ] ;
            
            # 4. One-or-more path: ex:knows+
            sh:property [
                sh:path [ sh:oneOrMorePath ex:knows ] ;
                sh:minCount 2 ;
            ] ;
            
            # 5. Zero-or-one path: ex:knows?
            sh:property [
                sh:path [ sh:zeroOrOnePath ex:knows ] ;
                sh:minCount 2 ;
            ] ;
            
            # 6. Sequence path: ex:knows / ex:knows
            sh:property [
                sh:path (ex:knows ex:knows) ;
                sh:minCount 1 ;
            ] .
    "#;

    let data_str = r#"
        @prefix ex: <http://example.org/> .

        # ex:Alice has father ex:Bob and mother ex:Charlie -> alternative path count = 2
        ex:Alice ex:father ex:Bob ;
                 ex:mother ex:Charlie .

        # ex:Diana has child ex:Alice -> inverse path of ex:child on ex:Alice -> value is ex:Diana
        ex:Diana ex:child ex:Alice .

        # ex:Alice knows ex:Bob. ex:Bob knows ex:Charlie.
        # zero-or-more knows: ex:Alice, ex:Bob, ex:Charlie -> count = 3
        # one-or-more knows: ex:Bob, ex:Charlie -> count = 2
        # zero-or-one knows: ex:Alice, ex:Bob -> count = 2
        # sequence knows / knows: ex:Charlie -> count = 1
        ex:Alice ex:knows ex:Bob .
        ex:Bob ex:knows ex:Charlie .
    "#;

    let data = build_data_index(data_str);
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);

    // Everything is designed to pass
    assert!(report.conforms, "Expected all property path constraints to pass, but got errors: {:?}", report.results);

    // Now let's test failing cases by making data insufficient
    let data_str_fail = r#"
        @prefix ex: <http://example.org/> .
        ex:Alice ex:father ex:Bob . # Alternative path count is 1 (fails minCount 2)
        # Missing inverse path ^ex:child (fails minCount 1)
        # Alice knows Bob, no more (zero-or-more knows is Alice, Bob -> count 2, fails minCount 3)
        # one-or-more knows is Bob -> count 1, fails minCount 2
        # zero-or-one knows is Alice, Bob -> count 2, passes minCount 2
        # sequence knows / knows -> empty, fails minCount 1
        ex:Alice ex:knows ex:Bob .
    "#;

    let data_fail = build_data_index(data_str_fail);
    let report_fail = Validator::validate(&data_fail, &shapes);
    assert!(!report_fail.conforms);
    // Let's count how many failures we got
    assert!(report_fail.results.len() >= 4);
}

#[test]
fn test_severity_and_datatype() {
    let shapes_str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

        ex:SeverityShape a sh:NodeShape ;
            sh:targetNode ex:Alice ;
            sh:severity sh:Warning ;
            sh:property [
                sh:path ex:age ;
                sh:datatype xsd:integer ;
                sh:severity sh:Info ;
            ] .
    "#;

    let data_str = r#"
        @prefix ex: <http://example.org/> .
        ex:Alice ex:age "thirty" . # fails datatype xsd:integer
    "#;

    let data = build_data_index(data_str);
    let shapes = ShapesGraph::parse(shapes_str).unwrap();
    let report = Validator::validate(&data, &shapes);

    assert!(!report.conforms);
    assert_eq!(report.results.len(), 1);
    
    // The violation is on the property shape, so it should carry the property's sh:severity, which is sh:Info.
    assert_eq!(report.results[0].severity.to_string(), "<http://www.w3.org/ns/shacl#Info>");
}
