use super::Reasoner;
use crate::{Binding, BodyLiteral, Encoder, QueryEngine, Rule, SimpleQueryEngine, Triple, TripleIndex, VarOrTerm};

impl Reasoner {
    /// The IRI of the `log:implies` built-in predicate.
    const LOG_IMPLIES: &'static str = "<http://www.w3.org/2000/10/swap/log#implies>";

    /// Find the (first) non-negated body literal of `rule` whose predicate is
    /// `log:implies`, if any. Its subject is expected to be bound (at
    /// evaluation time) to a formula term, and its object is a formula
    /// written directly in the rule (the consequent template).
    pub(crate) fn find_log_implies_literal(rule: &Rule) -> Option<usize> {
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
    pub(crate) fn process_log_implies_rule(
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
}
