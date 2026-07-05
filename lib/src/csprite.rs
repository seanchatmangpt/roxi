use crate::{
    BackwardChainer, Encoder, Reasoner, Rule, RuleIndex, Triple, TripleIndex, TripleStore,
};
use log::{info, trace, warn}; // Use log crate when building application
use std::collections::HashSet;
use std::fmt::Write;
use std::rc::Rc;

use crate::imars_window::{ImarsWindow, WindowConsumer};
use crate::reasoner::CSpriteReasoner;
use std::cell::RefCell;


pub struct CSprite {
    pub rules: Vec<Rule>,
    pub rules_index: RuleIndex,
    pub triple_index: TripleIndex,
    window_reasoner: CSpriteReasoner,
    reasoner: Reasoner,
    imars: ImarsWindow<Triple>,
}

impl CSprite {
    pub fn new() -> CSprite {
        CSprite {
            rules: Vec::new(),
            rules_index: RuleIndex::new(),
            triple_index: TripleIndex::new(),
            window_reasoner: CSpriteReasoner {},
            reasoner: Reasoner {},
            imars: ImarsWindow::new_no_window(),
        }
    }
    pub fn from(data: &str) -> CSprite {
        let triple_store = TripleStore::from(&data);
        CSprite {
            rules: triple_store.rules,
            rules_index: triple_store.rules_index,
            triple_index: triple_store.triple_index,
            window_reasoner: CSpriteReasoner {},
            reasoner: Reasoner {},
            imars: ImarsWindow::new_no_window(),
        }
    }
    pub fn window_update(
        &mut self,
        new_data: Vec<(i32, Rc<Triple>)>,
        old_data: Vec<(i32, Rc<Triple>)>,
        last_ts: &i32,
    ) {
        //remove expired data
        let old_items = self.imars.remove_old_elements(last_ts);

        old_items
            .into_iter()
            .for_each(|(_ts, item)| self.triple_index.remove_ref(&item));

        //add new data

        new_data.iter().for_each(|(ts, triple)| {
            self.imars.add_without_update(triple.clone(), *ts);
            self.add_ref(triple.clone());
        });
        let materialization = self.window_reasoner.materialize(
            &new_data,
            &mut self.triple_index,
            &self.rules_index,
            &mut self.imars,
        );

        //add materialization to maintenance program
        //materialization.into_iter().for_each(|(ts,t)|self.imars.add_without_update(t,ts));
    }
    fn decode_triples(triples: &Vec<(i32, Rc<Triple>)>) -> String {
        let mut res = String::new();
        for (ts, triple) in triples {
            let decoded_s = Encoder::decode(&triple.s.to_encoded()).unwrap();
            let decoded_p = Encoder::decode(&triple.p.to_encoded()).unwrap();
            let decoded_o = Encoder::decode(&triple.o.to_encoded()).unwrap();

            write!(
                &mut res,
                "{} {} {} @ {}.\n",
                decoded_s, decoded_p, decoded_o, ts
            )
            .unwrap();
        }
        res
    }

