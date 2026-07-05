use minimal::parser::Parser;

/// TICKET-005 (DoD): Test parsing N3 rules with multiple triples in the head.
/// Decomposes `{Body} => {Head1 . Head2}` into multiple rules sharing the same body.
#[test]
fn test_parse_multi_triple_head() {
    let input = "@prefix : <http://example.org/> .\n\
                 { ?x :parent ?y . ?y :sibling ?z } => { ?x :uncle ?z . ?z :nephew ?x }.";

    // Once implemented, parse_rules (or similar N3 parse function) should succeed and return decomposed rules.
    // E.g., we expect 2 rules to be generated from this single rule definition.
    let result = Parser::parse_rules(input);

    match result {
        Ok(rules) => {
            assert_eq!(
                rules.len(),
                2,
                "Expected N3 multi-triple head to decompose into 2 rules."
            );
            // Rule 1: Body => { ?x :uncle ?z }
            let r1 = &rules[0];
            assert_eq!(r1.body.len(), 2);
            assert_eq!(
                r1.head.p.to_encoded(),
                minimal::encoding::Encoder::add("<http://example.org/uncle>".to_string())
            );

            // Rule 2: Body => { ?z :nephew ?x }
            let r2 = &rules[1];
            assert_eq!(r2.body.len(), 2);
            assert_eq!(
                r2.head.p.to_encoded(),
                minimal::encoding::Encoder::add("<http://example.org/nephew>".to_string())
            );
        }
        Err(e) => {
            // Fail if not implemented or parsing fails
            panic!("test_parse_multi_triple_head failed: {}", e);
        }
    }
}

/// TICKET-005 (DoD): Test parsing N3 literals (strings, numbers, langtags, datatypes).
#[test]
fn test_parse_literal_terms() {
    let input = "@prefix : <http://example.org/> .\n\
                 @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
                 { ?s :hasValue 42 } => { ?s :status \"active\" }.\n\
                 { ?s :greeting \"hello\"@en } => { ?s :greetingFrench \"bonjour\"^^xsd:string }.";

    let result = Parser::parse_rules(input);
    match result {
        Ok(rules) => {
            assert!(!rules.is_empty(), "Parsed rules should not be empty");
            // Check that literals are parsed correctly. Once Term::Literal is implemented,
            // we should assert on the specific structure of the TermImpl/Literal variant.
            todo!("TICKET-005: Assert detailed literal values, types, and language tags when Term model is updated.");
        }
        Err(_) => {
            // Fail with todo until N3 literal grammar is fully supported
            todo!("TICKET-005: Support literals in N3 grammar");
        }
    }
}

/// TICKET-005 (DoD): Test parsing N3 blank nodes (_:x and [...]).
#[test]
fn test_parse_blank_nodes() {
    let input = "@prefix : <http://example.org/> .\n\
                 { _:blank :property :value } => { :s :p [ :nested :value ] }.";

    let result = Parser::parse_rules(input);
    match result {
        Ok(_) => {
            todo!("TICKET-005: Assert that blank nodes and nested blank nodes parse into Term::BlankNode variants");
        }
        Err(_) => {
            todo!("TICKET-005: Support blank nodes in N3 grammar");
        }
    }
}

/// TICKET-005 (DoD): Test parsing RDF lists (e.g. (1 2 3)).
#[test]
fn test_parse_lists() {
    let input = "@prefix : <http://example.org/> .\n\
                 { ?s :hasList ( 1 2 3 ) } => { ?s :status :valid }.";

    let result = Parser::parse_rules(input);
    match result {
        Ok(_) => {
            todo!("TICKET-005: Assert RDF lists parse correctly into linked list of rdf:first/rdf:rest or custom structures");
        }
        Err(_) => {
            todo!("TICKET-005: Support lists in N3 grammar");
        }
    }
}

/// TICKET-005 (DoD): Test parsing @forAll and @forSome quantifiers.
#[test]
fn test_parse_quantifiers() {
    let input = "@prefix : <http://example.org/> .\n\
                 @forAll ?x .\n\
                 @forSome ?y .\n\
                 { ?x :knows ?y } => { ?x :hasConnection :yes }.";

    let result = Parser::parse_rules(input);
    match result {
        Ok(_) => {
            todo!("TICKET-005: Assert quantifiers scoped correctly to rule or document scope");
        }
        Err(_) => {
            todo!("TICKET-005: Support @forAll/@forSome quantifiers in N3 grammar");
        }
    }
}

/// TICKET-005 (DoD): Test parsing quoted graphs (e.g., {...} used as terms).
#[test]
fn test_parse_quoted_graphs() {
    let input = "@prefix : <http://example.org/> .\n\
                 { :alice :believes { :bob :status :happy } } => { :bob :isEnviedBy :alice }.";

    let result = Parser::parse_rules(input);
    match result {
        Ok(_) => {
            todo!("TICKET-005: Assert quoted graphs parse into appropriate term variants for reasoning");
        }
        Err(_) => {
            todo!("TICKET-005: Support quoted graphs in N3 grammar");
        }
    }
}

/// TICKET-005 (DoD): Test parsing backward implication (<=).
#[test]
fn test_parse_backward_implication() {
    let input = "@prefix : <http://example.org/> .\n\
                 { ?x :status :happy } <= { ?x :status :rich }.";

    let result = Parser::parse_rules(input);
    match result {
        Ok(rules) => {
            assert_eq!(rules.len(), 1);
            // Backward implication swaps body and head at parse time,
            // translating to: { ?x :status :rich } => { ?x :status :happy }
            let rule = &rules[0];
            assert_eq!(rule.body.len(), 1);
            assert_eq!(
                rule.body[0].pattern.p.to_encoded(),
                minimal::encoding::Encoder::add("<http://example.org/status>".to_string())
            );
            assert_eq!(
                rule.body[0].pattern.o.to_encoded(),
                minimal::encoding::Encoder::add("<http://example.org/rich>".to_string())
            );
            assert_eq!(
                rule.head.o.to_encoded(),
                minimal::encoding::Encoder::add("<http://example.org/happy>".to_string())
            );
        }
        Err(_) => {
            todo!("TICKET-005: Support backward implication <= in N3 grammar");
        }
    }
}

/// TICKET-005 (DoD): Test that N3 comments (#) are ignored during parsing.
#[test]
fn test_parse_comments_ignored() {
    let input = "# This is a prefix section comment\n\
                 @prefix : <http://example.org/> . # Line-end prefix comment\n\
                 # Rule starts here\n\
                 { ?x :status :rich } => { ?x :status :happy } . # Implication comment";

    let result = Parser::parse_rules(input);
    match result {
        Ok(rules) => {
            assert_eq!(rules.len(), 1);
            let rule = &rules[0];
            assert_eq!(rule.body.len(), 1);
        }
        Err(e) => {
            panic!("Failed to parse rules with comments: {}", e);
        }
    }
}

/// TICKET-005 (DoD): Test that malformed rules return a ParseError result instead of panicking.
#[test]
fn test_parse_error_returns_result_not_panic() {
    // Malformed input lacking closing brace
    let input = "@prefix : <http://example.org/> .\n\
                 { ?x :status :rich => { ?x :status :happy }.";

    let result = std::panic::catch_unwind(|| Parser::parse_rules(input));

    match result {
        Ok(parse_result) => {
            assert!(
                parse_result.is_err(),
                "Expected parsing a malformed rule to return an Err Result."
            );
        }
        Err(_) => {
            panic!("Parser panicked instead of returning Err Result!");
        }
    }
}
