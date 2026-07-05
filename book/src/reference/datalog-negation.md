# Stratified Negation in Roxi Datalog

## 1. Theoretical Foundations

In traditional logic programming and relational databases, negation-as-failure (NAF) is a query-time rule: if a fact cannot be proven to be true after exhaustive search, it is assumed to be false. While NAF is intuitive for non-recursive query blocks, it introduces semantic ambiguity and logical contradictions when combined with recursive rules.

Consider the classic self-negation loop:
$$p \leftarrow \neg p$$

Under a standard forward-chaining evaluation:
* If we assume $p$ is false, the rule derives that $p$ is true.
* If we assume $p$ is true, the rule cannot derive $p$, implying $p$ is false.

This contradiction means the system has no stable model. To resolve this, Roxi enforces **stratified negation**. Stratification partitions the predicates in a program into a sequence of layers (strata) such that negation is only applied to predicates whose evaluation has already been fully completed and "fixed" in a lower stratum.

Specifically, a Datalog program is stratifiable if there exists a partitioning of its predicates into strata $S_0, S_1, \dots, S_n$ such that for every rule:
$$P \leftarrow Q_1, \dots, Q_k, \neg R_1, \dots, \neg R_m$$
1. The stratum of any positive body predicate $Q_i$ is less than or equal to the stratum of the head predicate $P$:
   $$\text{stratum}(Q_i) \le \text{stratum}(P)$$
2. The stratum of any negated body predicate $R_j$ is strictly less than the stratum of the head predicate $P$:
   $$\text{stratum}(R_j) < \text{stratum}(P)$$

---

## 2. Predicate Dependency Graph (PDG)

To verify stratifiability and compute the stratum assignments, Roxi's Datalog engine compiles the rules into a **Predicate Dependency Graph (PDG)** at load time.

* **Nodes**: Nodes in the graph represent unique predicates (e.g., table names, relations).
* **Positive Edges ($P \xrightarrow{+} Q$)**: Directed edge from $Q$ to $P$ if $Q$ appears as a positive literal in the body of a rule where $P$ is the head. This indicates that deriving $P$ depends positively on the derivation of $Q$.
* **Negative Edges ($P \xrightarrow{-} Q$)**: Directed edge from $Q$ to $P$ if $Q$ appears as a negated literal (`not Q(...)`) in the body of a rule where $P$ is the head. This indicates that deriving $P$ depends on the *absence* of $Q$.

```
       +-----------------+
       |   Predicate A   |
       +-----------------+
          /           \
   (+)   /             \   (-)
        v               v
+-----------------+   +-----------------+
|   Predicate B   |   |   Predicate C   |
+-----------------+   +-----------------+
```

---

## 3. Stratum Assignment Algorithm

Roxi executes the following algorithm in [lib/src/datalog.rs](file:///Users/sac/roxi/lib/src/datalog.rs) to calculate the topological strata:

1. **Cycle Detection (Strongly Connected Components)**: Run Tarjan's or Kosaraju's algorithm to identify all Strongly Connected Components (SCCs) in the PDG.
2. **Negation Cycle Verification**: Inspect each SCC. If any SCC contains a cycle that passes through a **negative edge**, the program is **unstratifiable**. The engine aborts rule-loading and returns an `UnstratifiableRulesError`.
3. **Topological Sort**: Collapse the SCCs into a directed acyclic graph (DAG) and perform a topological sort.
4. **Stratum Calculation**:
   * Set $\text{stratum}(P) = 0$ for all predicates that have no dependencies.
   * For each predicate $P$ in topological order, calculate its stratum based on its dependencies:
     $$\text{stratum}(P) = \max \left( \{ \text{stratum}(Q) \mid P \xrightarrow{+} Q \} \cup \{ \text{stratum}(R) + 1 \mid P \xrightarrow{-} R \} \right)$$

---

## 4. Evaluation Engine

Once the strata are computed, the forward-chaining `Reasoner` in [lib/src/reasoner.rs](file:///Users/sac/roxi/lib/src/reasoner.rs) evaluates the database stratum-by-stratum:

```
[Base Facts]
     |
     v
[Evaluate Stratum 0] ----> Fixpoint reached (Facts of Stratum 0 are locked)
     |
     v
[Evaluate Stratum 1] ----> Negated checks evaluate against locked Stratum 0 facts
     |
     v
[Evaluate Stratum 2] ----> Negated checks evaluate against locked Stratum 0 & 1 facts
```

During the evaluation of Stratum $i$:
* **Positive matching**: Pattern queries are evaluated dynamically against the entire `TripleIndex`.
* **Negated matching**: When matching a negated literal like `not Q(?x, ?y)`, the engine checks if the relation is present in the database. Because $Q$ belongs to a lower stratum ($< i$), its evaluation is guaranteed to be complete. If $Q(?x, ?y)$ is not found in the index, the negated check succeeds.

---

## 5. Rust Implementation Reference

Below is the Rust structural design of the stratification verification pass in `datalog.rs`:

```rust
use std::collections::{HashMap, HashSet};

pub type PredicateId = usize;

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub target: PredicateId,
    pub is_negated: bool,
}

pub struct PredicateDependencyGraph {
    pub adj_list: HashMap<PredicateId, Vec<DependencyEdge>>,
}

impl PredicateDependencyGraph {
    pub fn new() -> Self {
        Self {
            adj_list: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, head: PredicateId, body: PredicateId, is_negated: bool) {
        self.adj_list
            .entry(head)
            .or_default()
            .push(DependencyEdge {
                target: body,
                is_negated,
            });
    }

    /// Computes strata or returns Err if an unstratifiable negative cycle is detected.
    pub fn compute_strata(&self) -> Result<HashMap<PredicateId, usize>, DatalogError> {
        let mut strata = HashMap::new();
        // 1. Identify Strongly Connected Components (SCC)
        let sccs = self.find_sccs();
        
        // 2. Verify no negative edges inside any SCC
        for scc in &sccs {
            if self.has_negative_cycle_in_scc(scc) {
                return Err(DatalogError::Unstratifiable(
                    "Negation through a recursive cycle detected!".to_string()
                ));
            }
        }

        // 3. Compute topological strata assignment
        for scc in sccs {
            for &node in &scc {
                let mut max_stratum = 0;
                if let Some(edges) = self.adj_list.get(&node) {
                    for edge in edges {
                        let dep_stratum = strata.get(&edge.target).cloned().unwrap_or(0);
                        let cost = if edge.is_negated { 1 } else { 0 };
                        max_stratum = max_stratum.max(dep_stratum + cost);
                    }
                }
                strata.insert(node, max_stratum);
            }
        }

        Ok(strata)
    }
}
```
