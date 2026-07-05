use crate::{Binding, Encoder, Rule, RuleIndex, Triple, TripleIndex, TripleStore, VarOrTerm};
use log::{debug, info, trace, warn};
use std::rc::Rc; // Use log crate when building application



pub struct BackwardChainer;

impl BackwardChainer {
    pub fn eval_backward(
        triple_index: &TripleIndex,
        rule_index: &RuleIndex,
        rule_head: &Triple,
    ) -> Binding {
        let mut history = std::collections::HashSet::new();
        Self::eval_backward_inner(triple_index, rule_index, rule_head, &mut history)
    }

    fn eval_backward_inner(
        triple_index: &TripleIndex,
        rule_index: &RuleIndex,
        rule_head: &Triple,
        history: &mut std::collections::HashSet<Triple>,
    ) -> Binding {
        if !history.insert(rule_head.clone()) {
            return Binding::new();
        }
        let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> =
            Self::find_subrules(rule_index, rule_head);
        let mut all_bindings = Binding::new();
        for (sub_rule, var_subs) in sub_rules.into_iter() {
            debug!(
                "Backchaining rule: {:?}",
                TripleStore::decode_rule(&sub_rule)
            );
            // Initialize rule_bindings as "empty" — join identity.
            // For each body literal, we:
            //   1. Collect bindings from direct EDB lookup (triple_index.query)
            //   2. Collect bindings from recursive IDB derivation (eval_backward_inner)
            //   3. Union both via combine (bag union of result rows)
            //   4. Join the per-literal bindings into the running rule_bindings
            //      (natural join on shared variables across body literals)
            let mut rule_bindings = Binding::new();
            let mut rule_failed = false;
            for body_lit in &sub_rule.body {
                let rule_atom = &body_lit.pattern;
                debug!("Matching body: {:?}", TripleStore::decode_triple(rule_atom));

                // Collect bindings for this literal from both sources.
                let mut lit_bindings = Binding::new();

                // Direct EDB lookup.
                if let Some(result_bindings) = triple_index.query(rule_atom, None) {
                    debug!(
                        "   Found matching body (direct): {:?}",
                        TripleStore::decode_bindings(&result_bindings)
                    );
                    lit_bindings.combine(result_bindings);
                }

                // Recursive IDB derivation (skip for negated literals).
                if !body_lit.negated {
                    let recursive_bindings =
                        Self::eval_backward_inner(triple_index, rule_index, rule_atom, history);
                    if recursive_bindings.len() > 0 {
                        lit_bindings.combine(recursive_bindings);
                    }
                }

                // Join per-literal bindings with the running result.
                rule_bindings = rule_bindings.join(&lit_bindings);

                // Short-circuit: if neither direct nor recursive found anything,
                // this rule body cannot be satisfied.
                if lit_bindings.len() == 0 {
                    rule_failed = true;
                    break;
                }
            }
            if rule_failed {
                continue;
            }
            // Rename rule-internal variables to query variables via var_subs.
            let renamed = rule_bindings.rename(var_subs);
            all_bindings.combine(renamed);
        }
        history.remove(rule_head);
        all_bindings
    }

    /// Find all rules whose head unifies with `rule_head`.
    /// Returns the matching rules paired with variable substitution pairs
    /// `(rule_var, query_var)` for renaming after evaluation.
    pub fn find_subrules(
        rules_index: &RuleIndex,
        rule_head: &Triple,
    ) -> Vec<(Rc<Rule>, Vec<(usize, usize)>)> {
        let candidates: &[Rc<Rule>] = if rule_head.p.is_term() {
            // Fast path: look up only rules whose head predicate matches
            rules_index
                .head_by_pred
                .get(&rule_head.p.to_encoded())
                .map(|v| v.as_slice())
                .unwrap_or(&[])
        } else {
            // Variable predicate: must check all rules
            rules_index.rules.as_slice()
        };
        let mut rule_matches = Vec::new();
        for rule in candidates.iter() {
            let head: &Triple = &rule.head;
            let mut var_names_subs: Vec<(usize, usize)> = Vec::new();
            if Self::eval_triple_element(&head.s, &rule_head.s, &mut var_names_subs)
                && Self::eval_triple_element(&head.p, &rule_head.p, &mut var_names_subs)
                && Self::eval_triple_element(&head.o, &rule_head.o, &mut var_names_subs)
            {
                rule_matches.push((rule.clone(), var_names_subs));
            }
        }
        rule_matches
    }

    fn eval_triple_element(
        left: &VarOrTerm,
        right: &VarOrTerm,
        var_names_sub: &mut Vec<(usize, usize)>,
    ) -> bool {
        if let (VarOrTerm::Var(left_name), VarOrTerm::Var(right_name)) = (left, right) {
            var_names_sub.push((left_name.name, right_name.name));
            true
        } else {
            left.eq(right)
        }
    }
}

#[cfg(test)]
#[path = "backwardchaining_test.rs"]
mod backwardchaining_test;
