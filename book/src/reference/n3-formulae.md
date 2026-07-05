# N3 Formulae & Quoted Graphs

## 1. Nested Statements & Metadata

In standard RDF, graphs are flat collections of triples. Notation3 (N3) introduces **formulas** (also called **quoted graphs**), which are sets of triple patterns wrapped in `{}` braces. This allows a graph to contain statements about other statements as first-class terms.

Consider this N3 statement:

```turtle
{:alice :accessLevel :high} :assertedBy :securityOfficer .
```

Here:
* **Subject**: The quoted graph `{:alice :accessLevel :high}`.
* **Predicate**: `:assertedBy`.
* **Object**: `:securityOfficer`.

Quoted graphs are fundamental for expressing metadata, tracking provenance, asserting trust levels, and building logical rules that reason about context.

---

## 2. Compilation and Interning

Roxi represents quoted graphs natively in its type-safe term model [lib/src/triples.rs](file:///Users/sac/roxi/lib/src/triples.rs):

```rust
pub enum Term {
    Iri(TermImpl),
    Literal(LiteralImpl),
    BlankNode(BlankNodeImpl),
    Triple(Box<Triple>),
    Formula(FormulaImpl), // Native support for quoted graphs
}
```

```
                          Formula Term
                               |
                   +-----------+-----------+
                   |                       |
            Encoder ID: 99           Triples Vector
            (Interned Key)      [SPO 1, SPO 2, SPO N]
```

### The `FormulaImpl` representation:
* **`id`**: A unique interning identifier allocated by the `Encoder` to speed up comparison checks.
* **`triples`**: A vector of interned `Triple` structures contained within the formula.

This layout allows the query engine to perform fast matches on the formula identifier before evaluating its inner triples.

---

## 3. Query Matching and Variable Binding

Evaluating rules that match against quoted graphs requires matching variables at different levels of nesting:

```turtle
# Rule: If a statement is asserted by the security officer, assert it as true in the database.
{ ?statement :assertedBy :securityOfficer . ?statement } => { ?statement :status :verified } .
```

During matching:
1. The engine queries the index for triples matching `?statement :assertedBy :securityOfficer`.
2. `?statement` binds to the formula term `{:alice :accessLevel :high}`.
3. The rule evaluates the nested implication: the inner triple pattern `:alice :accessLevel :high` is extracted and inserted into the active database.
4. The conclusion `:alice :accessLevel :high :status :verified` is materialized.

This multi-level matching is executed efficiently by recursively evaluating query bindings across nested scopes.

---

## 4. Rust Implementation Reference

Below is the Rust structural design of formula terms and matching logic:

```rust
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Iri(usize),
    Literal(usize),
    Formula(FormulaImpl),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Triple {
    pub s: Term,
    pub p: Term,
    pub o: Term,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FormulaImpl {
    pub id: usize,
    pub triples: Vec<Triple>,
}

pub struct FormulaMatcher;

impl FormulaMatcher {
    /// Recursively checks if a pattern term matches a database term,
    /// binding variables across nested formula scopes.
    pub fn match_terms(
        pattern: &Term,
        data: &Term,
        bindings: &mut std::collections::HashMap<String, Term>,
    ) -> bool {
        match (pattern, data) {
            (Term::Iri(pid), Term::Iri(did)) => pid == did,
            (Term::Literal(pid), Term::Literal(did)) => pid == did,
            (Term::Formula(pf), Term::Formula(df)) => {
                if pf.triples.len() != df.triples.len() {
                    return false;
                }
                // Verify all nested triples match
                for (pt, dt) in pf.triples.iter().zip(df.triples.iter()) {
                    if !Self::match_triples(pt, dt, bindings) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub fn match_triples(
        pattern: &Triple,
        data: &Triple,
        bindings: &mut std::collections::HashMap<String, Term>,
    ) -> bool {
        Self::match_terms(&pattern.s, &data.s, bindings)
            && Self::match_terms(&pattern.p, &data.p, bindings)
            && Self::match_terms(&pattern.o, &data.o, bindings)
    }
}
```
