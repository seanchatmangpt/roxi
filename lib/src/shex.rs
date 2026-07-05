use crate::triples::Term;
use crate::tripleindex::TripleIndex;
use crate::oxrdf_adapter::{triple_index_to_oxrdf_graph, oxrdf_term_to_roxi_term};
use shex_ast::{ast::Schema as SchemaAST, ir::ast2ir::AST2IR, ResolveMethod};
use shex_ast::ir::{schema_ir::SchemaIR, semantic_actions_registry::SemanticActionsRegistry, map_state::MapState};
use shex_validation::{Validator, ValidatorConfig};
use rudof_rdf::rdf_impl::OxigraphInMemory;
use rudof_rdf::rdf_core::BuildRDF;

#[derive(Debug, Clone)]
pub struct ShexValidationReport {
    pub conforms: bool,
    pub failures: Vec<ShexValidationFailure>,
}

#[derive(Debug, Clone)]
pub struct ShexValidationFailure {
    pub node: Term, // Roxi native Term
    pub shape: String, // Shape label
    pub reason: String,
}

pub fn validate_shex(
    data: &TripleIndex,
    schema_json_str: &str,
    shape_map: &[(String, String)], // (Focus node IRI/string, Shape Label string)
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
        in_memory_rdf.add_triple(
            triple.subject,
            triple.predicate,
            triple.object,
        )?;
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
                let ox_term: oxrdf::Term = focus_node.as_object().clone().into();
                let native_node = oxrdf_term_to_roxi_term(&ox_term);
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
