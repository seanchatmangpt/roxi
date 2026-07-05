extern crate core;
pub mod backwardchaining;
pub mod bindings;
pub mod csprite;
pub mod dred;
pub mod encoding;
pub mod imars_reasoner;
pub mod imars_window;
pub mod observer;
pub mod parser;
pub mod pipeline;
pub mod queryengine;
pub mod reasoner;
pub mod rsp;
pub mod ruleindex;
mod service_composition;
pub mod sparql;
pub mod time_window;
pub mod tripleindex;
pub mod oxrdf_adapter;
pub mod triples;
pub mod utils;
pub mod aggregation;
pub mod shacl;
pub mod shex;
pub mod datalog;

extern crate pest;
#[macro_use]
extern crate pest_derive;
use crate::bindings::Binding;
use crate::ruleindex::RuleIndex;
use crate::tripleindex::TripleIndex;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::rc::Rc;

use log::{info, trace, warn}; // Use log crate when building application

use crate::backwardchaining::BackwardChainer;
use crate::encoding::Encoder;
use crate::parser::{Parser, Syntax};
use spargebra::Query;


use crate::queryengine::{QueryEngine, SimpleQueryEngine};
use crate::reasoner::Reasoner;
use crate::sparql::{eval_query, evaluate_plan_and_debug};
use crate::triples::{BlankNodeImpl, LiteralImpl, Rule, Term, TermImpl, Triple, VarOrTerm, BodyLiteral, Aggregate}; // Workaround to use prinltn! for logs.

pub struct TripleStore {
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex,
    pub triple_index: TripleIndex,
    pub reasoner: Reasoner,
    pub aggregates: HashMap<Rule, Aggregate>,
    pub strata: Vec<usize>,
}
unsafe impl Send for TripleStore {}

impl TripleStore {
    pub fn new() -> TripleStore {
        TripleStore {
            rules: Vec::new(),
            rules_index: RuleIndex::new(),
            triple_index: TripleIndex::new(),
            reasoner: Reasoner {},
            aggregates: HashMap::new(),
            strata: Vec::new(),
        }
    }
    pub fn from(data: &str) -> TripleStore {
        let (content, rules) = Parser::parse(data.to_string());
        let mut triple_index = TripleIndex::new();
        content.into_iter().for_each(|t| triple_index.add(t));
        let mut rules_index = RuleIndex::new();
        for rule in rules.iter() {
            rules_index.add_ref(rule);
        }
        let aggregates = HashMap::new();
        let mut strata = Vec::new();
        if let Ok(computed_strata) = datalog::validate_rules(&rules, &aggregates) {
            strata = computed_strata;
        }
        TripleStore {
            rules: rules,
            rules_index,
            triple_index,
            reasoner: Reasoner {},
            aggregates,
            strata,
        }
    }
    pub fn add(&mut self, triple: Triple) {
        trace! {"Adding triple: {:?}", Self::decode_triple(&triple) }
        self.triple_index.add(triple);
    }
    pub fn add_ref(&mut self, triple: Rc<Triple>) {
        trace! {"Adding triple: {:?}", Self::decode_triple(triple.as_ref()) }
        self.triple_index.add_ref(triple);
    }
    pub fn remove_ref(&mut self, triple: &Triple) {
        trace! {"Removing triple: {:?}", Self::decode_triple(triple) }
        self.triple_index.remove_ref(triple);
    }
    pub fn add_rules(&mut self, rules: Vec<Rule>) -> Result<(), String> {
        let mut all_rules = self.rules.clone();
        all_rules.extend(rules.clone());
        let strata = datalog::validate_rules(&all_rules, &self.aggregates)?;
        self.strata = strata;
        for rule in rules {
            self.rules.push(rule.clone());
            self.rules_index.add(rule);
        }
        Ok(())
    }
    pub fn add_rule_with_aggregate(&mut self, rule: Rule, aggregate: Aggregate) -> Result<(), String> {
        self.aggregates.insert(rule.clone(), aggregate);
        self.add_rules(vec![rule])
    }
    pub fn len(&self) -> usize {
        self.triple_index.len()
    }

    pub fn materialize(&mut self) -> Vec<Triple> {
        self.reasoner
            .materialize(&mut self.triple_index, &self.rules, &self.strata, &self.aggregates)
    }

    //Backward chaining

    ////

