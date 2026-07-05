use crate::encoding::Encoder;
use crate::tripleindex::TripleIndex;
use crate::triples::{Term, TermImpl, Triple, VarOrTerm};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Vocabulary
// ---------------------------------------------------------------------------

pub struct Vocab {
    pub rdf_type: usize,
    pub rdf_first: usize,
    pub rdf_rest: usize,
    pub rdf_nil: usize,
    pub rdfs_class: usize,
    pub rdfs_subclass_of: usize,
    pub sh_node_shape: usize,
    pub sh_property_shape: usize,
    pub sh_target_class: usize,
    pub sh_target_node: usize,
    pub sh_target_subjects_of: usize,
    pub sh_target_objects_of: usize,
    pub sh_property: usize,
    pub sh_node: usize,
    pub sh_path: usize,
    pub sh_min_count: usize,
    pub sh_max_count: usize,
    pub sh_datatype: usize,
    pub sh_class: usize,
    pub sh_pattern: usize,
    pub sh_flags: usize,
    pub sh_in: usize,
    pub sh_and: usize,
    pub sh_or: usize,
    pub sh_not: usize,
    pub sh_xone: usize,
    pub sh_node_kind: usize,
    pub sh_has_value: usize,
    pub sh_min_length: usize,
    pub sh_max_length: usize,
    pub sh_min_exclusive: usize,
    pub sh_min_inclusive: usize,
    pub sh_max_exclusive: usize,
    pub sh_max_inclusive: usize,
    pub sh_language_in: usize,
    pub sh_unique_lang: usize,
    pub sh_equals: usize,
    pub sh_disjoint: usize,
    pub sh_less_than: usize,
    pub sh_less_than_or_equals: usize,
    pub sh_qualified_value_shape: usize,
    pub sh_qualified_min_count: usize,
    pub sh_qualified_max_count: usize,
    pub sh_closed: usize,
    pub sh_ignored_properties: usize,
    pub sh_deactivated: usize,
    pub sh_message: usize,
    pub sh_severity: usize,
    pub sh_violation: usize,
    pub sh_warning: usize,
    pub sh_info: usize,
    pub sh_validation_report: usize,
    pub sh_conforms: usize,
    pub sh_result: usize,
    pub sh_validation_result: usize,
    pub sh_focus_node: usize,
    pub sh_result_path: usize,
    pub sh_value: usize,
    pub sh_result_message: usize,
    pub sh_source_constraint_component: usize,
    pub sh_source_shape: usize,
    pub sh_result_severity: usize,
    pub sh_alternative_path: usize,
    pub sh_inverse_path: usize,
    pub sh_zero_or_more_path: usize,
    pub sh_one_or_more_path: usize,
    pub sh_zero_or_one_path: usize,

    // sh:nodeKind values
    pub sh_iri: usize,
    pub sh_blank_node: usize,
    pub sh_literal: usize,
    pub sh_blank_node_or_iri: usize,
    pub sh_blank_node_or_literal: usize,
    pub sh_iri_or_literal: usize,

    // Constraint Components
    pub sh_min_count_constraint_component: usize,
    pub sh_max_count_constraint_component: usize,
    pub sh_datatype_constraint_component: usize,
    pub sh_class_constraint_component: usize,
    pub sh_pattern_constraint_component: usize,
    pub sh_in_constraint_component: usize,
    pub sh_and_constraint_component: usize,
    pub sh_or_constraint_component: usize,
    pub sh_not_constraint_component: usize,
    pub sh_xone_constraint_component: usize,
    pub sh_node_kind_constraint_component: usize,
    pub sh_has_value_constraint_component: usize,
    pub sh_min_length_constraint_component: usize,
    pub sh_max_length_constraint_component: usize,
    pub sh_min_exclusive_constraint_component: usize,
    pub sh_min_inclusive_constraint_component: usize,
    pub sh_max_exclusive_constraint_component: usize,
    pub sh_max_inclusive_constraint_component: usize,
    pub sh_language_in_constraint_component: usize,
    pub sh_unique_lang_constraint_component: usize,
    pub sh_equals_constraint_component: usize,
    pub sh_disjoint_constraint_component: usize,
    pub sh_less_than_constraint_component: usize,
    pub sh_less_than_or_equals_constraint_component: usize,
    pub sh_qualified_value_shape_constraint_component: usize,
    pub sh_closed_constraint_component: usize,
    pub sh_node_constraint_component: usize,
}

