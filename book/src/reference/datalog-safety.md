# Datalog Rule Safety Verification

## 1. The Safety Problem in Logic Programming

In Datalog, rules are executed over a database of facts to derive new facts. If a rule is unrestricted, it can potentially generate an infinite number of derived facts, or facts containing variables that cannot be bound to any concrete value in the database. 

Consider the following rules:

$$\text{unsafe1}(?X, ?Y) \leftarrow \text{person}(?X)$$
$$\text{unsafe2}(?X) \leftarrow \text{person}(?X), \neg \text{employee}(?Y)$$

* In $\text{unsafe1}$, the variable $?Y$ appears in the head but does not appear in any positive body literal. If we try to ground this rule, $?Y$ can match any value from an infinite domain, making evaluation impossible.
* In $\text{unsafe2}$, the variable $?Y$ appears in a negated body literal but does not appear in any positive body literal. The database cannot bind $?Y$ to verify its absence, leading to an infinite domain of checkable values.

To guarantee that all Datalog evaluations terminate in finite time and yield well-defined relations, Roxi enforces the standard **Datalog Safety Condition** at rule-loading time.

---

## 2. The Datalog Safety Rules

A rule in Roxi is considered **safe** if and only if it satisfies the following three conditions:

1. **Head Variable Safety**: Every variable that appears in the **head** of the rule must also appear in at least one **positive body literal**.
2. **Negated Literal Safety**: Every variable that appears in a **negated body literal** (`not Predicate(...)`) must also appear in at least one **positive body literal**.
3. **Aggregate Variable Safety**: Every variable that appears as a grouping key in the head or inside an aggregate function must also appear in at least one **positive body literal**.

Positive body literals act as "binders." They restrict the search space for all variables to the active domain of the database (the set of all constants present in the index).

---

## 3. Verification Pipeline

The safety verification pipeline in [lib/src/ruleindex.rs](file:///Users/sac/roxi/lib/src/ruleindex.rs) executes during rule compilation:

```
          Rule String
               |
               v
          Pest Parser
               |
               v
      Raw Rule Structure
               |
               v
   Collect Positive Variables (PVar)
   Collect Head & Negated Variables (HVar, NVar)
               |
               +---> Check: HVar subset of PVar?
               +---> Check: NVar subset of PVar?
               /                               \\
           [Pass]                            [Fail]
             /                                  \\
     Insert into RuleIndex            Abort & Return RuleSafetyError
```

---

## 4. Rust Verification Code Reference

Below is the Rust code implementing the safety verifier in Roxi:

```rust
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Variable(String),
    Constant(usize),
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub predicate: usize,
    pub terms: Vec<Term>,
    pub is_negated: bool,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub head_predicate: usize,
    pub head_terms: Vec<Term>,
    pub body: Vec<Literal>,
}

#[derive(Debug)]
pub enum RuleSafetyError {
    UnboundHeadVariable(String),
    UnboundNegatedVariable(String),
}

impl Rule {
    /// Verifies the safety of the rule. Returns Ok(()) or a RuleSafetyError.
    pub fn verify_safety(&self) -> Result<(), RuleSafetyError> {
        // 1. Collect all variables bound in positive body literals
        let mut bound_vars = HashSet::new();
        for literal in &self.body {
            if !literal.is_negated {
                for term in &literal.terms {
                    if let Term::Variable(name) = term {
                        bound_vars.insert(name.clone());
                    }
                }
            }
        }

        // 2. Verify all variables in the head are bound
        for term in &self.head_terms {
            if let Term::Variable(name) = term {
                if !bound_vars.contains(name) {
                    return Err(RuleSafetyError::UnboundHeadVariable(name.clone()));
                }
            }
        }

        // 3. Verify all variables in negated body literals are bound
        for literal in &self.body {
            if literal.is_negated {
                for term in &literal.terms {
                    if let Term::Variable(name) = term {
                        if !bound_vars.contains(name) {
                            return Err(RuleSafetyError::UnboundNegatedVariable(name.clone()));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
```

---

## 5. Safe vs. Unsafe Rule Examples

### Safe Rule (Valid)
```prolog
# All variables (?x, ?y) are bound in positive body literals.
hasManager(?x, ?y) :- employee(?x), manages(?y, ?x).
```

### Unsafe Rule (Rejected)
```prolog
# ?y is not present in a positive body literal.
# Generates RuleSafetyError::UnboundHeadVariable("y")
worksWith(?x, ?y) :- employee(?x).
```

### Unsafe Negation (Rejected)
```prolog
# ?y in the negated literal is not bound by any positive body literal.
# Generates RuleSafetyError::UnboundNegatedVariable("y")
unassigned(?x) :- job(?x), not assigns(?y, ?x).
```
