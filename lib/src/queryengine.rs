use crate::{Binding, BodyLiteral, Encoder, Triple, TripleIndex, VarOrTerm};
use std::rc::Rc;

pub trait QueryEngine {
    fn query(
        data: &TripleIndex,
        query_triples: &Vec<BodyLiteral>,
        triple_counter: Option<usize>,
    ) -> Option<Binding>;
}
pub struct SimpleQueryEngine;

impl QueryEngine for SimpleQueryEngine {
    fn query(
        data: &TripleIndex,
        query_triples: &Vec<BodyLiteral>,
        triple_counter: Option<usize>,
    ) -> Option<Binding> {
        let positive_lits: Vec<&BodyLiteral> = query_triples.iter().filter(|lit| !lit.negated).collect();
        let negated_lits: Vec<&BodyLiteral> = query_triples.iter().filter(|lit| lit.negated).collect();

        let mut bindings = Binding::new();
        let mut first = true;
        for lit in &positive_lits {
            if let Some(current_bindings) = data.query(&lit.pattern, None) {
                if first {
                    bindings = current_bindings;
                    first = false;
                } else {
                    let joined = bindings.join(&current_bindings);
                    if joined.len() == 0 && bindings.len() > 0 && current_bindings.len() > 0 {
                        return None;
                    }
                    bindings = joined;
                }
            } else {
                return None;
            }
        }

        if negated_lits.is_empty() {
            return Some(bindings);
        }

        let mut filtered_bindings = Binding::new();
        let num_rows = if positive_lits.is_empty() { 1 } else { bindings.len() };

        for c in 0..num_rows {
            let mut satisfied = true;
            for negated_lit in &negated_lits {
                let mut ground_pattern = negated_lit.pattern.clone();
                
                let mut substitute = |term: &mut VarOrTerm| {
                    if term.is_var() {
                        let var_id = term.to_encoded();
                        if let Some(vals) = bindings.get(&var_id) {
                            if let Some(&val) = vals.get(c) {
                                *term = VarOrTerm::new_term(Encoder::decode(&val).unwrap());
                            }
                        }
                    }
                };
                
                substitute(&mut ground_pattern.s);
                substitute(&mut ground_pattern.p);
                substitute(&mut ground_pattern.o);
                if let Some(ref mut g) = ground_pattern.g {
                    substitute(g);
                }

                if data.query(&ground_pattern, triple_counter).is_some() {
                    satisfied = false;
                    break;
                }
            }

            if satisfied {
                if !positive_lits.is_empty() {
                    for (&var_id, vals) in bindings.iter() {
                        filtered_bindings.add(&var_id, vals[c]);
                    }
                }
            }
        }

        if filtered_bindings.len() > 0 || (positive_lits.is_empty() && filtered_bindings.len() == 0 && num_rows == 1) {
            Some(filtered_bindings)
        } else {
            None
        }
    }
}
// pub fn query(&self, query_triple:&Triple, triple_counter : Option<usize>) -> Binding{
//     let mut bindings = Binding::new();
//     let mut counter = if let Some(size) = triple_counter{size} else {self.triple_index.len()};
//     for Triple{s,p,o} in self.triple_index.triples.iter().take(counter){
//         match &query_triple.s{
//             VarOrTerm::Var(s_var)=> bindings.add(&s_var.name,s.as_Term().iri),
//             VarOrTerm::Term(s_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (s_term,s.as_Term()) {
//                 if !iri.eq(iri2){break;}
//             }
//         }
//         match &query_triple.p{
//             VarOrTerm::Var(p_var)=> bindings.add(&p_var.name,p.as_Term().iri),
//             VarOrTerm::Term(p_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (p_term,p.as_Term()) {
//                 if !iri.eq(iri2){break;}
//             }
//         }
//         match &query_triple.o{
//             VarOrTerm::Var(o_var)=> bindings.add(&o_var.name,o.as_Term().iri),
//             VarOrTerm::Term(o_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (o_term,o.as_Term()) {
//                 if !iri.eq(iri2){break;}
//             }
//         }
//     }
//     bindings
// }
// fn find_matching_rules(&self, triple: &Triple) -> Vec<&Rule> {
//     let mut matching_rules = Vec::new();
//     for rule in self.rules.iter(){
//         for body_item in rule.body.iter(){
//             if let Triple{s,p,o} = triple{
//                 match &body_item.s{
//                     VarOrTerm::Term(s_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (s_term,s.as_Term()) {
//                         if !iri.eq(iri2){break;}
//                     },
//                     _ => ()
//                 }
//                 match &body_item.p{
//                     VarOrTerm::Term(p_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (p_term,p.as_Term()) {
//                         if !iri.eq(iri2){break;}
//                     },
//                     _ => ()
//                 }
//                 match &body_item.o{
//                     VarOrTerm::Term(o_term)=>if let (TermImpl{iri}, TermImpl{iri:iri2})= (o_term,o.as_Term()) {
//                         if !iri.eq(iri2){break;}
//                     },
//                     _ => ()
//                 }
//                 if !matching_rules.contains(&rule){
//                     matching_rules.push(rule);
//
//                 }
//             }
//         }
//     }
//     matching_rules
// }