impl Vocab {
    pub fn new() -> Self {
        Vocab {
            rdf_type: Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string()),
            rdf_first: Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>".to_string()),
            rdf_rest: Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>".to_string()),
            rdf_nil: Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>".to_string()),
            rdfs_class: Encoder::add("<http://www.w3.org/2000/01/rdf-schema#Class>".to_string()),
            rdfs_subclass_of: Encoder::add("<http://www.w3.org/2000/01/rdf-schema#subClassOf>".to_string()),
            sh_node_shape: Encoder::add("<http://www.w3.org/ns/shacl#NodeShape>".to_string()),
            sh_property_shape: Encoder::add("<http://www.w3.org/ns/shacl#PropertyShape>".to_string()),
            sh_target_class: Encoder::add("<http://www.w3.org/ns/shacl#targetClass>".to_string()),
            sh_target_node: Encoder::add("<http://www.w3.org/ns/shacl#targetNode>".to_string()),
            sh_target_subjects_of: Encoder::add("<http://www.w3.org/ns/shacl#targetSubjectsOf>".to_string()),
            sh_target_objects_of: Encoder::add("<http://www.w3.org/ns/shacl#targetObjectsOf>".to_string()),
            sh_property: Encoder::add("<http://www.w3.org/ns/shacl#property>".to_string()),
            sh_node: Encoder::add("<http://www.w3.org/ns/shacl#node>".to_string()),
            sh_path: Encoder::add("<http://www.w3.org/ns/shacl#path>".to_string()),
            sh_min_count: Encoder::add("<http://www.w3.org/ns/shacl#minCount>".to_string()),
            sh_max_count: Encoder::add("<http://www.w3.org/ns/shacl#maxCount>".to_string()),
            sh_datatype: Encoder::add("<http://www.w3.org/ns/shacl#datatype>".to_string()),
            sh_class: Encoder::add("<http://www.w3.org/ns/shacl#class>".to_string()),
            sh_pattern: Encoder::add("<http://www.w3.org/ns/shacl#pattern>".to_string()),
            sh_flags: Encoder::add("<http://www.w3.org/ns/shacl#flags>".to_string()),
            sh_in: Encoder::add("<http://www.w3.org/ns/shacl#in>".to_string()),
            sh_and: Encoder::add("<http://www.w3.org/ns/shacl#and>".to_string()),
            sh_or: Encoder::add("<http://www.w3.org/ns/shacl#or>".to_string()),
            sh_not: Encoder::add("<http://www.w3.org/ns/shacl#not>".to_string()),
            sh_xone: Encoder::add("<http://www.w3.org/ns/shacl#xone>".to_string()),
            sh_node_kind: Encoder::add("<http://www.w3.org/ns/shacl#nodeKind>".to_string()),
            sh_has_value: Encoder::add("<http://www.w3.org/ns/shacl#hasValue>".to_string()),
            sh_min_length: Encoder::add("<http://www.w3.org/ns/shacl#minLength>".to_string()),
            sh_max_length: Encoder::add("<http://www.w3.org/ns/shacl#maxLength>".to_string()),
            sh_min_exclusive: Encoder::add("<http://www.w3.org/ns/shacl#minExclusive>".to_string()),
            sh_min_inclusive: Encoder::add("<http://www.w3.org/ns/shacl#minInclusive>".to_string()),
            sh_max_exclusive: Encoder::add("<http://www.w3.org/ns/shacl#maxExclusive>".to_string()),
            sh_max_inclusive: Encoder::add("<http://www.w3.org/ns/shacl#maxInclusive>".to_string()),
            sh_language_in: Encoder::add("<http://www.w3.org/ns/shacl#languageIn>".to_string()),
            sh_unique_lang: Encoder::add("<http://www.w3.org/ns/shacl#uniqueLang>".to_string()),
            sh_equals: Encoder::add("<http://www.w3.org/ns/shacl#equals>".to_string()),
            sh_disjoint: Encoder::add("<http://www.w3.org/ns/shacl#disjoint>".to_string()),
            sh_less_than: Encoder::add("<http://www.w3.org/ns/shacl#lessThan>".to_string()),
            sh_less_than_or_equals: Encoder::add("<http://www.w3.org/ns/shacl#lessThanOrEquals>".to_string()),
            sh_qualified_value_shape: Encoder::add("<http://www.w3.org/ns/shacl#qualifiedValueShape>".to_string()),
            sh_qualified_min_count: Encoder::add("<http://www.w3.org/ns/shacl#qualifiedMinCount>".to_string()),
            sh_qualified_max_count: Encoder::add("<http://www.w3.org/ns/shacl#qualifiedMaxCount>".to_string()),
            sh_closed: Encoder::add("<http://www.w3.org/ns/shacl#closed>".to_string()),
            sh_ignored_properties: Encoder::add("<http://www.w3.org/ns/shacl#ignoredProperties>".to_string()),
            sh_deactivated: Encoder::add("<http://www.w3.org/ns/shacl#deactivated>".to_string()),
            sh_message: Encoder::add("<http://www.w3.org/ns/shacl#message>".to_string()),
            sh_severity: Encoder::add("<http://www.w3.org/ns/shacl#severity>".to_string()),
            sh_violation: Encoder::add("<http://www.w3.org/ns/shacl#Violation>".to_string()),
            sh_warning: Encoder::add("<http://www.w3.org/ns/shacl#Warning>".to_string()),
            sh_info: Encoder::add("<http://www.w3.org/ns/shacl#Info>".to_string()),
            sh_validation_report: Encoder::add("<http://www.w3.org/ns/shacl#ValidationReport>".to_string()),
            sh_conforms: Encoder::add("<http://www.w3.org/ns/shacl#conforms>".to_string()),
            sh_result: Encoder::add("<http://www.w3.org/ns/shacl#result>".to_string()),
            sh_validation_result: Encoder::add("<http://www.w3.org/ns/shacl#ValidationResult>".to_string()),
            sh_focus_node: Encoder::add("<http://www.w3.org/ns/shacl#focusNode>".to_string()),
            sh_result_path: Encoder::add("<http://www.w3.org/ns/shacl#resultPath>".to_string()),
            sh_value: Encoder::add("<http://www.w3.org/ns/shacl#value>".to_string()),
            sh_result_message: Encoder::add("<http://www.w3.org/ns/shacl#resultMessage>".to_string()),
            sh_source_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#sourceConstraintComponent>".to_string()),
            sh_source_shape: Encoder::add("<http://www.w3.org/ns/shacl#sourceShape>".to_string()),
            sh_result_severity: Encoder::add("<http://www.w3.org/ns/shacl#resultSeverity>".to_string()),
            sh_alternative_path: Encoder::add("<http://www.w3.org/ns/shacl#alternativePath>".to_string()),
            sh_inverse_path: Encoder::add("<http://www.w3.org/ns/shacl#inversePath>".to_string()),
            sh_zero_or_more_path: Encoder::add("<http://www.w3.org/ns/shacl#zeroOrMorePath>".to_string()),
            sh_one_or_more_path: Encoder::add("<http://www.w3.org/ns/shacl#oneOrMorePath>".to_string()),
            sh_zero_or_one_path: Encoder::add("<http://www.w3.org/ns/shacl#zeroOrOnePath>".to_string()),

            // sh:nodeKind values
            sh_iri: Encoder::add("<http://www.w3.org/ns/shacl#IRI>".to_string()),
            sh_blank_node: Encoder::add("<http://www.w3.org/ns/shacl#BlankNode>".to_string()),
            sh_literal: Encoder::add("<http://www.w3.org/ns/shacl#Literal>".to_string()),
            sh_blank_node_or_iri: Encoder::add("<http://www.w3.org/ns/shacl#BlankNodeOrIRI>".to_string()),
            sh_blank_node_or_literal: Encoder::add("<http://www.w3.org/ns/shacl#BlankNodeOrLiteral>".to_string()),
            sh_iri_or_literal: Encoder::add("<http://www.w3.org/ns/shacl#IRIOrLiteral>".to_string()),

            // Constraint Components
            sh_min_count_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MinCountConstraintComponent>".to_string()),
            sh_max_count_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MaxCountConstraintComponent>".to_string()),
            sh_datatype_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#DatatypeConstraintComponent>".to_string()),
            sh_class_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#ClassConstraintComponent>".to_string()),
            sh_pattern_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#PatternConstraintComponent>".to_string()),
            sh_in_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#InConstraintComponent>".to_string()),
            sh_and_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#AndConstraintComponent>".to_string()),
            sh_or_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#OrConstraintComponent>".to_string()),
            sh_not_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#NotConstraintComponent>".to_string()),
            sh_xone_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#XoneConstraintComponent>".to_string()),
            sh_node_kind_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#NodeKindConstraintComponent>".to_string()),
            sh_has_value_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#HasValueConstraintComponent>".to_string()),
            sh_min_length_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MinLengthConstraintComponent>".to_string()),
            sh_max_length_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MaxLengthConstraintComponent>".to_string()),
            sh_min_exclusive_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MinExclusiveConstraintComponent>".to_string()),
            sh_min_inclusive_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MinInclusiveConstraintComponent>".to_string()),
            sh_max_exclusive_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MaxExclusiveConstraintComponent>".to_string()),
            sh_max_inclusive_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#MaxInclusiveConstraintComponent>".to_string()),
            sh_language_in_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#LanguageInConstraintComponent>".to_string()),
            sh_unique_lang_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#UniqueLangConstraintComponent>".to_string()),
            sh_equals_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#EqualsConstraintComponent>".to_string()),
            sh_disjoint_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#DisjointConstraintComponent>".to_string()),
            sh_less_than_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#LessThanConstraintComponent>".to_string()),
            sh_less_than_or_equals_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#LessThanOrEqualsConstraintComponent>".to_string()),
            sh_qualified_value_shape_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#QualifiedValueShapeConstraintComponent>".to_string()),
            sh_closed_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#ClosedConstraintComponent>".to_string()),
            sh_node_constraint_component: Encoder::add("<http://www.w3.org/ns/shacl#NodeConstraintComponent>".to_string()),
        }
    }
}