    pub fn add(&mut self, triple: Triple) {
        trace! {"Adding triple: {:?}", self.decode_triple(&triple) }
        self.triple_index.add(triple);
    }
    pub fn add_ref(&mut self, triple: Rc<Triple>) {
        trace! {"Adding triple: {:?}", self.decode_triple(triple.as_ref()) }
        self.triple_index.add_ref(triple);
    }
    pub fn remove_ref(&mut self, triple: Rc<Triple>) {
        trace! {"Removing triple: {:?}", self.decode_triple(triple.as_ref()) }
        self.triple_index.remove_ref(&triple);
    }
    pub fn add_rules(&mut self, rules: Vec<Rule>) {
        rules
            .into_iter()
            .for_each(|rule| self.rules_index.add(rule));
    }
    pub fn len(&self) -> usize {
        self.triple_index.len()
    }
    fn decode_triple(&self, triple: &Triple) -> String {
        let s = Encoder::decode(&triple.s.to_encoded()).unwrap();
        let p = Encoder::decode(&triple.p.to_encoded()).unwrap();
        let o = Encoder::decode(&triple.o.to_encoded()).unwrap();
        format!("{} {} {}", s, p, o)
    }
    pub fn materialize_window(
        &mut self,
        window_items: Vec<(i32, Rc<Triple>)>,
    ) -> Vec<(i32, Triple)> {
        self.window_reasoner
            .materialize(&window_items, &mut self.triple_index, &self.rules_index, &mut self.imars)
            .into_iter()
            .map(|(ts, t)| (ts, Rc::try_unwrap(t).unwrap_or_else(|t| (*t).clone())))
            .collect()
    }
    pub fn materialize(&mut self) -> Vec<Triple> {
        let aggregates = std::collections::HashMap::new();
        let strata = crate::datalog::validate_rules(&self.rules, &aggregates)
            .unwrap_or_else(|_| vec![0; self.rules.len()]);
        self.reasoner
            .materialize(&mut self.triple_index, &self.rules, &strata, &aggregates)
    }
    pub fn clear(&mut self) {
        self.triple_index.clear();
    }
    pub(crate) fn compute_sprite(&mut self, query: &Triple) {
        let (backward_rules, hierarcies) = self.eval_backward_csprite(query);

        // new rules
        let mut new_rules: Vec<Rc<Rule>> = backward_rules
            .into_iter()
            .filter(|r| r.body.len() > 1)
            .collect();
        for hierarchy in hierarcies {
            let rewritten_hierarchy = Self::rewrite_hierarchy(&hierarchy);
            rewritten_hierarchy
                .into_iter()
                .for_each(|r| new_rules.push(Rc::new(r)));
        }

        // new rule index
        let mut parsed_rules_index = RuleIndex::new();
        for rule in new_rules.iter() {
            parsed_rules_index.add_ref(rule);
        }
        self.rules_index = parsed_rules_index;
        self.rules = new_rules.into_iter().map(|r| Rc::try_unwrap(r).unwrap_or_else(|r| (*r).clone())).collect();
    }
    fn eval_backward_csprite(&self, rule_head: &Triple) -> (HashSet<Rc<Rule>>, Vec<Vec<Rc<Rule>>>) {
        let mut matched_rules = HashSet::new();
        let mut hierarchies = Vec::new();
        let mut history = HashSet::new();
        self.eval_backward_csprite_helper(
            rule_head,
            &mut matched_rules,
            false,
            &mut hierarchies,
            &mut history,
        );
        (matched_rules, hierarchies)
        //self.eval_backward_csprite_helper_with_stack(rule_head)
    }
    fn eval_backward_csprite_helper(
        &self,
        rule_head: &Triple,
        matched_rules: &mut HashSet<Rc<Rule>>,
        hierarchy: bool,
        hierarchies: &mut Vec<Vec<Rc<Rule>>>,
        history: &mut HashSet<Triple>,
    ) {
        if !history.insert(rule_head.clone()) {
            return;
        }
        let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> =
            BackwardChainer::find_subrules(&self.rules_index, rule_head);
        let mut current_hierarchy = false;
        for (sub_rule, var_subs) in sub_rules.into_iter() {
            if matched_rules.insert(sub_rule.clone()) {
                if sub_rule.body.len() == 1 {
                    //hierarchy candidate
                    if hierarchy {
                        if let Some(current_hierarchy) = hierarchies.last_mut() {
                            current_hierarchy.push(sub_rule.clone());
                        }
                    } else {
                        hierarchies.push(Vec::from([sub_rule.clone()]));
                    }
                    current_hierarchy = true;
                }
                for body_lit in &sub_rule.body {
                    let rule_atom = &body_lit.pattern;
                    //recursive call
                    self.eval_backward_csprite_helper(
                        rule_atom,
                        matched_rules,
                        current_hierarchy,
                        hierarchies,
                        history,
                    );
                }
            }
        }
        history.remove(rule_head);
    }
    fn eval_backward_csprite_helper_with_stack(
        &self,
        rule_head: &Triple,
    ) -> (HashSet<Rc<Rule>>, Vec<Vec<Rc<Rule>>>) {
        enum StackFrame {
            Enter { rule_head: Triple, hierarchy: bool },
            Exit { rule_head: Triple },
        }
        let mut stack = vec![StackFrame::Enter {
            rule_head: rule_head.clone(),
            hierarchy: false,
        }];
        let mut matched_rules = HashSet::new();
        let mut hierarchies: Vec<Vec<Rc<Rule>>> = Vec::new();
        let mut history = HashSet::new();
        while let Some(frame) = stack.pop() {
            match frame {
                StackFrame::Exit { rule_head } => {
                    history.remove(&rule_head);
                }
                StackFrame::Enter { rule_head, hierarchy } => {
                    if !history.insert(rule_head.clone()) {
                        continue;
                    }
                    stack.push(StackFrame::Exit {
                        rule_head: rule_head.clone(),
                    });
                    let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> =
                        BackwardChainer::find_subrules(&self.rules_index, &rule_head);
                    for (sub_rule, _var_subs) in sub_rules.into_iter() {
                        if matched_rules.insert(sub_rule.clone()) {
                            let mut current_hierarchy = false;
                            if sub_rule.body.len() == 1 {
                                //hierarchy candidate
                                if hierarchy {
                                    if let Some(h) = hierarchies.last_mut() {
                                        h.push(sub_rule.clone());
                                    }
                                } else {
                                    hierarchies.push(Vec::from([sub_rule.clone()]));
                                }
                                current_hierarchy = true;
                            }
                            for body_lit in sub_rule.body.iter().rev() {
                                let rule_atom = &body_lit.pattern;
                                stack.push(StackFrame::Enter {
                                    rule_head: rule_atom.clone(),
                                    hierarchy: current_hierarchy,
                                });
                            }
                        }
                    }
                }
            }
        }
        (matched_rules, hierarchies)
    }
    fn rewrite_hierarchy(rules: &Vec<Rc<Rule>>) -> Vec<Rule> {
        let mut new_rules = Vec::new();
        if rules.len() > 0 {
            let new_head = &rules.get(0).unwrap().head;
            for rule in rules.iter() {
                new_rules.push(Rule {
                    body: rule.body.clone(),
                    head: new_head.clone(),
                    
                })
            }
        }

        new_rules
    }
}
#[cfg(test)]
#[path = "csprite_test.rs"]
mod csprite_test;
