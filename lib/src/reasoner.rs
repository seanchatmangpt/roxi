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
                // `.get(i).copied().unwrap_or(0)` rather than `strata[i]`: if
                // stratification failed validation (e.g. `strata` is shorter
                // than `rules`, or empty), fall back to treating the rule as
                // stratum 0 instead of panicking on an out-of-bounds index.
                .filter(|(i, _)| strata.get(*i).copied().unwrap_or(0) == s)
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
                    } else if let Some(implies_idx) = Self::find_log_implies_literal(rule) {
                        // log:implies dynamic rule reification (see
                        // process_log_implies_rule doc comment).
                        for new_head in Self::process_log_implies_rule(rule, implies_idx, triple_index) {
                            if !triple_index.contains(&new_head) && !new_triples_in_loop.contains(&new_head) {
                                new_triples_in_loop.push(new_head);
                            }
                        }
                    } else if let Some(collect_idx) = Self::find_log_collect_all_in_literal(rule) {
                        // log:collectAllIn (see process_log_collect_all_in_rule doc comment).
                        for new_head in Self::process_log_collect_all_in_rule(rule, collect_idx, triple_index) {
                            if !triple_index.contains(&new_head) && !new_triples_in_loop.contains(&new_head) {
                                new_triples_in_loop.push(new_head);
                            }
                        }
                    } else if let Some(not_includes_idx) = Self::find_log_not_includes_literal(rule) {
                        // log:notIncludes SNAF guard (see process_log_not_includes_rule doc comment).
                        for new_head in Self::process_log_not_includes_rule(rule, not_includes_idx, triple_index) {
                            if !triple_index.contains(&new_head) && !new_triples_in_loop.contains(&new_head) {
                                new_triples_in_loop.push(new_head);
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

    /// The IRI of the `log:implies` built-in predicate.
    const LOG_IMPLIES: &'static str = "<http://www.w3.org/2000/10/swap/log#implies>";

    /// Find the (first) non-negated body literal of `rule` whose predicate is
    /// `log:implies`, if any. Its subject is expected to be bound (at
    /// evaluation time) to a formula term, and its object is a formula
    /// written directly in the rule (the consequent template).
    fn find_log_implies_literal(rule: &Rule) -> Option<usize> {
        rule.body.iter().position(|lit| {
            !lit.negated
                && lit.pattern.p.is_term()
                && Encoder::decode(&lit.pattern.p.to_encoded()).as_deref() == Some(Self::LOG_IMPLIES)
        })
    }

    /// Dynamic rule reification for `log:implies`.
    ///
    /// Given a rule body containing `?formula log:implies { consequent }`
    /// (alongside other, ordinary body literals), this:
    ///  1. Runs the *other* body literals as a normal query to bind
    ///     `?formula` (and anything else in scope) -- call this the "outer"
    ///     bindings, one row per match.
    ///  2. For each outer row, resolves `?formula` to a concrete formula
    ///     term and looks up the triples it was built from (its
    ///     "antecedent") via `VarOrTerm::formula_triples`.
    ///  3. Matches the antecedent's triples against the *current* data via
    ///     the ordinary query engine -- exactly like testing whether a
    ///     nested sub-query holds -- producing zero or more "antecedent"
    ///     bindings for any variables internal to the antecedent formula.
    ///  4. For each (outer row, antecedent row) pair, merges the two into a
    ///     single-row binding and substitutes it into both the consequent
    ///     formula's own triples *and* the rule's declared head, asserting
    ///     whichever come out fully ground. In a well-formed rule (as in
    ///     this crate's test fixtures) the two are the same shape and this
    ///     produces the same triple twice, which the caller dedupes; keeping
    ///     both substitutions handles the general case where they differ.
    ///
    /// Variables are matched across antecedent/consequent/outer scopes purely
    /// by *name* (this engine uses one flat, process-wide variable
    /// namespace -- there is no `@forSome`/`@forAll`-style scoping), so e.g.
    /// reusing `?citizen` inside both the quoted antecedent and the rule's
    /// own head is exactly how a variable "threads through" the implication.
    fn process_log_implies_rule(
        rule: &Rule,
        implies_idx: usize,
        triple_index: &TripleIndex,
    ) -> Vec<Triple> {
        let mut results = Vec::new();
        let implies_lit = &rule.body[implies_idx];

        let regular_body: Vec<BodyLiteral> = rule
            .body
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != implies_idx)
            .map(|(_, lit)| lit.clone())
            .collect();

        let outer_bindings = if regular_body.is_empty() {
            Some(Binding::new())
        } else {
            SimpleQueryEngine::query(triple_index, &regular_body, None)
        };
        let Some(outer_bindings) = outer_bindings else {
            return results;
        };
        let num_outer_rows = if outer_bindings.len() == 0 { 1 } else { outer_bindings.len() };

        // The consequent is written directly in the rule (the log:implies
        // literal's object), so it never needs a bindings lookup.
        let consequent_id = implies_lit.pattern.o.to_encoded();
        let Some(consequent_triples) = VarOrTerm::formula_triples(consequent_id) else {
            return results;
        };

        for row in 0..num_outer_rows {
            let formula_id = if implies_lit.pattern.s.is_var() {
                match outer_bindings
                    .get(&implies_lit.pattern.s.to_encoded())
                    .and_then(|v| v.get(row))
                {
                    Some(&id) => id,
                    None => continue,
                }
            } else {
                implies_lit.pattern.s.to_encoded()
            };

            let Some(antecedent_triples) = VarOrTerm::formula_triples(formula_id) else { continue };
            if antecedent_triples.is_empty() {
                continue;
            }
            let antecedent_body: Vec<BodyLiteral> = antecedent_triples
                .into_iter()
                .map(|pattern| BodyLiteral { negated: false, pattern })
                .collect();

            let Some(antecedent_bindings) = SimpleQueryEngine::query(triple_index, &antecedent_body, None) else {
                continue;
            };
            let num_ante_rows = if antecedent_bindings.len() == 0 { 1 } else { antecedent_bindings.len() };

            for a_row in 0..num_ante_rows {
                let mut merged = Binding::new();
                for (&k, v) in outer_bindings.iter() {
                    if let Some(&val) = v.get(row) {
                        merged.add(&k, val);
                    }
                }
                for (&k, v) in antecedent_bindings.iter() {
                    if let Some(&val) = v.get(a_row) {
                        merged.add(&k, val);
                    }
                }

                for consequent_pattern in &consequent_triples {
                    let t = Self::substitute_single_row(consequent_pattern, &merged);
                    if Self::is_ground(&t) && !results.contains(&t) {
                        results.push(t);
                    }
                }
                let head_t = Self::substitute_single_row(&rule.head, &merged);
                if Self::is_ground(&head_t) && !results.contains(&head_t) {
                    results.push(head_t);
                }
            }
        }

        results
    }

    /// The IRI of the `log:collectAllIn` built-in predicate.
    const LOG_COLLECT_ALL_IN: &'static str = "<http://www.w3.org/2000/10/swap/log#collectAllIn>";
    /// The IRI of the `log:notIncludes` built-in predicate.
    const LOG_NOT_INCLUDES: &'static str = "<http://www.w3.org/2000/10/swap/log#notIncludes>";

    /// Shared helper for `log:collectAllIn` / `log:notIncludes`: resolve
    /// `formula_id` to its quoted-graph triples, substitute any variables
    /// already bound in `row_binding` (e.g. an outer `?Subject` referenced
    /// inside the quoted formula) into each of them, and evaluate the result
    /// as a one-shot sub-query against the *current* `triple_index` -- the
    /// same "match this antecedent's triples against the live store" idea as
    /// the antecedent-matching step of `process_log_implies_rule`. Returns
    /// `None` when the formula is unknown or has zero solutions, `Some(bindings)`
    /// otherwise (possibly a zero-column `Binding` when the formula's triples
    /// are already fully ground and match).
    fn eval_embedded_formula_against_store(
        formula_id: usize,
        row_binding: &Binding,
        triple_index: &TripleIndex,
    ) -> Option<Binding> {
        let triples = VarOrTerm::formula_triples(formula_id)?;
        if triples.is_empty() {
            return Some(Binding::new());
        }
        let body: Vec<BodyLiteral> = triples
            .iter()
            .map(|p| BodyLiteral { negated: false, pattern: Self::substitute_single_row(p, row_binding) })
            .collect();
        SimpleQueryEngine::query(triple_index, &body, None)
    }

    /// Find the (first) non-negated body literal of `rule` whose predicate is
    /// `log:collectAllIn`.
    fn find_log_collect_all_in_literal(rule: &Rule) -> Option<usize> {
        rule.body.iter().position(|lit| {
            !lit.negated
                && lit.pattern.p.is_term()
                && Encoder::decode(&lit.pattern.p.to_encoded()).as_deref() == Some(Self::LOG_COLLECT_ALL_IN)
        })
    }

    /// `log:collectAllIn` -- collect every distinct binding of a variable
    /// across all solutions of a quoted-graph formula into an RDF list.
    ///
    /// The body literal has the shape `(?Var {quoted-graph} ?List) log:collectAllIn ?Scope`:
    /// its *subject* is a 3-element list term `(?Var {formula} ?List)` (the
    /// scope/object is simply ignored -- this engine only ever has one,
    /// current, triple store to collect against). For each row produced by
    /// evaluating the rule's *other* body literals (the "outer" bindings,
    /// e.g. binding `?Subject` before the quoted formula references it),
    /// this substitutes those outer bindings into the formula's own triples,
    /// runs them as a sub-query via `eval_embedded_formula_against_store`,
    /// dedups the resulting `?Var` column (preserving first-seen order) into
    /// an RDF list, and binds that list to `?List`. Any body literals that
    /// come *after* the `log:collectAllIn` literal (e.g. `?List math:sum
    /// ?Count`, which depends on the freshly-built list) are then evaluated
    /// against the merged row using the same builtin dispatch
    /// `SimpleQueryEngine::query` uses internally, since they can't be
    /// pre-computed by an ordinary whole-body query (the list doesn't exist
    /// as data until this step produces it).
    fn process_log_collect_all_in_rule(
        rule: &Rule,
        collect_idx: usize,
        triple_index: &TripleIndex,
    ) -> Vec<Triple> {
        let mut results = Vec::new();
        let collect_lit = &rule.body[collect_idx];

        let Some(members) = VarOrTerm::list_members(collect_lit.pattern.s.to_encoded()) else {
            return results;
        };
        if members.len() != 3 {
            return results;
        }
        let collect_var_id = members[0];
        let formula_id_template = members[1];
        let target_id = members[2];

        let pre_body: Vec<BodyLiteral> = rule.body[..collect_idx].to_vec();
        let post_body: Vec<BodyLiteral> = rule.body[collect_idx + 1..].to_vec();

        let outer_bindings = if pre_body.is_empty() {
            Some(Binding::new())
        } else {
            SimpleQueryEngine::query(triple_index, &pre_body, None)
        };
        let Some(outer_bindings) = outer_bindings else {
            return results;
        };
        let num_outer_rows = if outer_bindings.len() == 0 { 1 } else { outer_bindings.len() };

        for row in 0..num_outer_rows {
            let mut row_binding = Binding::new();
            for (&k, v) in outer_bindings.iter() {
                if let Some(&val) = v.get(row) {
                    row_binding.add(&k, val);
                }
            }

            // The quoted formula may itself be a variable (rare) or, as in
            // every known corpus case, written directly in the rule -- if
            // `formula_id_template` happens to be bound as a variable in
            // this row, use that binding; otherwise it already names the
            // formula directly.
            let formula_id = row_binding
                .get(&formula_id_template)
                .and_then(|v| v.get(0))
                .copied()
                .unwrap_or(formula_id_template);

            if VarOrTerm::formula_triples(formula_id).is_none() {
                continue;
            }
            let ante_bindings = Self::eval_embedded_formula_against_store(formula_id, &row_binding, triple_index);

            // Dedup the collected variable's column across all antecedent
            // solutions, preserving first-seen order (deterministic). If
            // `collect_var_id` never appears as a binding key, the template
            // isn't actually a variable (e.g. EYE's `dog` corpus case uses
            // the ground template `1`, purely to *count* solutions via a
            // following `math:sum` -- see the `1 { ... } ?List` shape): in
            // that case every antecedent solution contributes one (un-deduped)
            // copy of the ground value, since the point is the count of
            // solutions, not their distinct values.
            let mut collected: Vec<usize> = Vec::new();
            if let Some(ante_bindings) = &ante_bindings {
                if let Some(vals) = ante_bindings.get(&collect_var_id) {
                    for &v in vals {
                        if !collected.contains(&v) {
                            collected.push(v);
                        }
                    }
                } else {
                    let num_ante_rows = if ante_bindings.len() == 0 { 1 } else { ante_bindings.len() };
                    for _ in 0..num_ante_rows {
                        collected.push(collect_var_id);
                    }
                }
            }

            let list_members: Vec<VarOrTerm> = collected.into_iter().map(VarOrTerm::new_encoded_term).collect();
            let list_id = VarOrTerm::new_list(list_members).to_encoded();

            let mut merged = row_binding.clone();
            merged.add(&target_id, list_id);

            // Evaluate any remaining body literals (typically builtins like
            // math:sum/list:length/math:greaterThan operating on the new
            // list/its derived values) against the merged single row.
            let Some(final_binding) = Self::eval_post_literals(&post_body, merged, triple_index) else {
                continue;
            };

            let head_t = Self::substitute_single_row(&rule.head, &final_binding);
            if Self::is_ground(&head_t) && !results.contains(&head_t) {
                results.push(head_t);
            }
        }

        results
    }

    /// Find the (first) non-negated body literal of `rule` whose predicate is
    /// `log:notIncludes`.
    fn find_log_not_includes_literal(rule: &Rule) -> Option<usize> {
        rule.body.iter().position(|lit| {
            !lit.negated
                && lit.pattern.p.is_term()
                && Encoder::decode(&lit.pattern.p.to_encoded()).as_deref() == Some(Self::LOG_NOT_INCLUDES)
        })
    }

    /// `log:notIncludes` -- SNAF (simple negation-as-failure) guard.
    ///
    /// The body literal `?Scope log:notIncludes { pattern }` succeeds for a
    /// given row of the rule's *other* body literals iff substituting that
    /// row's bindings into `{ pattern }` and querying the result against the
    /// current `triple_index` (via `eval_embedded_formula_against_store`)
    /// yields **zero** solutions. Rows for which it fails are dropped
    /// (filtered out), mirroring how negated body literals are handled in
    /// `SimpleQueryEngine::query`, except this needs the full `TripleIndex`
    /// (to run a genuine sub-query rather than a single ground lookup) so it
    /// is dispatched here rather than through the ordinary builtins module.
    fn process_log_not_includes_rule(
        rule: &Rule,
        not_includes_idx: usize,
        triple_index: &TripleIndex,
    ) -> Vec<Triple> {
        let mut results = Vec::new();
        let lit = &rule.body[not_includes_idx];
        let formula_id_template = lit.pattern.o.to_encoded();

        let other_body: Vec<BodyLiteral> = rule
            .body
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != not_includes_idx)
            .map(|(_, l)| l.clone())
            .collect();

        let outer_bindings = if other_body.is_empty() {
            Some(Binding::new())
        } else {
            SimpleQueryEngine::query(triple_index, &other_body, None)
        };
        let Some(outer_bindings) = outer_bindings else {
            return results;
        };
        let num_rows = if outer_bindings.len() == 0 { 1 } else { outer_bindings.len() };

        for row in 0..num_rows {
            let mut row_binding = Binding::new();
            for (&k, v) in outer_bindings.iter() {
                if let Some(&val) = v.get(row) {
                    row_binding.add(&k, val);
                }
            }

            if VarOrTerm::formula_triples(formula_id_template).is_none() {
                continue;
            }
            let guard_result = Self::eval_embedded_formula_against_store(formula_id_template, &row_binding, triple_index);

            if guard_result.is_none() {
                // Zero solutions for the (row-instantiated) guard formula:
                // the SNAF guard passes, so this row survives.
                let head_t = Self::substitute_single_row(&rule.head, &row_binding);
                if Self::is_ground(&head_t) && !results.contains(&head_t) {
                    results.push(head_t);
                }
            }
        }

        results
    }

    /// Evaluate a sequence of body literals (typically N3 builtins, e.g.
    /// `math:sum`/`math:greaterThan`) against a single already-built row of
    /// bindings, in order -- used by `process_log_collect_all_in_rule` to
    /// handle literals that depend on values (like the freshly-built list)
    /// which don't exist as queryable data. Returns `None` if any literal
    /// fails to hold for this row.
    fn eval_post_literals(
        post_body: &[BodyLiteral],
        start: Binding,
        triple_index: &TripleIndex,
    ) -> Option<Binding> {
        let mut bindings = start;
        for lit in post_body {
            if lit.negated {
                continue;
            }
            if let Some(kind) = crate::queryengine::builtins::classify(&lit.pattern.p) {
                bindings = crate::queryengine::builtins::evaluate(kind, &lit.pattern, &bindings)?;
            } else if let Some(cur) = triple_index.query(&lit.pattern, None) {
                bindings = bindings.join(&cur);
            } else {
                return None;
            }
        }
        Some(bindings)
    }

    /// Substitute variables in `pattern` using a *single-row* binding (each
    /// key maps to exactly one value). Unlike `substitute_triple_with_bindings`,
    /// this never skips substitution when `binding` has columns but a
    /// particular variable happens to be unbound -- it simply leaves that
    /// operand as the original variable, letting `is_ground` catch it.
    fn substitute_single_row(pattern: &Triple, binding: &Binding) -> Triple {
        let sub = |term: &VarOrTerm| -> VarOrTerm {
            if term.is_var() {
                if let Some(&val) = binding.get(&term.to_encoded()).and_then(|v| v.get(0)) {
                    return VarOrTerm::new_encoded_term(val);
                }
            }
            term.clone()
        };
        Triple {
            s: sub(&pattern.s),
            p: sub(&pattern.p),
            o: sub(&pattern.o),
            g: pattern.g.as_ref().map(sub),
        }
    }

    /// A triple is safe to assert as a derived fact only once every position
    /// is a concrete term (no leftover unbound variables).
    fn is_ground(t: &Triple) -> bool {
        !t.s.is_var() && !t.p.is_var() && !t.o.is_var() && t.g.as_ref().map_or(true, |g| !g.is_var())
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