impl Default for Vocab {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ShapesGraph
// ---------------------------------------------------------------------------

pub struct ShapesGraph {
    pub raw_index: TripleIndex,
}

impl ShapesGraph {
    pub fn parse(shapes_str: &str) -> Result<Self, String> {
        let triples = crate::parser::Parser::parse_triples(shapes_str, crate::parser::Syntax::Turtle)?;
        let mut raw_index = TripleIndex::new();
        for triple in triples {
            raw_index.add(triple);
        }
        Ok(ShapesGraph { raw_index })
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidationResult {
    pub focus_node: Term,
    pub result_path: Option<Term>,
    pub value: Option<Term>,
    pub source_constraint_component: Term,
    pub source_shape: Term,
    pub severity: Term,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidationReport {
    pub conforms: bool,
    pub results: Vec<ValidationResult>,
}

impl ValidationReport {
    /// Serialise the report as SHACL-conformant RDF triples.
    pub fn to_triples(&self) -> Vec<Triple> {
        let mut triples = Vec::new();

        let report_id = format!("report_{}", self.results.len());
        let report_term = VarOrTerm::new_blank_node(report_id);

        let rdf_type = VarOrTerm::convert("http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string());
        let sh_validation_report = VarOrTerm::convert("http://www.w3.org/ns/shacl#ValidationReport".to_string());
        let sh_conforms = VarOrTerm::convert("http://www.w3.org/ns/shacl#conforms".to_string());
        let sh_result = VarOrTerm::convert("http://www.w3.org/ns/shacl#result".to_string());
        let sh_focus_node = VarOrTerm::convert("http://www.w3.org/ns/shacl#focusNode".to_string());
        let sh_result_path = VarOrTerm::convert("http://www.w3.org/ns/shacl#resultPath".to_string());
        let sh_value = VarOrTerm::convert("http://www.w3.org/ns/shacl#value".to_string());
        let sh_source_constraint_component = VarOrTerm::convert("http://www.w3.org/ns/shacl#sourceConstraintComponent".to_string());
        let sh_source_shape = VarOrTerm::convert("http://www.w3.org/ns/shacl#sourceShape".to_string());
        let sh_result_severity = VarOrTerm::convert("http://www.w3.org/ns/shacl#resultSeverity".to_string());
        let sh_result_message = VarOrTerm::convert("http://www.w3.org/ns/shacl#resultMessage".to_string());
        let sh_validation_result = VarOrTerm::convert("http://www.w3.org/ns/shacl#ValidationResult".to_string());

        triples.push(Triple {
            s: report_term.clone(),
            p: rdf_type.clone(),
            o: sh_validation_report,
            g: None,
        });

        let conforms_str = if self.conforms { "true" } else { "false" };
        let conforms_literal = VarOrTerm::new_literal(
            conforms_str.to_string(),
            Some("<http://www.w3.org/2001/XMLSchema#boolean>".to_string()),
            None,
        );
        triples.push(Triple {
            s: report_term.clone(),
            p: sh_conforms,
            o: conforms_literal,
            g: None,
        });

        for (i, res) in self.results.iter().enumerate() {
            let res_term = VarOrTerm::new_blank_node(format!("result_{}", i));

            triples.push(Triple {
                s: report_term.clone(),
                p: sh_result.clone(),
                o: res_term.clone(),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: rdf_type.clone(),
                o: sh_validation_result.clone(),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: sh_focus_node.clone(),
                o: VarOrTerm::Term(res.focus_node.clone()),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: sh_source_constraint_component.clone(),
                o: VarOrTerm::Term(res.source_constraint_component.clone()),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: sh_source_shape.clone(),
                o: VarOrTerm::Term(res.source_shape.clone()),
                g: None,
            });

            triples.push(Triple {
                s: res_term.clone(),
                p: sh_result_severity.clone(),
                o: VarOrTerm::Term(res.severity.clone()),
                g: None,
            });

            if let Some(ref path) = res.result_path {
                triples.push(Triple {
                    s: res_term.clone(),
                    p: sh_result_path.clone(),
                    o: VarOrTerm::Term(path.clone()),
                    g: None,
                });
            }

            if let Some(ref val) = res.value {
                triples.push(Triple {
                    s: res_term.clone(),
                    p: sh_value.clone(),
                    o: VarOrTerm::Term(val.clone()),
                    g: None,
                });
            }

            if let Some(ref msg) = res.message {
                let msg_literal = VarOrTerm::new_literal(msg.clone(), None, None);
                triples.push(Triple {
                    s: res_term.clone(),
                    p: sh_result_message.clone(),
                    o: msg_literal,
                    g: None,
                });
            }
        }

        triples
    }
}

// ---------------------------------------------------------------------------
// Validator entry point
// ---------------------------------------------------------------------------

pub struct Validator;

impl Validator {
    pub fn validate(data: &TripleIndex, shapes: &ShapesGraph) -> ValidationReport {
        let vocab = Vocab::new();
        let shapes_index = &shapes.raw_index;

        let mut shape_nodes = HashSet::new();

        // Subjects of rdf:type sh:NodeShape or sh:PropertyShape
        for s in get_subjects(shapes_index, vocab.rdf_type, vocab.sh_node_shape) {
            shape_nodes.insert(s);
        }
        for s in get_subjects(shapes_index, vocab.rdf_type, vocab.sh_property_shape) {
            shape_nodes.insert(s);
        }

        // Subjects of any target declaration
        for t in [
            vocab.sh_target_class,
            vocab.sh_target_node,
            vocab.sh_target_subjects_of,
            vocab.sh_target_objects_of,
        ] {
            if let Some(objs) = shapes_index.pos.get(&t) {
                for subjs in objs.values() {
                    for (s, _, _) in subjs {
                        shape_nodes.insert(*s);
                    }
                }
            }
        }

        // Objects of sh:property are property shapes
        for (_, ps) in get_triples_by_predicate(shapes_index, vocab.sh_property) {
            shape_nodes.insert(ps);
        }

        let mut results = Vec::new();

        for shape in shape_nodes {
            // Skip deactivated shapes
            if is_shape_deactivated(shapes_index, shape, &vocab) {
                continue;
            }
            let focus_nodes = get_focus_nodes(data, shapes_index, shape, &vocab);
            for focus in focus_nodes {
                let mut visited = HashSet::new();
                validate_shape(data, shapes_index, &vocab, focus, shape, &mut results, &mut visited);
            }
        }

        let conforms = results.is_empty();
        ValidationReport { conforms, results }
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn is_shape_deactivated(shapes: &TripleIndex, shape_node: usize, vocab: &Vocab) -> bool {
    let deactivated_vals = get_objects(shapes, shape_node, vocab.sh_deactivated);
    for v in deactivated_vals {
        if let Some(lex) = get_lexical_form(v) {
            if lex == "true" || lex == "1" {
                return true;
            }
        }
    }
    false
}

fn is_blank_node(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::BlankNode(_)))
}

fn is_iri(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::Iri(_)))
}

fn is_literal(id: usize) -> bool {
    matches!(Encoder::decode_to_term(id), Some(Term::Literal(_)))
}

fn contains_triple(index: &TripleIndex, s: usize, p: usize, o: usize) -> bool {
    if let Some(preds) = index.spo.get(&s) {
        if let Some(objs) = preds.get(&p) {
            return objs.iter().any(|(obj_val, _, _)| *obj_val == o);
        }
    }
    false
}

fn get_objects(index: &TripleIndex, subject: usize, predicate: usize) -> Vec<usize> {
    let mut result = Vec::new();
    if let Some(preds) = index.spo.get(&subject) {
        if let Some(objs) = preds.get(&predicate) {
            for (o, _, _) in objs {
                result.push(*o);
            }
        }
    }
    result
}

fn get_subjects(index: &TripleIndex, predicate: usize, object: usize) -> Vec<usize> {
    let mut result = Vec::new();
    if let Some(objs) = index.pos.get(&predicate) {
        if let Some(subjs) = objs.get(&object) {
            for (s, _, _) in subjs {
                result.push(*s);
            }
        }
    }
    result
}

fn get_triples_by_predicate(index: &TripleIndex, predicate: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    if let Some(objs) = index.pos.get(&predicate) {
        for (o, subjs) in objs {
            for (s, _, _) in subjs {
                result.push((*s, *o));
            }
        }
    }
    result
}

fn get_rdf_list(shapes: &TripleIndex, list_node: usize) -> Vec<usize> {
    let mut result = Vec::new();
    let mut current = list_node;
    let nil_node = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#nil>");
    let first_pred = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>").unwrap_or(0);
    let rest_pred = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#rest>").unwrap_or(0);
    let mut visited = HashSet::new();
    while Some(current) != nil_node && visited.insert(current) {
        let firsts = get_objects(shapes, current, first_pred);
        if firsts.is_empty() { break; }
        result.push(firsts[0]);
        let rests = get_objects(shapes, current, rest_pred);
        if rests.is_empty() { break; }
        current = rests[0];
    }
    result
}

fn get_datatype(term_id: usize) -> Option<usize> {
    let term = Encoder::decode_to_term(term_id)?;
    if let Term::Literal(lit) = term {
        if let Some(dt) = lit.datatype {
            Some(dt)
        } else if lit.lang.is_some() {
            Some(Encoder::add("<http://www.w3.org/1999/02/22-rdf-syntax-ns#langString>".to_string()))
        } else {
            Some(Encoder::add("<http://www.w3.org/2001/XMLSchema#string>".to_string()))
        }
    } else {
        None
    }
}

fn check_datatype(x: usize, expected_dt: usize) -> bool {
    get_datatype(x).map_or(false, |dt| dt == expected_dt)
}

fn has_class(data: &TripleIndex, x: usize, class: usize, rdfs_subclass_of: usize, rdf_type: usize) -> bool {
    let types = get_objects(data, x, rdf_type);
    for t in types {
        let mut visited = HashSet::new();
        let mut queue = vec![t];
        while !queue.is_empty() {
            let current = queue.remove(0);
            if current == class { return true; }
            if visited.insert(current) {
                for p in get_objects(data, current, rdfs_subclass_of) {
                    queue.push(p);
                }
            }
        }
    }
    false
}

fn get_lexical_form(x: usize) -> Option<String> {
    let term = Encoder::decode_to_term(x)?;
    match term {
        Term::Iri(_) => {
            let s = Encoder::decode(&x)?;
            if s.starts_with('<') && s.ends_with('>') {
                Some(s[1..s.len()-1].to_string())
            } else {
                Some(s)
            }
        }
        Term::Literal(lit) => Encoder::decode(&lit.value),
        Term::BlankNode(_) => {
            let s = Encoder::decode(&x)?;
            Some(if s.starts_with("_:") { s[2..].to_string() } else { s })
        }
    }
}

fn get_lang_tag(x: usize) -> Option<String> {
    if let Some(Term::Literal(lit)) = Encoder::decode_to_term(x) {
        lit.lang.as_ref().and_then(|l| Encoder::decode(l))
    } else {
        None
    }
}

/// Return the numeric value of a literal as f64, handling xsd:integer, xsd:decimal, xsd:double.
fn get_numeric_value(term_id: usize) -> Option<f64> {
    let lex = get_lexical_form(term_id)?;
    lex.trim().parse::<f64>().ok()
}

/// Compare two terms numerically (for sh:lessThan etc.). Returns None if not comparable.
fn compare_numeric(a: usize, b: usize) -> Option<std::cmp::Ordering> {
    let av = get_numeric_value(a)?;
    let bv = get_numeric_value(b)?;
    av.partial_cmp(&bv)
}

fn match_regex(pattern: &str, text: &str, flags: &str) -> bool {
    let mut builder = regex::RegexBuilder::new(pattern);
    for c in flags.chars() {
        match c {
            'i' => { builder.case_insensitive(true); }
            'm' => { builder.multi_line(true); }
            's' => { builder.dot_matches_new_line(true); }
            'x' => { builder.ignore_whitespace(true); }
            _ => {}
        }
    }
    builder.build().map_or(false, |re| re.is_match(text))
}

/// Parse an xsd:integer literal as i64 (allowing negative values).
fn get_integer_value(term_id: usize) -> Option<i64> {
    let lex = get_lexical_form(term_id)?;
    lex.trim().parse::<i64>().ok()
}

fn decode_to_term(id: usize) -> Term {
    Encoder::decode_to_term(id).unwrap_or_else(|| Term::Iri(TermImpl { iri: id }))
}

fn get_severity(shapes: &TripleIndex, shape_node: usize, vocab: &Vocab) -> usize {
    let sevs = get_objects(shapes, shape_node, vocab.sh_severity);
    if sevs.is_empty() { vocab.sh_violation } else { sevs[0] }
}

fn get_shape_messages(shapes: &TripleIndex, shape_node: usize, vocab: &Vocab) -> Vec<String> {
    get_objects(shapes, shape_node, vocab.sh_message)
        .into_iter()
        .filter_map(|m| get_lexical_form(m))
        .collect()
}

fn make_result(
    focus_node: usize,
    result_path: Option<usize>,
    value: Option<usize>,
    component: usize,
    shape_node: usize,
    severity: usize,
    message: Option<String>,
) -> ValidationResult {
    ValidationResult {
        focus_node: decode_to_term(focus_node),
        result_path: result_path.map(decode_to_term),
        value: value.map(decode_to_term),
        source_constraint_component: decode_to_term(component),
        source_shape: decode_to_term(shape_node),
        severity: decode_to_term(severity),
        message,
    }
}

fn get_focus_nodes(
    data: &TripleIndex,
    shapes: &TripleIndex,
    shape_id: usize,
    vocab: &Vocab,
) -> HashSet<usize> {
    let mut focus_nodes = HashSet::new();

    // 1. sh:targetClass
    for class in get_objects(shapes, shape_id, vocab.sh_target_class) {
        for inst in get_subjects(data, vocab.rdf_type, class) {
            focus_nodes.insert(inst);
        }
    }

    // 2. sh:targetNode
    for node in get_objects(shapes, shape_id, vocab.sh_target_node) {
        focus_nodes.insert(node);
    }

    // 3. sh:targetSubjectsOf
    for pred in get_objects(shapes, shape_id, vocab.sh_target_subjects_of) {
        if let Some(objs) = data.pos.get(&pred) {
            for subjs in objs.values() {
                for (s, _, _) in subjs { focus_nodes.insert(*s); }
            }
        }
    }

    // 4. sh:targetObjectsOf
    for pred in get_objects(shapes, shape_id, vocab.sh_target_objects_of) {
        if let Some(objs) = data.pos.get(&pred) {
            for o in objs.keys() { focus_nodes.insert(*o); }
        }
    }

    // Implicit class target: only when shape is also declared as a class AND has no other targets
    if !is_blank_node(shape_id) {
        let has_explicit_target =
            !get_objects(shapes, shape_id, vocab.sh_target_class).is_empty()
            || !get_objects(shapes, shape_id, vocab.sh_target_node).is_empty()
            || !get_objects(shapes, shape_id, vocab.sh_target_subjects_of).is_empty()
            || !get_objects(shapes, shape_id, vocab.sh_target_objects_of).is_empty();

        if !has_explicit_target {
            let is_class =
                contains_triple(shapes, shape_id, vocab.rdf_type, vocab.rdfs_class)
                || contains_triple(data, shape_id, vocab.rdf_type, vocab.rdfs_class);
            if is_class {
                for inst in get_subjects(data, vocab.rdf_type, shape_id) {
                    focus_nodes.insert(inst);
                }
            }
        }
    }

    focus_nodes
}

fn eval_path(
    data: &TripleIndex,
    shapes: &TripleIndex,
    focus_node: usize,
    path_node: usize,
) -> Vec<usize> {
    // Sequence path (RDF list of steps)
    let rdf_first = Encoder::get("<http://www.w3.org/1999/02/22-rdf-syntax-ns#first>").unwrap_or(0);
    if !get_objects(shapes, path_node, rdf_first).is_empty() {
        let steps = get_rdf_list(shapes, path_node);
        let mut current = vec![focus_node];
        for step in steps {
            let mut next = Vec::new();
            for node in current {
                next.extend(eval_path(data, shapes, node, step));
            }
            current = next;
            if current.is_empty() { break; }
        }
        return current;
    }

    // sh:inversePath
    let sh_inverse_path = Encoder::get("<http://www.w3.org/ns/shacl#inversePath>").unwrap_or(0);
    let inverse = get_objects(shapes, path_node, sh_inverse_path);
    if !inverse.is_empty() {
        let mut results = Vec::new();
        for inv in inverse {
            results.extend(get_subjects(data, inv, focus_node));
        }
        return results;
    }

    // sh:alternativePath
    let sh_alternative_path = Encoder::get("<http://www.w3.org/ns/shacl#alternativePath>").unwrap_or(0);
    let alternative = get_objects(shapes, path_node, sh_alternative_path);
    if !alternative.is_empty() {
        let mut results = Vec::new();
        for alt_list in alternative {
            for p in get_rdf_list(shapes, alt_list) {
                results.extend(eval_path(data, shapes, focus_node, p));
            }
        }
        return results;
    }

    // sh:zeroOrMorePath
    let sh_zero_or_more = Encoder::get("<http://www.w3.org/ns/shacl#zeroOrMorePath>").unwrap_or(0);
    let zero_or_more = get_objects(shapes, path_node, sh_zero_or_more);
    if !zero_or_more.is_empty() {
        let mut results = vec![focus_node];
        let mut visited = HashSet::new();
        visited.insert(focus_node);
        let mut queue = vec![focus_node];
        while !queue.is_empty() {
            let node = queue.remove(0);
            for target in &zero_or_more {
                for n in eval_path(data, shapes, node, *target) {
                    if visited.insert(n) { results.push(n); queue.push(n); }
                }
            }
        }
        return results;
    }

    // sh:oneOrMorePath
    let sh_one_or_more = Encoder::get("<http://www.w3.org/ns/shacl#oneOrMorePath>").unwrap_or(0);
    let one_or_more = get_objects(shapes, path_node, sh_one_or_more);
    if !one_or_more.is_empty() {
        let mut results = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = vec![focus_node];
        while !queue.is_empty() {
            let node = queue.remove(0);
            for target in &one_or_more {
                for n in eval_path(data, shapes, node, *target) {
                    if visited.insert(n) { results.push(n); queue.push(n); }
                }
            }
        }
        return results;
    }

    // sh:zeroOrOnePath
    let sh_zero_or_one = Encoder::get("<http://www.w3.org/ns/shacl#zeroOrOnePath>").unwrap_or(0);
    let zero_or_one = get_objects(shapes, path_node, sh_zero_or_one);
    if !zero_or_one.is_empty() {
        let mut results = vec![focus_node];
        for target in zero_or_one {
            for n in eval_path(data, shapes, focus_node, target) {
                if !results.contains(&n) { results.push(n); }
            }
        }
        return results;
    }

    // Direct IRI property
    get_objects(data, focus_node, path_node)
}

/// Returns true if the node conforms to the given shape (no violations).
fn conforms_to_shape(
    data: &TripleIndex,
    shapes: &TripleIndex,
    vocab: &Vocab,
    node: usize,
    shape_node: usize,
    visited: &mut HashSet<(usize, usize)>,
) -> bool {
    let mut temp = Vec::new();
    validate_shape(data, shapes, vocab, node, shape_node, &mut temp, visited);
    temp.is_empty()
}

fn validate_shape(
    data: &TripleIndex,
    shapes: &TripleIndex,
    vocab: &Vocab,
    focus_node: usize,
    shape_node: usize,
    results: &mut Vec<ValidationResult>,
    visited: &mut HashSet<(usize, usize)>,
) {
    // Cycle detection: if we are already validating (focus, shape), skip.
    if !visited.insert((focus_node, shape_node)) {
        return;
    }

    // Deactivated check
    if is_shape_deactivated(shapes, shape_node, vocab) {
        visited.remove(&(focus_node, shape_node));
        return;
    }

    let severity = get_severity(shapes, shape_node, vocab);
    let messages = get_shape_messages(shapes, shape_node, vocab);
    let default_msg = messages.first().cloned();

    // -----------------------------------------------------------------------
    // Node-level constraints
    // -----------------------------------------------------------------------

    // sh:nodeKind
    for nk in get_objects(shapes, shape_node, vocab.sh_node_kind) {
        let ok = if nk == vocab.sh_iri {
            is_iri(focus_node)
        } else if nk == vocab.sh_blank_node {
            is_blank_node(focus_node)
        } else if nk == vocab.sh_literal {
            is_literal(focus_node)
        } else if nk == vocab.sh_blank_node_or_iri {
            is_blank_node(focus_node) || is_iri(focus_node)
        } else if nk == vocab.sh_blank_node_or_literal {
            is_blank_node(focus_node) || is_literal(focus_node)
        } else if nk == vocab.sh_iri_or_literal {
            is_iri(focus_node) || is_literal(focus_node)
        } else {
            true
        };
        if !ok {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_node_kind_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:datatype (node-level)
    for dt in get_objects(shapes, shape_node, vocab.sh_datatype) {
        if !check_datatype(focus_node, dt) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_datatype_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:class (node-level)
    for class in get_objects(shapes, shape_node, vocab.sh_class) {
        if !has_class(data, focus_node, class, vocab.rdfs_subclass_of, vocab.rdf_type) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_class_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:hasValue (node-level)
    for hv in get_objects(shapes, shape_node, vocab.sh_has_value) {
        if focus_node != hv {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_has_value_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:in (node-level)
    for in_list in get_objects(shapes, shape_node, vocab.sh_in) {
        let allowed = get_rdf_list(shapes, in_list);
        if !allowed.contains(&focus_node) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_in_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:pattern + sh:flags (node-level)
    let flags_str = get_objects(shapes, shape_node, vocab.sh_flags)
        .first()
        .and_then(|f| get_lexical_form(*f))
        .unwrap_or_default();
    for pat in get_objects(shapes, shape_node, vocab.sh_pattern) {
        if let Some(pat_str) = get_lexical_form(pat) {
            let focus_str = get_lexical_form(focus_node).unwrap_or_default();
            if !match_regex(&pat_str, &focus_str, &flags_str) {
                results.push(make_result(focus_node, None, Some(focus_node),
                    vocab.sh_pattern_constraint_component, shape_node, severity, default_msg.clone()));
            }
        }
    }

    // sh:minLength / sh:maxLength (node-level)
    if let Some(lex) = get_lexical_form(focus_node) {
        let char_len = lex.chars().count();
        for ml in get_objects(shapes, shape_node, vocab.sh_min_length) {
            if let Some(v) = get_integer_value(ml) {
                if (char_len as i64) < v {
                    results.push(make_result(focus_node, None, Some(focus_node),
                        vocab.sh_min_length_constraint_component, shape_node, severity, default_msg.clone()));
                }
            }
        }
        for ml in get_objects(shapes, shape_node, vocab.sh_max_length) {
            if let Some(v) = get_integer_value(ml) {
                if (char_len as i64) > v {
                    results.push(make_result(focus_node, None, Some(focus_node),
                        vocab.sh_max_length_constraint_component, shape_node, severity, default_msg.clone()));
                }
            }
        }
    }

    // sh:minExclusive / sh:minInclusive / sh:maxExclusive / sh:maxInclusive (node-level)
    for bound in get_objects(shapes, shape_node, vocab.sh_min_exclusive) {
        match compare_numeric(focus_node, bound) {
            Some(std::cmp::Ordering::Greater) => {}
            _ => { results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_min_exclusive_constraint_component, shape_node, severity, default_msg.clone())); }
        }
    }
    for bound in get_objects(shapes, shape_node, vocab.sh_min_inclusive) {
        match compare_numeric(focus_node, bound) {
            Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {}
            _ => { results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_min_inclusive_constraint_component, shape_node, severity, default_msg.clone())); }
        }
    }
    for bound in get_objects(shapes, shape_node, vocab.sh_max_exclusive) {
        match compare_numeric(focus_node, bound) {
            Some(std::cmp::Ordering::Less) => {}
            _ => { results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_max_exclusive_constraint_component, shape_node, severity, default_msg.clone())); }
        }
    }
    for bound in get_objects(shapes, shape_node, vocab.sh_max_inclusive) {
        match compare_numeric(focus_node, bound) {
            Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {}
            _ => { results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_max_inclusive_constraint_component, shape_node, severity, default_msg.clone())); }
        }
    }

    // sh:and (node-level)
    for and_list in get_objects(shapes, shape_node, vocab.sh_and) {
        let sub_shapes = get_rdf_list(shapes, and_list);
        let mut sub_results = Vec::new();
        for sub_shape in sub_shapes {
            validate_shape(data, shapes, vocab, focus_node, sub_shape, &mut sub_results, visited);
        }
        if !sub_results.is_empty() {
            // Propagate sub-results and add a top-level and-violation
            results.extend(sub_results);
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_and_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:or (node-level)
    for or_list in get_objects(shapes, shape_node, vocab.sh_or) {
        let sub_shapes = get_rdf_list(shapes, or_list);
        let conforms = sub_shapes.iter().any(|&sub| {
            conforms_to_shape(data, shapes, vocab, focus_node, sub, visited)
        });
        if !conforms {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_or_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:xone (node-level) — exactly one sub-shape must conform
    for xone_list in get_objects(shapes, shape_node, vocab.sh_xone) {
        let sub_shapes = get_rdf_list(shapes, xone_list);
        let count = sub_shapes.iter()
            .filter(|&&sub| conforms_to_shape(data, shapes, vocab, focus_node, sub, visited))
            .count();
        if count != 1 {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_xone_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:not (node-level)
    for not_shape in get_objects(shapes, shape_node, vocab.sh_not) {
        if conforms_to_shape(data, shapes, vocab, focus_node, not_shape, visited) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_not_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // sh:node (node-level) — the focus node itself must conform to the referenced shape
    for node_shape in get_objects(shapes, shape_node, vocab.sh_node) {
        if !conforms_to_shape(data, shapes, vocab, focus_node, node_shape, visited) {
            results.push(make_result(focus_node, None, Some(focus_node),
                vocab.sh_node_constraint_component, shape_node, severity, default_msg.clone()));
        }
    }

    // -----------------------------------------------------------------------
    // sh:property — property shape constraints
    // -----------------------------------------------------------------------
    for ps in get_objects(shapes, shape_node, vocab.sh_property) {
        let paths = get_objects(shapes, ps, vocab.sh_path);
        if paths.is_empty() { continue; }
        let path = paths[0];
        let v_nodes = eval_path(data, shapes, focus_node, path);
        let ps_severity = get_severity(shapes, ps, vocab);
        let ps_messages = get_shape_messages(shapes, ps, vocab);
        let ps_msg = ps_messages.first().cloned();

        // sh:minCount
        for mc in get_objects(shapes, ps, vocab.sh_min_count) {
            if let Some(mc_val) = get_integer_value(mc) {
                if (v_nodes.len() as i64) < mc_val {
                    results.push(make_result(focus_node, Some(path), None,
                        vocab.sh_min_count_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:maxCount
        for mc in get_objects(shapes, ps, vocab.sh_max_count) {
            if let Some(mc_val) = get_integer_value(mc) {
                if (v_nodes.len() as i64) > mc_val {
                    results.push(make_result(focus_node, Some(path), None,
                        vocab.sh_max_count_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:datatype (per-value)
        for dt in get_objects(shapes, ps, vocab.sh_datatype) {
            for &v in &v_nodes {
                if !check_datatype(v, dt) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_datatype_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:nodeKind (per-value)
        for nk in get_objects(shapes, ps, vocab.sh_node_kind) {
            for &v in &v_nodes {
                let ok = if nk == vocab.sh_iri { is_iri(v) }
                    else if nk == vocab.sh_blank_node { is_blank_node(v) }
                    else if nk == vocab.sh_literal { is_literal(v) }
                    else if nk == vocab.sh_blank_node_or_iri { is_blank_node(v) || is_iri(v) }
                    else if nk == vocab.sh_blank_node_or_literal { is_blank_node(v) || is_literal(v) }
                    else if nk == vocab.sh_iri_or_literal { is_iri(v) || is_literal(v) }
                    else { true };
                if !ok {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_node_kind_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:class (per-value)
        for class in get_objects(shapes, ps, vocab.sh_class) {
            for &v in &v_nodes {
                if !has_class(data, v, class, vocab.rdfs_subclass_of, vocab.rdf_type) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_class_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:hasValue (per-property — value must appear at least once)
        for hv in get_objects(shapes, ps, vocab.sh_has_value) {
            if !v_nodes.contains(&hv) {
                results.push(make_result(focus_node, Some(path), None,
                    vocab.sh_has_value_constraint_component, ps, ps_severity, ps_msg.clone()));
            }
        }

        // sh:in (per-value)
        for in_list in get_objects(shapes, ps, vocab.sh_in) {
            let allowed = get_rdf_list(shapes, in_list);
            for &v in &v_nodes {
                if !allowed.contains(&v) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_in_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:pattern + sh:flags (per-value)
        let ps_flags = get_objects(shapes, ps, vocab.sh_flags)
            .first()
            .and_then(|f| get_lexical_form(*f))
            .unwrap_or_default();
        for pat in get_objects(shapes, ps, vocab.sh_pattern) {
            if let Some(pat_str) = get_lexical_form(pat) {
                for &v in &v_nodes {
                    let v_str = get_lexical_form(v).unwrap_or_default();
                    if !match_regex(&pat_str, &v_str, &ps_flags) {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_pattern_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }

        // sh:minLength / sh:maxLength (per-value)
        for ml in get_objects(shapes, ps, vocab.sh_min_length) {
            if let Some(min) = get_integer_value(ml) {
                for &v in &v_nodes {
                    let len = get_lexical_form(v).map(|s| s.chars().count() as i64).unwrap_or(0);
                    if len < min {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_min_length_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }
        for ml in get_objects(shapes, ps, vocab.sh_max_length) {
            if let Some(max) = get_integer_value(ml) {
                for &v in &v_nodes {
                    let len = get_lexical_form(v).map(|s| s.chars().count() as i64).unwrap_or(0);
                    if len > max {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_max_length_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }

        // Numeric range constraints (per-value)
        for bound in get_objects(shapes, ps, vocab.sh_min_exclusive) {
            for &v in &v_nodes {
                match compare_numeric(v, bound) {
                    Some(std::cmp::Ordering::Greater) => {}
                    _ => { results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_min_exclusive_constraint_component, ps, ps_severity, ps_msg.clone())); }
                }
            }
        }
        for bound in get_objects(shapes, ps, vocab.sh_min_inclusive) {
            for &v in &v_nodes {
                match compare_numeric(v, bound) {
                    Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {}
                    _ => { results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_min_inclusive_constraint_component, ps, ps_severity, ps_msg.clone())); }
                }
            }
        }
        for bound in get_objects(shapes, ps, vocab.sh_max_exclusive) {
            for &v in &v_nodes {
                match compare_numeric(v, bound) {
                    Some(std::cmp::Ordering::Less) => {}
                    _ => { results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_max_exclusive_constraint_component, ps, ps_severity, ps_msg.clone())); }
                }
            }
        }
        for bound in get_objects(shapes, ps, vocab.sh_max_inclusive) {
            for &v in &v_nodes {
                match compare_numeric(v, bound) {
                    Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {}
                    _ => { results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_max_inclusive_constraint_component, ps, ps_severity, ps_msg.clone())); }
                }
            }
        }

        // sh:languageIn (per-value)
        for lang_list in get_objects(shapes, ps, vocab.sh_language_in) {
            let allowed_langs: Vec<String> = get_rdf_list(shapes, lang_list)
                .into_iter()
                .filter_map(|l| get_lexical_form(l))
                .map(|s| s.to_lowercase())
                .collect();
            for &v in &v_nodes {
                if let Some(tag) = get_lang_tag(v) {
                    if !allowed_langs.iter().any(|al| tag.to_lowercase().starts_with(al.as_str())) {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_language_in_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                } else {
                    // Value has no language tag — violates sh:languageIn
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_language_in_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:uniqueLang (per-property)
        let unique_lang_vals = get_objects(shapes, ps, vocab.sh_unique_lang);
        for ul in unique_lang_vals {
            if let Some(lex) = get_lexical_form(ul) {
                if lex == "true" || lex == "1" {
                    let mut seen_langs: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                    for &v in &v_nodes {
                        if let Some(tag) = get_lang_tag(v) {
                            let lower = tag.to_lowercase();
                            if let Some(&prev) = seen_langs.get(&lower) {
                                // Duplicate language tag
                                results.push(make_result(focus_node, Some(path), Some(v),
                                    vocab.sh_unique_lang_constraint_component, ps, ps_severity, ps_msg.clone()));
                                // Also report the first duplicate if not already reported
                                let _ = prev;
                            } else {
                                seen_langs.insert(lower, v);
                            }
                        }
                    }
                }
            }
        }

        // sh:equals (values must equal values for the given path)
        for eq_prop in get_objects(shapes, ps, vocab.sh_equals) {
            let other_values: HashSet<usize> = eval_path(data, shapes, focus_node, eq_prop).into_iter().collect();
            let self_values: HashSet<usize> = v_nodes.iter().cloned().collect();
            if self_values != other_values {
                for &v in &v_nodes {
                    if !other_values.contains(&v) {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_equals_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
                for &v in &other_values {
                    if !self_values.contains(&v) {
                        results.push(make_result(focus_node, Some(path), Some(v),
                            vocab.sh_equals_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }

        // sh:disjoint (no overlap between path values and the other property's values)
        for disj_prop in get_objects(shapes, ps, vocab.sh_disjoint) {
            let other_values: HashSet<usize> = eval_path(data, shapes, focus_node, disj_prop).into_iter().collect();
            for &v in &v_nodes {
                if other_values.contains(&v) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_disjoint_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:lessThan
        for lt_prop in get_objects(shapes, ps, vocab.sh_less_than) {
            let other_values: Vec<usize> = eval_path(data, shapes, focus_node, lt_prop);
            for &v in &v_nodes {
                for &ov in &other_values {
                    match compare_numeric(v, ov) {
                        Some(std::cmp::Ordering::Less) => {}
                        _ => {
                            results.push(make_result(focus_node, Some(path), Some(v),
                                vocab.sh_less_than_constraint_component, ps, ps_severity, ps_msg.clone()));
                        }
                    }
                }
            }
        }

        // sh:lessThanOrEquals
        for lte_prop in get_objects(shapes, ps, vocab.sh_less_than_or_equals) {
            let other_values: Vec<usize> = eval_path(data, shapes, focus_node, lte_prop);
            for &v in &v_nodes {
                for &ov in &other_values {
                    match compare_numeric(v, ov) {
                        Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {}
                        _ => {
                            results.push(make_result(focus_node, Some(path), Some(v),
                                vocab.sh_less_than_or_equals_constraint_component, ps, ps_severity, ps_msg.clone()));
                        }
                    }
                }
            }
        }

        // sh:qualifiedValueShape / sh:qualifiedMinCount / sh:qualifiedMaxCount
        let qvs_list = get_objects(shapes, ps, vocab.sh_qualified_value_shape);
        if !qvs_list.is_empty() {
            let qvs = qvs_list[0];
            let conforming_count = v_nodes.iter()
                .filter(|&&v| conforms_to_shape(data, shapes, vocab, v, qvs, visited))
                .count() as i64;
            for qmin in get_objects(shapes, ps, vocab.sh_qualified_min_count) {
                if let Some(min) = get_integer_value(qmin) {
                    if conforming_count < min {
                        results.push(make_result(focus_node, Some(path), None,
                            vocab.sh_qualified_value_shape_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
            for qmax in get_objects(shapes, ps, vocab.sh_qualified_max_count) {
                if let Some(max) = get_integer_value(qmax) {
                    if conforming_count > max {
                        results.push(make_result(focus_node, Some(path), None,
                            vocab.sh_qualified_value_shape_constraint_component, ps, ps_severity, ps_msg.clone()));
                    }
                }
            }
        }

        // Logical constraints on property shape values
        // sh:and per-value
        for and_list in get_objects(shapes, ps, vocab.sh_and) {
            let sub_shapes = get_rdf_list(shapes, and_list);
            for &v in &v_nodes {
                let mut sub_results = Vec::new();
                for &sub in &sub_shapes {
                    validate_shape(data, shapes, vocab, v, sub, &mut sub_results, visited);
                }
                if !sub_results.is_empty() {
                    results.extend(sub_results);
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_and_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:or per-value
        for or_list in get_objects(shapes, ps, vocab.sh_or) {
            let sub_shapes = get_rdf_list(shapes, or_list);
            for &v in &v_nodes {
                let conforms = sub_shapes.iter().any(|&sub|
                    conforms_to_shape(data, shapes, vocab, v, sub, visited));
                if !conforms {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_or_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:xone per-value
        for xone_list in get_objects(shapes, ps, vocab.sh_xone) {
            let sub_shapes = get_rdf_list(shapes, xone_list);
            for &v in &v_nodes {
                let count = sub_shapes.iter()
                    .filter(|&&sub| conforms_to_shape(data, shapes, vocab, v, sub, visited))
                    .count();
                if count != 1 {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_xone_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:not per-value
        for not_shape in get_objects(shapes, ps, vocab.sh_not) {
            for &v in &v_nodes {
                if conforms_to_shape(data, shapes, vocab, v, not_shape, visited) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_not_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }

        // sh:node per-value
        for node_shape in get_objects(shapes, ps, vocab.sh_node) {
            for &v in &v_nodes {
                if !conforms_to_shape(data, shapes, vocab, v, node_shape, visited) {
                    results.push(make_result(focus_node, Some(path), Some(v),
                        vocab.sh_node_constraint_component, ps, ps_severity, ps_msg.clone()));
                }
            }
        }
    }

    // sh:closed / sh:ignoredProperties (node-level)
    let closed_vals = get_objects(shapes, shape_node, vocab.sh_closed);
    for cv in closed_vals {
        if let Some(lex) = get_lexical_form(cv) {
            if lex == "true" || lex == "1" {
                // Collect all explicitly allowed predicates from sh:property sh:path
                let mut allowed_preds: HashSet<usize> = HashSet::new();
                for ps in get_objects(shapes, shape_node, vocab.sh_property) {
                    for p in get_objects(shapes, ps, vocab.sh_path) {
                        allowed_preds.insert(p);
                    }
                }
                // Add ignored properties
                for ig_list in get_objects(shapes, shape_node, vocab.sh_ignored_properties) {
                    for ig in get_rdf_list(shapes, ig_list) {
                        allowed_preds.insert(ig);
                    }
                }
                // Check that all actual predicates on the focus node are in allowed set
                if let Some(preds) = data.spo.get(&focus_node) {
                    for &pred in preds.keys() {
                        if !allowed_preds.contains(&pred) {
                            results.push(make_result(focus_node, Some(pred), None,
                                vocab.sh_closed_constraint_component, shape_node, severity, default_msg.clone()));
                        }
                    }
                }
            }
        }
    }

    // Remove from visited so the same (focus, shape) pair can be re-checked from
    // a different call path (e.g., inside sh:and or sh:or sub-checks). This is
    // correct because the visited set is only used to prevent infinite recursion,
    // not to cache results.
    visited.remove(&(focus_node, shape_node));
}
