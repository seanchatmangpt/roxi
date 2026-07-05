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
            if let Some(kind) = builtins::classify(&lit.pattern.p) {
                // N3 built-in predicates (log:/math:/string:/list:) are evaluated
                // procedurally against the bindings accumulated *so far*, rather
                // than looked up in the data's TripleIndex (they never appear as
                // literal triples). They are never joined via `Binding::join`
                // either -- unlike a real query result set, a builtin's output
                // rows correspond 1:1 (or, for list:in, 1:N) to the rows already
                // in `bindings`, so it directly replaces `bindings`.
                match builtins::evaluate(kind, &lit.pattern, &bindings) {
                    Some(new_bindings) => {
                        bindings = new_bindings;
                        first = false;
                    }
                    None => return None,
                }
            } else if let Some(current_bindings) = data.query(&lit.pattern, None) {
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

/// N3 core built-in predicates (log:, math:, string:, list:).
///
/// `log:implies` is deliberately **not** handled here: it requires dynamically
/// reifying its antecedent formula as a rule and feeding the result back into
/// the fixpoint loop, which only makes sense at the `Reasoner::materialize`
/// level (see reasoner.rs). Everything else here is a pure, one-shot
/// evaluation over the current row(s) of bindings.
pub mod builtins {
    use crate::{Binding, Encoder, Term, Triple, VarOrTerm};

    const LOG_EQUAL_TO: &str = "<http://www.w3.org/2000/10/swap/log#equalTo>";
    const MATH_GREATER_THAN: &str = "<http://www.w3.org/2000/10/swap/math#greaterThan>";
    const MATH_SUM: &str = "<http://www.w3.org/2000/10/swap/math#sum>";
    const STRING_LENGTH: &str = "<http://www.w3.org/2000/10/swap/string#length>";
    const STRING_CONCAT: &str = "<http://www.w3.org/2000/10/swap/string#concat>";
    const LIST_LENGTH: &str = "<http://www.w3.org/2000/10/swap/list#length>";
    const LIST_IN: &str = "<http://www.w3.org/2000/10/swap/list#in>";
    const MATH_DIFFERENCE: &str = "<http://www.w3.org/2000/10/swap/math#difference>";
    const MATH_PRODUCT: &str = "<http://www.w3.org/2000/10/swap/math#product>";
    const MATH_QUOTIENT: &str = "<http://www.w3.org/2000/10/swap/math#quotient>";
    const MATH_REMAINDER: &str = "<http://www.w3.org/2000/10/swap/math#remainder>";
    const MATH_NOT_LESS_THAN: &str = "<http://www.w3.org/2000/10/swap/math#notLessThan>";
    const MATH_NOT_GREATER_THAN: &str = "<http://www.w3.org/2000/10/swap/math#notGreaterThan>";
    const MATH_LESS_THAN: &str = "<http://www.w3.org/2000/10/swap/math#lessThan>";
    const MATH_EQUAL_TO: &str = "<http://www.w3.org/2000/10/swap/math#equalTo>";
    const LOG_NOT_EQUAL_TO: &str = "<http://www.w3.org/2000/10/swap/log#notEqualTo>";
    const LIST_APPEND: &str = "<http://www.w3.org/2000/10/swap/list#append>";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BuiltinKind {
        EqualTo,
        GreaterThan,
        Sum,
        StringLength,
        StringConcat,
        ListLength,
        ListIn,
        Difference,
        Product,
        Quotient,
        Remainder,
        NotLessThan,
        NotGreaterThan,
        LessThan,
        MathEqualTo,
        NotEqualTo,
        ListAppend,
    }

    /// Identify whether `p` (a body literal's predicate position) names one of
    /// the built-ins handled procedurally by `evaluate`.
    pub fn classify(p: &VarOrTerm) -> Option<BuiltinKind> {
        if !p.is_term() {
            return None;
        }
        let decoded = Encoder::decode(&p.to_encoded())?;
        Some(match decoded.as_str() {
            LOG_EQUAL_TO => BuiltinKind::EqualTo,
            MATH_GREATER_THAN => BuiltinKind::GreaterThan,
            MATH_SUM => BuiltinKind::Sum,
            STRING_LENGTH => BuiltinKind::StringLength,
            STRING_CONCAT => BuiltinKind::StringConcat,
            LIST_LENGTH => BuiltinKind::ListLength,
            LIST_IN => BuiltinKind::ListIn,
            MATH_DIFFERENCE => BuiltinKind::Difference,
            MATH_PRODUCT => BuiltinKind::Product,
            MATH_QUOTIENT => BuiltinKind::Quotient,
            MATH_REMAINDER => BuiltinKind::Remainder,
            MATH_NOT_LESS_THAN => BuiltinKind::NotLessThan,
            MATH_NOT_GREATER_THAN => BuiltinKind::NotGreaterThan,
            MATH_LESS_THAN => BuiltinKind::LessThan,
            MATH_EQUAL_TO => BuiltinKind::MathEqualTo,
            LOG_NOT_EQUAL_TO => BuiltinKind::NotEqualTo,
            LIST_APPEND => BuiltinKind::ListAppend,
            _ => return None,
        })
    }

    /// Evaluate a single built-in body literal against the bindings
    /// accumulated so far in the enclosing conjunction. Returns `None` if the
    /// builtin cannot fire at all (constraint failed / operands unresolved
    /// for every row), mirroring a failed `TripleIndex::query` lookup.
    pub fn evaluate(kind: BuiltinKind, pattern: &Triple, bindings: &Binding) -> Option<Binding> {
        match kind {
            BuiltinKind::EqualTo => eval_row_constraint(pattern, bindings, |s, o| {
                s == o
                    || matches!((numeric_value(s), numeric_value(o)), (Some(a), Some(b)) if a == b)
            }),
            BuiltinKind::GreaterThan => eval_row_constraint(pattern, bindings, |s, o| {
                matches!((numeric_value(s), numeric_value(o)), (Some(a), Some(b)) if a > b)
            }),
            BuiltinKind::Sum => eval_functional(pattern, bindings, |pattern, bindings, row| {
                // math:sum is n-ary per the N3 built-ins spec ("the sum of
                // the elements of the list"), not fixed at 2 operands -- e.g.
                // EYE's `dog` corpus case sums an arbitrarily-long collected
                // list. Fold over however many members the list has (an
                // empty list sums to 0, the additive identity).
                let members = subject_list_members(&pattern.s, bindings, row)?;
                let mut total = 0.0;
                for m in &members {
                    total += numeric_value(*m)?;
                }
                Some(intern_number(total))
            }),
            BuiltinKind::StringLength => eval_functional(pattern, bindings, |pattern, bindings, row| {
                let s = resolve_operand(&pattern.s, bindings, row)?;
                let lex = lexical_value(s)?;
                Some(intern_number(lex.chars().count() as f64))
            }),
            BuiltinKind::StringConcat => eval_functional(pattern, bindings, |pattern, bindings, row| {
                let members = subject_list_members(&pattern.s, bindings, row)?;
                let mut out = String::new();
                for m in members {
                    out.push_str(&lexical_value(m)?);
                }
                Some(intern_string(out))
            }),
            BuiltinKind::ListLength => eval_functional(pattern, bindings, |pattern, bindings, row| {
                let members = subject_list_members(&pattern.s, bindings, row)?;
                Some(intern_number(members.len() as f64))
            }),
            BuiltinKind::ListIn => eval_generator(pattern, bindings),
            BuiltinKind::Difference => eval_functional(pattern, bindings, |pattern, bindings, row| {
                let members = subject_list_members(&pattern.s, bindings, row)?;
                if members.len() != 2 {
                    return None;
                }
                let a = numeric_value(members[0])?;
                let b = numeric_value(members[1])?;
                Some(intern_number(a - b))
            }),
            BuiltinKind::Product => eval_functional(pattern, bindings, |pattern, bindings, row| {
                // n-ary, like math:sum above.
                let members = subject_list_members(&pattern.s, bindings, row)?;
                let mut total = 1.0;
                for m in &members {
                    total *= numeric_value(*m)?;
                }
                Some(intern_number(total))
            }),
            BuiltinKind::Quotient => eval_functional(pattern, bindings, |pattern, bindings, row| {
                let members = subject_list_members(&pattern.s, bindings, row)?;
                if members.len() != 2 {
                    return None;
                }
                let a = numeric_value(members[0])?;
                let b = numeric_value(members[1])?;
                if b == 0.0 {
                    return None;
                }
                Some(intern_number(a / b))
            }),
            BuiltinKind::Remainder => eval_functional(pattern, bindings, |pattern, bindings, row| {
                let members = subject_list_members(&pattern.s, bindings, row)?;
                if members.len() != 2 {
                    return None;
                }
                let a = numeric_value(members[0])?;
                let b = numeric_value(members[1])?;
                if b == 0.0 {
                    return None;
                }
                Some(intern_number(a % b))
            }),
            BuiltinKind::NotLessThan => eval_row_constraint(pattern, bindings, |s, o| {
                matches!((numeric_value(s), numeric_value(o)), (Some(a), Some(b)) if a >= b)
            }),
            BuiltinKind::NotGreaterThan => eval_row_constraint(pattern, bindings, |s, o| {
                matches!((numeric_value(s), numeric_value(o)), (Some(a), Some(b)) if a <= b)
            }),
            BuiltinKind::LessThan => eval_row_constraint(pattern, bindings, |s, o| {
                matches!((numeric_value(s), numeric_value(o)), (Some(a), Some(b)) if a < b)
            }),
            BuiltinKind::MathEqualTo => eval_row_constraint(pattern, bindings, |s, o| {
                matches!((numeric_value(s), numeric_value(o)), (Some(a), Some(b)) if a == b)
            }),
            BuiltinKind::NotEqualTo => eval_row_constraint(pattern, bindings, |s, o| {
                !(s == o
                    || matches!((numeric_value(s), numeric_value(o)), (Some(a), Some(b)) if a == b))
            }),
            BuiltinKind::ListAppend => eval_functional(pattern, bindings, |pattern, bindings, row| {
                let members = subject_list_members(&pattern.s, bindings, row)?;
                if members.len() != 2 {
                    return None;
                }
                let list1 = VarOrTerm::list_members(members[0])?;
                let list2 = VarOrTerm::list_members(members[1])?;
                let mut combined: Vec<VarOrTerm> = Vec::with_capacity(list1.len() + list2.len());
                for id in list1.into_iter().chain(list2.into_iter()) {
                    combined.push(VarOrTerm::new_encoded_term(id));
                }
                Some(VarOrTerm::new_list(combined).to_encoded())
            }),
        }
    }

    // -- operand resolution -------------------------------------------------

    /// Resolve a pattern operand (subject/object) to a concrete encoded value
    /// id for the given row: variables are looked up in `bindings`, ground
    /// terms (including list/formula handles) are already resolved.
    fn resolve_operand(term: &VarOrTerm, bindings: &Binding, row: usize) -> Option<usize> {
        if term.is_var() {
            bindings.get(&term.to_encoded())?.get(row).copied()
        } else {
            Some(term.to_encoded())
        }
    }

    /// Resolve a list-term operand (subject of math:sum/string:concat/list:length)
    /// to its ordered, row-resolved member value ids. Members that are
    /// themselves variables (e.g. `( ?p1 ?p2 )`) are looked up against `row`;
    /// members that are ground terms (e.g. the literal " " in
    /// `( ?fn " " ?ln )`) are used as-is.
    fn subject_list_members(term: &VarOrTerm, bindings: &Binding, row: usize) -> Option<Vec<usize>> {
        let list_id = resolve_operand(term, bindings, row)?;
        let member_ids = VarOrTerm::list_members(list_id)?;
        let mut resolved = Vec::with_capacity(member_ids.len());
        for m in member_ids {
            if let Some(vals) = bindings.get(&m) {
                resolved.push(*vals.get(row)?);
            } else {
                resolved.push(m);
            }
        }
        Some(resolved)
    }

    fn numeric_value(id: usize) -> Option<f64> {
        let lex = match Encoder::decode_to_term(id) {
            Some(Term::Literal(lit)) => Encoder::decode(&lit.value)?,
            _ => Encoder::decode(&id)?,
        };
        lex.trim().trim_matches(|c| c == '<' || c == '>').parse::<f64>().ok()
    }

    fn lexical_value(id: usize) -> Option<String> {
        match Encoder::decode_to_term(id) {
            Some(Term::Literal(lit)) => Encoder::decode(&lit.value),
            _ => Encoder::decode(&id),
        }
    }

    fn intern_number(value: f64) -> usize {
        let is_whole = value.fract() == 0.0 && value.abs() < 1e15;
        let lexical = if is_whole {
            format!("{}", value as i64)
        } else {
            format!("{}", value)
        };
        let datatype = if is_whole {
            "<http://www.w3.org/2001/XMLSchema#integer>"
        } else {
            "<http://www.w3.org/2001/XMLSchema#decimal>"
        };
        VarOrTerm::new_literal(lexical, Some(datatype.to_string()), None).to_encoded()
    }

    fn intern_string(value: String) -> usize {
        VarOrTerm::new_literal(
            value,
            Some("<http://www.w3.org/2001/XMLSchema#string>".to_string()),
            None,
        )
        .to_encoded()
    }

    /// Copy every column's value at `row` from `bindings` into `out` -- used
    /// to carry forward already-bound variables when a builtin filters rows
    /// (constraints) or expands them (list:in).
    fn copy_row(bindings: &Binding, row: usize, out: &mut Binding) {
        for (&k, v) in bindings.iter() {
            if let Some(&val) = v.get(row) {
                out.add(&k, val);
            }
        }
    }

    // -- constraint builtins (log:equalTo, math:greaterThan) ----------------

    fn eval_row_constraint(
        pattern: &Triple,
        bindings: &Binding,
        check: impl Fn(usize, usize) -> bool,
    ) -> Option<Binding> {
        if bindings.len() == 0 {
            // No prior rows: treat as a single ground/implicit row. A
            // successful check yields "1 matched row with 0 columns" (same
            // convention TripleIndex::query uses for a matched ground triple).
            let s = resolve_operand(&pattern.s, bindings, 0)?;
            let o = resolve_operand(&pattern.o, bindings, 0)?;
            return if check(s, o) { Some(Binding::new()) } else { None };
        }
        let mut result = Binding::new();
        for row in 0..bindings.len() {
            if let (Some(s), Some(o)) = (
                resolve_operand(&pattern.s, bindings, row),
                resolve_operand(&pattern.o, bindings, row),
            ) {
                if check(s, o) {
                    copy_row(bindings, row, &mut result);
                }
            }
        }
        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    // -- functional builtins (math:sum, string:length/concat, list:length) --

    fn eval_functional(
        pattern: &Triple,
        bindings: &Binding,
        compute: impl Fn(&Triple, &Binding, usize) -> Option<usize>,
    ) -> Option<Binding> {
        if !pattern.o.is_var() {
            return None;
        }
        let obj_var = pattern.o.to_encoded();
        if bindings.len() == 0 {
            let value = compute(pattern, bindings, 0)?;
            let mut result = Binding::new();
            result.add(&obj_var, value);
            return Some(result);
        }
        let mut result = Binding::new();
        for row in 0..bindings.len() {
            if let Some(value) = compute(pattern, bindings, row) {
                copy_row(bindings, row, &mut result);
                result.add(&obj_var, value);
            }
        }
        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }

    // -- generator builtin (list:in) -----------------------------------------

    fn eval_generator(pattern: &Triple, bindings: &Binding) -> Option<Binding> {
        if !pattern.s.is_var() {
            return None;
        }
        let subj_var = pattern.s.to_encoded();
        if bindings.len() == 0 {
            let list_id = resolve_operand(&pattern.o, bindings, 0)?;
            let members = VarOrTerm::list_members(list_id)?;
            let mut result = Binding::new();
            for m in members {
                result.add(&subj_var, m);
            }
            return if result.len() > 0 { Some(result) } else { None };
        }
        let mut result = Binding::new();
        for row in 0..bindings.len() {
            if let Some(list_id) = resolve_operand(&pattern.o, bindings, row) {
                if let Some(members) = VarOrTerm::list_members(list_id) {
                    for m in members {
                        copy_row(bindings, row, &mut result);
                        result.add(&subj_var, m);
                    }
                }
            }
        }
        if result.len() > 0 {
            Some(result)
        } else {
            None
        }
    }
}
