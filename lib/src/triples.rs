use crate::Encoder;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

// ============================================================================
// N3 built-in support: RDF lists and quoted graphs ("formulas")
// ============================================================================
//
// Design decision (TICKET-005): rather than adding new `Term` variants for
// lists and quoted graphs, both are represented as ordinary `Term::BlankNode`
// values whose (synthetic, process-unique) label acts as a key into a
// process-wide side table:
//   - `LIST_REGISTRY`:    blank-node id -> ordered member ids (Vec<usize>)
//   - `FORMULA_REGISTRY`: blank-node id -> the quoted graph's own triples
// This keeps `Term` a closed 3-variant enum (Iri/Literal/BlankNode), so the
// many exhaustive `match term { Term::Iri | Term::Literal | Term::BlankNode }`
// sites elsewhere in the crate (sparql.rs, oxrdf_adapter.rs, shacl.rs) need no
// changes. List members may themselves be *variables* (e.g. `( ?p1 ?p2 )` used
// as the subject of math:sum in a rule body) -- those are resolved against the
// current row's `Binding` at builtin-evaluation time (see queryengine.rs).
static LIST_REGISTRY: Lazy<Mutex<HashMap<usize, Vec<usize>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static FORMULA_REGISTRY: Lazy<Mutex<HashMap<usize, Vec<Triple>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static SYNTHETIC_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum VarOrTerm {
    Var(Variable),
    Term(Term),
}

impl VarOrTerm {
    pub fn new_term(iri: String) -> VarOrTerm {
        let term = Term::parse(iri);
        VarOrTerm::Term(term)
    }

    pub fn new_var(name: String) -> VarOrTerm {
        let encoded = Encoder::add(name);
        VarOrTerm::Var(Variable { name: encoded })
    }

    pub fn new_encoded_term(id: usize) -> VarOrTerm {
        let term = Encoder::decode_to_term(id).unwrap_or_else(|| {
            // Fallback for untracked IDs (e.g. in tests/mock setups)
            Term::Iri(TermImpl { iri: id })
        });
        VarOrTerm::Term(term)
    }

    pub fn new_encoded_var(name: usize) -> VarOrTerm {
        VarOrTerm::Var(Variable { name })
    }

    pub(crate) fn as_term(&self) -> &Term {
        if let VarOrTerm::Term(t) = self {
            t
        } else {
            panic!("Not a term")
        }
    }

    pub(crate) fn as_var(&self) -> &Variable {
        if let VarOrTerm::Var(v) = self {
            v
        } else {
            panic!("Not a Var")
        }
    }

    pub fn is_var(&self) -> bool {
        match self {
            Self::Var(_) => true,
            Self::Term(_) => false,
        }
    }

    pub fn is_term(&self) -> bool {
        !self.is_var()
    }

    pub fn to_encoded(&self) -> usize {
        match self {
            Self::Var(var) => var.name,
            Self::Term(term) => term.id(),
        }
    }

    fn rem_first_and_last(value: &str) -> &str {
        let mut chars = value.chars();
        chars.next();
        chars.next_back();
        chars.as_str()
    }

    pub fn convert(var_or_term: String) -> VarOrTerm {
        let var_or_term = var_or_term.trim().to_string();
        if var_or_term.starts_with('?') {
            let var_name = &var_or_term[1..];
            VarOrTerm::new_var(var_name.to_string())
        } else {
            let mut iri_prefix = var_or_term;
            if !iri_prefix.starts_with('<')
                && !iri_prefix.starts_with('"')
                && !iri_prefix.starts_with("_:")
            {
                iri_prefix = format!("<{}>", iri_prefix).to_string();
            }
            VarOrTerm::new_term(iri_prefix)
        }
    }

    pub fn new_literal(value: String, datatype: Option<String>, lang: Option<String>) -> VarOrTerm {
        let id = Encoder::add_literal(value, datatype, lang);
        let term = Encoder::decode_to_term(id).expect("Successfully decoded just-added literal");
        VarOrTerm::Term(term)
    }

    pub fn new_blank_node(label: String) -> VarOrTerm {
        let id = Encoder::add_blank_node(label);
        let term = Encoder::decode_to_term(id).expect("Successfully decoded just-added blank node");
        VarOrTerm::Term(term)
    }