    pub fn decode_rule(rule: &Rule) -> String {
        let mut res = String::new();
        let decoded_head = Self::decode_triples(&[rule.head.clone()]);
        let decoded_body: String = rule
            .body
            .iter()
            .map(|lit| {
                let s = Self::decode_triples(&[lit.pattern.clone()]);
                if lit.negated {
                    format!("not {{{}}}", s.trim())
                } else {
                    s
                }
            })
            .collect();
        write!(&mut res, "{{{}}}=>{{{}}}.\n", decoded_body, decoded_head).unwrap();
        res
    }
    pub fn decode_rules(rules: &[Rule]) -> String {
        let mut res = String::new();
        for rule in rules {
            let decoded_head = Self::decode_triples(&[rule.head.clone()]);
            let decoded_body: String = rule
                .body
                .iter()
                .map(|lit| {
                    let s = Self::decode_triples(&[lit.pattern.clone()]);
                    if lit.negated {
                        format!("not {{{}}}", s.trim())
                    } else {
                        s
                    }
                })
                .collect();
            write!(&mut res, "{{{}}}=>{{{}}}.\n", decoded_body, decoded_head).unwrap();
        }
        res
    }
    pub fn decode_triples(triples: &[Triple]) -> String {
        let mut res = String::new();
        for triple in triples {
            let decoded_s = Encoder::decode(&triple.s.to_encoded()).unwrap();
            let decoded_p = Encoder::decode(&triple.p.to_encoded()).unwrap();
            let decoded_o = Encoder::decode(&triple.o.to_encoded()).unwrap();

            writeln!(&mut res, "{} {} {}.", decoded_s, decoded_p, decoded_o).unwrap();
        }
        res
    }
    pub fn decode_bindings(bindings: &Binding) -> String {
        let mut res = String::new();
        for (key, val) in bindings.iter() {
            let decoded_values: String = val.iter().map(|t| Encoder::decode(t).unwrap()).collect();

            writeln!(
                &mut res,
                " {}: [{}] .",
                Encoder::decode(key).unwrap(),
                decoded_values
            )
            .unwrap();
        }
        res
    }
    pub fn decode_triple(triple: &Triple) -> String {
        let s = Encoder::decode(&triple.s.to_encoded()).unwrap();
        let p = Encoder::decode(&triple.p.to_encoded()).unwrap();
        let o = Encoder::decode(&triple.o.to_encoded()).unwrap();
        format!("{} {} {}", s, p, o)
    }
    pub fn content_to_string(&self) -> String {
        let content = &self.triple_index.triples;
        TripleStore::decode_triples(content)
    }

    pub fn load_triples(&mut self, data: &str, syntax: Syntax) -> Result<(), String> {
        match Parser::parse_triples(data, syntax) {
            Ok(triples) => {
                triples.into_iter().for_each(|t| self.triple_index.add(t));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
    pub fn load_rules(&mut self, data: &str) -> Result<(), String> {
        match Parser::parse_rules(data) {
            Ok(rules) => self.add_rules(rules),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn query(&self, query_str: &str) -> Result<Vec<Vec<crate::sparql::Binding>>, String> {
        match Query::parse(query_str, None) {
            Ok(query) => {
                let plan = eval_query(&query, &self.triple_index);
                Ok(evaluate_plan_and_debug(&plan, &self.triple_index).collect())
            }
            Err(err) => Err(format!("Unable to parse Query: {:?}", err.to_string())),
        }
    }

    /// Validate the triples in this store against a SHACL shapes graph.
    ///
    /// `shapes_turtle` is a Turtle-serialised SHACL shapes graph.
    /// Returns a `ValidationReport` describing conformance and any violations.
    pub fn validate_shacl(&self, shapes_turtle: &str) -> Result<crate::shacl::ValidationReport, String> {
        let shapes = crate::shacl::ShapesGraph::parse(shapes_turtle)?;
        Ok(crate::shacl::Validator::validate(&self.triple_index, &shapes))
    }

    /// Validate specific (focus-node, shape) pairs against a ShEx schema.
    ///
    /// `schema_json` is the ShExJ (JSON) serialisation of the ShEx schema.
    /// `shape_map` is a slice of `(focus_node_iri, shape_label_iri)` pairs.
    /// Returns a `ShexValidationReport` describing conformance.
    pub fn validate_shex(
        &self,
        schema_json: &str,
        shape_map: &[(String, String)],
    ) -> Result<crate::shex::ShexValidationReport, Box<dyn std::error::Error>> {
        crate::shex::validate_shex(&self.triple_index, schema_json, shape_map)
    }
}

mod lib_test;
