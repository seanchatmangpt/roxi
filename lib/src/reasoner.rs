use crate::triples::{Aggregate, AggregateFunction};
use crate::aggregation::{Accumulator, CountAccumulator, SumAccumulator, MinAccumulator, MaxAccumulator, AvgAccumulator, AccumulatorImpl};
use std::collections::HashMap;
use crate::{
    Binding, BodyLiteral, Encoder, Parser, QueryEngine, Rule, RuleIndex, SimpleQueryEngine, Term,
    Triple, TripleIndex, TripleStore, VarOrTerm,
};
use log::{debug, info, trace, warn}; // Use log crate when building application
use std::fmt::Write;

use crate::imars_window::ImarsWindow;
use std::cell::RefCell;
use std::rc::Rc;


pub struct Reasoner;

impl Reasoner {
    pub fn materialize(
        &mut self,
        triple_index: &mut TripleIndex,
        rules: &Vec<Rule>,
        strata: &Vec<usize>,
        aggregates: &std::collections::HashMap<Rule, crate::triples::Aggregate>,
    ) -> Vec<Triple> {
        let mut inferred = Vec::new();
        if rules.is_empty() {
            return inferred;
        }

        let max_stratum = *strata.iter().max().unwrap_or(&0);
        
        for s in 0..=max_stratum {
            
            let stratum_rules: Vec<&Rule> = rules
                .iter()
                .enumerate()
                .filter(|(i, _)| strata[*i] == s)
                .map(|(_, r)| r)
                .collect();

            if stratum_rules.is_empty() {
                continue;
            }

            let mut stratum_start_counter = None;
            let mut changed = true;

            while changed {
                changed = false;
                let next_start_counter = triple_index.len();
                let mut new_triples_in_loop = Vec::new();

                for rule in &stratum_rules {
                    if let Some(agg) = aggregates.get(rule) {
                        if let Some(bindings) = SimpleQueryEngine::query(
                            triple_index,
                            &rule.body,
                            stratum_start_counter,
                        ) {
                            let len = bindings.len();
                            if len > 0 {
                                let group_var_ids: Vec<usize> = agg
                                    .group_vars
                                    .iter()
                                    .map(|v| VarOrTerm::convert(v.clone()).to_encoded())
                                    .collect();
                                let source_var_id = VarOrTerm::convert(agg.source_var.clone()).to_encoded();
                                let target_var_id = VarOrTerm::convert(agg.target_var.clone()).to_encoded();

                                let mut groups: HashMap<Vec<usize>, Vec<usize>> = HashMap::new();
                                for c in 0..len {
                                    let mut group_key = Vec::new();
                                    for &var_id in &group_var_ids {
                                        if let Some(vals) = bindings.get(&var_id) {
                                            group_key.push(vals[c]);
                                        } else {
                                            group_key.push(0);
                                        }
                                    }
                                    if let Some(vals) = bindings.get(&source_var_id) {
                                        groups.entry(group_key).or_default().push(vals[c]);
                                    }
                                }

                                for (group_key, source_vals) in groups {
                                    let mut acc = match agg.function {
                                        AggregateFunction::Count => AccumulatorImpl::Count(CountAccumulator::default()),
                                        AggregateFunction::Sum => AccumulatorImpl::Sum(SumAccumulator::default()),
                                        AggregateFunction::Min => AccumulatorImpl::Min(MinAccumulator::default()),
                                        AggregateFunction::Max => AccumulatorImpl::Max(MaxAccumulator::default()),
                                        AggregateFunction::Avg => AccumulatorImpl::Avg(AvgAccumulator::default()),
                                    };
                                    for val in source_vals {
                                        acc.add(val);
                                    }
                                    let target_val = acc.get();

                                    let mut head = rule.head.clone();
                                    let substitute = |term: &mut VarOrTerm| {
                                        if term.is_var() {
                                            let var_id = term.to_encoded();
                                            if var_id == target_var_id {
                                                *term = VarOrTerm::new_encoded_term(target_val);
                                            } else {
                                                for (i, &gv_id) in group_var_ids.iter().enumerate() {
                                                    if var_id == gv_id {
                                                        *term = VarOrTerm::new_encoded_term(group_key[i]);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    };
                                    substitute(&mut head.s);
                                    substitute(&mut head.p);
                                    substitute(&mut head.o);
                                    if let Some(ref mut g) = head.g {
                                        substitute(g);
                                    }

                                    if !triple_index.contains(&head) && !new_triples_in_loop.contains(&head) {
                                        new_triples_in_loop.push(head);
                                    }
                                }
                            }
                        }
                    } else {
                        
                        if let Some(bindings) = SimpleQueryEngine::query(
                            triple_index,
                            &rule.body,
                            stratum_start_counter,
                        ) {
                            
                            let new_heads = Self::substitute_head_with_bindings(&rule.head, &bindings);
                            
                            for new_head in new_heads {
                                if !triple_index.contains(&new_head) && !new_triples_in_loop.contains(&new_head) {
                                    new_triples_in_loop.push(new_head);
                                }
                            }
                        }
                    }
                }

                if !new_triples_in_loop.is_empty() {
                    for triple in new_triples_in_loop {
                        debug!("Inferred: {:?}", TripleStore::decode_triple(&triple));
                        inferred.push(triple.clone());
                        triple_index.add(triple);
                    }
                    changed = true;
                }
                stratum_start_counter = Some(next_start_counter);
            }
        }

        inferred
    }

    pub fn infer_rule_heads(
        triple_index: &TripleIndex,
        counter: Option<usize>,
        matching_rules: Vec<Rule>,
    ) -> Vec<Triple> {
        let mut new_triples = Vec::new();
        for rule in matching_rules {
            if let Some(temp_bindings) = SimpleQueryEngine::query(triple_index, &rule.body, counter)
            {
                let new_heads = Reasoner::substitute_head_with_bindings(&rule.head, &temp_bindings);

                for new_head in new_heads {
                    new_triples.push(new_head);
                }
            }
        }
        new_triples
    }

    fn substitute_head_with_bindings(head: &Triple, binding: &Binding) -> Vec<Triple> {
        if binding.len() == 0 {
            return vec![head.clone()];
        }
        let mut new_heads = Vec::new();
        let mut s: usize;
        let mut p: usize;
        let mut o: usize;
        for result_counter in 0..binding.len() {
            match &head.s {
                VarOrTerm::Var(s_var) => {
                    s = *binding
                        .get(&s_var.name)
                        .unwrap()
                        .get(result_counter)
                        .unwrap()
                }
                VarOrTerm::Term(s_term) => s = s_term.id(),
            }
            match &head.p {
                VarOrTerm::Var(p_var) => {
                    p = *binding
                        .get(&p_var.name)
                        .unwrap()
                        .get(result_counter)
                        .unwrap()
                }
                VarOrTerm::Term(p_term) => p = p_term.id(),
            }
            match &head.o {
                VarOrTerm::Var(o_var) => {
                    o = *binding
                        .get(&o_var.name)
                        .unwrap()
                        .get(result_counter)
                        .unwrap()
                }
                VarOrTerm::Term(o_term) => o = o_term.id(),
            }
            new_heads.push(Triple {
                s: VarOrTerm::new_encoded_term(s),
                p: VarOrTerm::new_encoded_term(p),
                o: VarOrTerm::new_encoded_term(o),
                g: None,
            })
        }

        new_heads
    }
    fn subsitute_binding(
        var_name: &usize,
        binding: &Binding,
        binding_counter: &usize,
    ) -> VarOrTerm {
        if let Some(s) = binding.get(var_name) {
            let iri = s.get(*binding_counter).unwrap().clone();
            VarOrTerm::new_encoded_term(iri)
        } else {
            VarOrTerm::new_encoded_var(var_name.clone())
        }
    }
    pub fn substitute_triple_with_bindings(head: &Triple, binding: &Binding) -> Vec<Triple> {
        let mut new_heads = Vec::new();
        let mut s: VarOrTerm;
        let mut p: VarOrTerm;
        let mut o: VarOrTerm;
        for result_counter in 0..binding.len() {
            match &head.s {
                VarOrTerm::Var(s_var) => {
                    s = Self::subsitute_binding(&s_var.name, binding, &result_counter)
                }
                VarOrTerm::Term(s_term) => s = VarOrTerm::Term(s_term.clone()),
            }
            match &head.p {
                VarOrTerm::Var(p_var) => {
                    p = Self::subsitute_binding(&p_var.name, binding, &result_counter)
                }
                VarOrTerm::Term(p_term) => p = VarOrTerm::Term(p_term.clone()),
            }
            match &head.o {
                VarOrTerm::Var(o_var) => {
                    o = Self::subsitute_binding(&o_var.name, binding, &result_counter)
                }
                VarOrTerm::Term(o_term) => o = VarOrTerm::Term(o_term.clone()),
            }
            new_heads.push(Triple { s, p, o, g: None })
        }

        new_heads
    }
    pub fn substitute_rule(matching_triple: &Triple, matching_rule: &Rule) -> Vec<Rule> {
        let mut results = Vec::new();
        for body_lit in matching_rule.body.iter() {
            if let Some(bindings) = query(&body_lit.pattern, matching_triple) {
                if bindings.len() == 0 {
                    return vec![matching_rule.clone()];
                }
                let new_body = Self::substitute_rule_body_with_binding(matching_rule, &bindings);
                let new_head =
                    Reasoner::substitute_triple_with_bindings(&matching_rule.head, &bindings)
                        .get(0)
                        .unwrap()
                        .clone();
                results.push(Rule {
                    body: new_body,
                    head: new_head,
                    
                });
            }
        }
        results
    }

    pub fn substitute_rule_body_with_binding(
        matching_rule: &Rule,
        bindings: &Binding,
    ) -> Vec<BodyLiteral> {
        let mut new_body = Vec::new();
        for body_lit in matching_rule.body.iter() {
            let substituted =
                Reasoner::substitute_triple_with_bindings(&body_lit.pattern, &bindings);
            new_body.push(BodyLiteral {
                negated: body_lit.negated,
                pattern: substituted.get(0).unwrap().clone(),
            });
        }
        new_body
    }
}
pub struct CSpriteReasoner;

impl CSpriteReasoner {
    pub fn materialize(
        &mut self,
        new_data: &Vec<(i32, Rc<Triple>)>,
        triple_index: &mut TripleIndex,
        rules_index: &RuleIndex,
        window: &mut ImarsWindow<Triple>,
    ) -> Vec<(i32, Rc<Triple>)> {
        let mut inferred = Vec::new();
        let mut counter = 0;
        let mut pending_changes = Vec::new();
        new_data
            .into_iter()
            .for_each(|i| pending_changes.push(i.clone()));
        while counter < pending_changes.len() {
            let (_ts, process_quad) = pending_changes.get(counter).unwrap();
            //trace!("Processing: {:?}",decode_triple(process_quad));
            //let matching_rules = self.find_matching_rules(process_quad);
            let matching_rules = rules_index.find_match(process_quad);
            trace!("Found Rules: {:?}", matching_rules);
            let mut new_triples = Vec::new();

            for rule in matching_rules {
                if let Some(mut temp_bindings) =
                    SimpleQueryEngine::query(triple_index, &rule.body, None)
                {
                    let new_heads =
                        Reasoner::substitute_head_with_bindings(&rule.head, &temp_bindings);
                    let reconstructed = CSpriteReasoner::reconstruct_triples_from_bindings(
                        &mut temp_bindings,
                        rule,
                    );
                    for i in 0..new_heads.len() {
                        let new_head = new_heads.get(i).unwrap().clone();
                        //println!("Inferred head: {:?}", Self::decode_triple(&new_head,encoder));
                        //compute time stamp
                        let triples = reconstructed.get(i).unwrap();
                        //println!("Triples: {:?}", triples);
                        // let min_ts: Vec<Option<i32>> =triples.iter().map(|t|window.get_time_stamp(Rc::new(t.clone()))).collect();
                        // let min_ts =triples.iter().map(|t|window.get_time_stamp(Rc::new(t.clone()))).filter(|t|t.is_some()).min().unwrap().unwrap();//todo update to reference only
                        let items: Vec<(i32, &Triple)> = triples
                            .iter()
                            .map(|t| (window.get_time_stamp(Rc::new(t.clone())), t))
                            .filter(|(ts, _t)| ts.is_some())
                            .map(|(ts, t)| (ts.unwrap(), t))
                            .collect();
                        if items.is_empty() {
                            continue; // skip this head — no timestamps available
                        }
                        let (min_ts, min_triple) =
                            items.iter().fold(
                                items[0],
                                |acc, &item| {
                                    if acc.0 <= item.0 {
                                        acc
                                    } else {
                                        item
                                    }
                                },
                            );
                        new_triples.push((min_ts.clone(), new_head.clone(), min_triple.clone()));
                    }
                }
            }
            for (ts, new_triple, min_triple) in new_triples {
                if !triple_index.contains(&new_triple) {
                    //trace!("Inferred: {:?}",self.decode_triple(&triple));
                    let triple_ref = Rc::new(new_triple);
                    inferred.push((ts, triple_ref.clone()));
                    //add to maintanance program

                    // window.add_without_update(triple_ref.clone(),ts);
                    window.add_after(triple_ref.clone(), Rc::new(min_triple.clone()), ts);
                    pending_changes.push((ts, triple_ref.clone()));

                    triple_index.add_ref(triple_ref);
                }
            }
            counter += 1;
        }

        inferred
    }
    fn decode_triple(triple: &Triple) -> String {
        let mut res = String::new();

        let decoded_s = Encoder::decode(&triple.s.to_encoded()).unwrap();
        let decoded_p = Encoder::decode(&triple.p.to_encoded()).unwrap();
        let decoded_o = Encoder::decode(&triple.o.to_encoded()).unwrap();

        write!(&mut res, "{} {} {}.\n", decoded_s, decoded_p, decoded_o).unwrap();

        res
    }
    fn reconstruct_triples_from_bindings(
        result_bindings: &mut Binding,
        rule: &Rule,
    ) -> Vec<Vec<Triple>> {
        let mut counter = 0;
        let mut all_triples = Vec::new();
        while counter < result_bindings.len() {
            let mut triples = Vec::new();
            for body_lit in rule.body.iter() {
                let triple = &body_lit.pattern;
                let mut s;
                let mut p;
                let mut o;
                if triple.s.is_var() {
                    s = VarOrTerm::new_encoded_term(
                        *result_bindings
                            .get(&triple.s.as_var().name)
                            .unwrap()
                            .get(counter)
                            .unwrap(),
                    );
                } else {
                    s = triple.s.clone();
                }
                if triple.p.is_var() {
                    p = VarOrTerm::new_encoded_term(
                        *result_bindings
                            .get(&triple.p.as_var().name)
                            .unwrap()
                            .get(counter)
                            .unwrap(),
                    );
                } else {
                    p = triple.p.clone();
                }
                if triple.o.is_var() {
                    o = VarOrTerm::new_encoded_term(
                        *result_bindings
                            .get(&triple.o.as_var().name)
                            .unwrap()
                            .get(counter)
                            .unwrap(),
                    );
                } else {
                    o = triple.o.clone();
                }
                triples.push(Triple { s, p, o, g: None });
            }
            counter += 1;
            all_triples.push(triples);
        }
        all_triples
    }
}
#[test]
#[ignore]
fn test_reconstruct_from_bindings() {
    let data = "{?a in ?c}=>{?a in ?c}";
    let (_content, rules) = Parser::parse(data.to_string());
    println!("encoded {:?}", rules);

    assert_eq!(1, rules.len());
    let rule = &rules[0];

    // Derive the variable/term IDs from the parsed rule instead of hardcoding,
    // so this test is robust against global Encoder state from other tests.
    let body_triple = &rule.body[0].pattern;
    let var_a_id = body_triple.s.as_var().name;
    let in_term_id = body_triple.p.as_term().id();
    let var_c_id = body_triple.o.as_var().name;

    // Bind ?a → 10, ?c → 11 (arbitrary placeholder term IDs for the test)
    let mut result_bindings: Binding = Binding::new();
    result_bindings.add(&var_a_id, 10);
    result_bindings.add(&var_c_id, 11);

    let expected = vec![vec![Triple {
        s: VarOrTerm::new_encoded_term(10),
        p: VarOrTerm::new_encoded_term(in_term_id),
        o: VarOrTerm::new_encoded_term(11),
        g: None,
    }]];

    let triples =
        CSpriteReasoner::reconstruct_triples_from_bindings(&mut result_bindings, rule);
    assert_eq!(expected, triples);
}
pub fn query(query_triple: &Triple, match_triple: &Triple) -> Option<Binding> {
    let mut bindings = Binding::new();
    let Triple { s, p, o, g } = match_triple;
    match &query_triple.s {
        VarOrTerm::Var(s_var) => bindings.add(&s_var.name, s.as_term().id()),
        VarOrTerm::Term(s_term) => {
            if s_term != s.as_term() {
                return None;
            }
        }
    }
    match &query_triple.p {
        VarOrTerm::Var(p_var) => bindings.add(&p_var.name, p.as_term().id()),
        VarOrTerm::Term(p_term) => {
            if p_term != p.as_term() {
                return None;
            }
        }
    }
    match &query_triple.o {
        VarOrTerm::Var(o_var) => bindings.add(&o_var.name, o.as_term().id()),
        VarOrTerm::Term(o_term) => {
            if o_term != o.as_term() {
                return None;
            }
        }
    }

    Some(bindings)
}

#[test]
fn test_rule_substitution() {
    let data = ":a in :b.\n\
                {?a in ?b.?b in ?c}=>{?a in ?c}\n\
                {:a in :b.:b in ?c}=>{:a in ?c}\n\
                {?a in :a.:a in :b}=>{?a in :b}";
    let (content, rules) = Parser::parse(data.to_string());
    let matching_triple = content.get(0).unwrap();
    let matching_rule = rules.get(0).unwrap();
    let results = Reasoner::substitute_rule(matching_triple, matching_rule);
    assert_eq!(&rules[1..], results);
}