    /// Build an RDF-list term from an ordered set of members (which may be
    /// variables or ground terms). See the module-level design note above.
    pub fn new_list(members: Vec<VarOrTerm>) -> VarOrTerm {
        let member_ids: Vec<usize> = members.iter().map(|m| m.to_encoded()).collect();
        let tag = SYNTHETIC_COUNTER.fetch_add(1, Ordering::SeqCst);
        let id = Encoder::add_blank_node(format!("__n3list_{}", tag));
        LIST_REGISTRY.lock().unwrap().insert(id, member_ids);
        VarOrTerm::new_encoded_term(id)
    }

    /// Build a quoted-graph ("formula") term from its constituent triples.
    /// See the module-level design note above.
    pub fn new_formula(triples: Vec<Triple>) -> VarOrTerm {
        let tag = SYNTHETIC_COUNTER.fetch_add(1, Ordering::SeqCst);
        let id = Encoder::add_blank_node(format!("__n3formula_{}", tag));
        FORMULA_REGISTRY.lock().unwrap().insert(id, triples);
        VarOrTerm::new_encoded_term(id)
    }

    /// If `id` names a synthetic list term, return its ordered member ids.
    pub fn list_members(id: usize) -> Option<Vec<usize>> {
        LIST_REGISTRY.lock().unwrap().get(&id).cloned()
    }

    /// If `id` names a synthetic formula (quoted graph) term, return its triples.
    pub fn formula_triples(id: usize) -> Option<Vec<Triple>> {
        FORMULA_REGISTRY.lock().unwrap().get(&id).cloned()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Variable {
    pub(crate) name: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Term {
    Iri(TermImpl),
    Literal(LiteralImpl),
    BlankNode(BlankNodeImpl),
}

impl Term {
    pub fn id(&self) -> usize {
        match self {
            Self::Iri(iri) => iri.iri,
            Self::Literal(lit) => lit.id,
            Self::BlankNode(bnode) => bnode.id,
        }
    }

    pub fn parse(s: String) -> Self {
        let id = Encoder::add(s);
        Encoder::decode_to_term(id).expect("Successfully decoded just-added term")
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = Encoder::decode(&self.id()) {
            write!(f, "{}", s)
        } else {
            write!(f, "Term({})", self.id())
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TermImpl {
    pub(crate) iri: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LiteralImpl {
    pub(crate) id: usize,
    pub(crate) value: usize,
    pub(crate) datatype: Option<usize>,
    pub(crate) lang: Option<usize>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BlankNodeImpl {
    pub(crate) id: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Triple {
    pub s: VarOrTerm,
    pub p: VarOrTerm,
    pub o: VarOrTerm,
    pub g: Option<VarOrTerm>,
}

impl Triple {
    pub fn from(subject: String, property: String, object: String) -> Triple {
        Triple {
            s: VarOrTerm::convert(subject),
            p: VarOrTerm::convert(property),
            o: VarOrTerm::convert(object),
            g: None,
        }
    }

    pub fn from_with_graph_name(
        subject: String,
        property: String,
        object: String,
        graph_name: String,
    ) -> Triple {
        let mut triple = Self::from(subject, property, object);
        triple.g = Some(VarOrTerm::convert(graph_name));
        triple
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BodyLiteral {
    pub negated: bool,
    pub pattern: Triple,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AggregateFunction {
    Count,
    Sum,
    Min,
    Max,
    Avg,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Aggregate {
    pub function: AggregateFunction,
    pub source_var: String,
    pub target_var: String,
    pub group_vars: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Rule {
    pub body: Vec<BodyLiteral>,
    pub head: Triple,
}

#[test]
fn test_literal_term_roundtrip() {
    let lit_term_str = "\"hello\"@en".to_string();
    let var_or_term = VarOrTerm::new_term(lit_term_str.clone());

    assert!(var_or_term.is_term());
    let term = var_or_term.as_term();
    assert!(matches!(term, Term::Literal(_)));

    let decoded = Encoder::decode(&term.id()).unwrap();
    assert_eq!(decoded, lit_term_str);
}

#[test]
fn test_blank_node_term_encoding() {
    let blank_node_str = "_:b0".to_string();
    let var_or_term = VarOrTerm::new_term(blank_node_str.clone());

    assert!(var_or_term.is_term());
    let term = var_or_term.as_term();
    assert!(matches!(term, Term::BlankNode(_)));

    let decoded = Encoder::decode(&term.id()).unwrap();
    assert_eq!(decoded, blank_node_str);
}
