# SPARQL 1.1 Conformance Spike Report (TICKET-006)

This report details the findings from the read-only spike on the SPARQL 1.1 Conformance milestone. It covers the current state of tests, the engine implementation gaps, and a concrete plan for achieving 100% W3C conformance.

---

## 1. Executive Summary
The `roxi` SPARQL engine (`lib/src/sparql.rs`) uses `spargebra` for parsing but lacks basic support for several fundamental SPARQL 1.1 features.
- No cached W3C SPARQL 1.1 test suite exists in the repository or home directories.
- The current integration test runner is a mock setup containing a single hardcoded SELECT query verification.
- The core engine contains critical correctness bugs, including:
  1. A **panic** when executing cross joins (empty variable intersection).
  2. A **correctness bug** where joins on multiple shared variables only evaluate the first shared variable.
  3. A **correctness bug** where only the first aggregate function in a SELECT projection is evaluated, ignoring all subsequent ones.
  4. An **inconsistent variable encoding ID mismatch** where variable names are registered with and without the `?` prefix, causing lookup failures in BIND/Extend logic.
  5. The absence of **Effective Boolean Value (EBV)** logic in `FILTER` operations.

---

## 2. Test Suite & Test Runner Findings

### 2.1 Search for W3C Test Suite
- No cached copy of the official W3C SPARQL 1.1 Query Evaluation test suite was found in the workspace `/Users/sac/roxi` or the parent directory `/Users/sac`.
- The current structure in `lib/tests/sparql11_conformance` contains a single integration test (`main.rs`) and a folder `mock_suite` with:
  - `data.ttl`: A tiny Turtle graph (2 triples).
  - `query.rq`: A simple SELECT query.
  - `expected.json`: A static file showing the expected bindings.
- **Mock Behavior**: The runner in `main.rs` reads `query.rq` and `data.ttl` but completely ignores `expected.json`. The verification is hardcoded via Rust assertions. It then writes a hardcoded Markdown manifest report to `docs/jira/26.7.4/manifests/sparql11_manifest.md`.

### 2.2 Proposed Test Suite Structure
We recommend replacing the mock suite with a robust, data-driven runner resembling the N3 parser conformance runner (`n3_conformance`). 
We have drafted and written two proposed artifacts inside the explorer's folder:
1. `proposed_manifest.json`: Defines a suite of W3C-like query evaluation test cases covering:
   - OPTIONAL (LeftJoin)
   - UNION
   - MINUS
   - Subqueries
   - BIND (Extend)
   - FILTER (Equality, EBV, and Logical AND/OR/NOT)
   - Aggregates (Multiple functions with GROUP BY)
2. `proposed_test_runner.rs`: A generic Rust test runner that reads the JSON manifest, dynamically executes the tests, parses `expected.json` files as raw bindings, compares results in an order-independent set-based manner, and generates a dynamic markdown pass-rate report.

---

## 3. SPARQL Engine Implementation Gaps (`lib/src/sparql.rs`)

We analyzed the current implementation of `lib/src/sparql.rs` and identified the following gaps, categorized by engine component:

### 3.1 GraphPattern Gaps
In `extract_query_plan(graph_pattern: &GraphPattern) -> PlanNode`, the following variants map directly to `PlanNode::Done` (fallback arm) and are unsupported:
- `LeftJoin`: Core primitive for `OPTIONAL`.
- `Union`: Core primitive for `UNION`.
- `Minus`: Core primitive for `MINUS`.
- `Extend`: Only supports `Expression::Variable` renaming. Standard BIND expressions (e.g., `BIND(5 AS ?x)` or arithmetic) return `PlanNode::Done`. Additionally, the existing `Extend` execution replaces/deletes the original variable instead of keeping it and extending the bindings.
- `OrderBy`: Ordering expressions are unsupported.
- `Distinct` and `Reduced`: Duplicate elimination modifiers are unsupported.
- `Slice`: Used for `LIMIT` and `OFFSET`. Unsupported.
- `Values`: Inline data (`VALUES`) is unsupported.
- `Path`: Property paths are unsupported.
- `Service`: Federated `SERVICE` is unsupported.
- `Query::Ask`, `Query::Construct`, `Query::Describe`: `eval_query` maps all query types except `Query::Select` to `PlanNode::Done`.

### 3.2 Join Execution Bugs
In `evaluate_plan` for `PlanNode::Join { left, right }` (lines 280-327):
- **Cross-Join Panic**: When joining two patterns that share no variables (e.g. cross product), the intersection of variables is empty. The code attempts to access the first shared variable:
  ```rust
  if binding.var == *intersection.get(0).unwrap()
  ```
  This call to `.unwrap()` causes a panic during query execution.
- **Incorrect Multi-Variable Joins**: If two patterns share multiple variables, the intersection vector contains multiple elements. The code only hashes and matches on the *first* shared variable:
  ```rust
  if binding.var == *intersection.get(0).unwrap()
  ```
  This ignores all subsequent shared variables, returning incorrect result bindings that violate the join constraints.

### 3.3 Aggregate Execution Gaps
In `evaluate_plan` for `PlanNode::Aggregate` (lines 339-445):
- **Single Aggregate Limitation**: The code extracts only the first aggregate:
  ```rust
  let (aggregate_function, aggregate_var) = aggregates.iter().next().unwrap();
  ```
  If a query defines multiple aggregates (e.g., `SELECT (COUNT(?s) AS ?count) (SUM(?val) AS ?sum)`), all aggregates beyond the first one are silently ignored.
- **Empty Aggregates Panic**: If a query has a `GROUP BY` clause but no aggregate functions (which is syntactically valid in SPARQL), `aggregates` is empty, and `.unwrap()` panics.

### 3.4 Expression & Filter Gaps
In `extract_expression` (lines 210-241) and `eval_expression` (lines 614-675):
- **Missing Operators**: Supports only `Greater`, `GreaterOrEqual`, `Less`, and `LessOrEqual`. Missing `Equal`, `NotEqual`, `And`, `Or`, `Not`, arithmetic operators, and built-in functions (e.g., `bound()`, `coalesce()`).
- **No Effective Boolean Value (EBV)**: The `Filter` execution checks specifically for `Some(EncodedTerm::BooleanLiteral(true))` (line 332). The `to_bool` function is defined but never called, meaning string and numeric values are never coercion-tested to their boolean equivalence.
- **Numeric Comparisons**: Compares integer terms or strings but does not support float type promotions or datetime comparisons.

### 3.5 Variable Mismatch Mismatch (Lookups)
There is a critical mismatch in how variable names are registered in the global `Encoder`:
- In `Project` and `VarOrTerm::convert`, variables are added via their names without `?` (e.g., `"title"`). Since they don't start with `?`, the encoder registers them as `EncodedValue::Iri`.
- In `Extend` (line 160), `spargebra::Expression::Variable::into_string()` returns `"?var"`. The encoder adds it as `EncodedValue::Variable("?var")`.
- In `PlanNode::Extend` evaluation, `from.as_str()` returns the variable name without `?` (e.g. `"var"`), and calls `Encoder::get("var")`. Because it doesn't start with `?`, the lookup looks for `EncodedValue::Iri("var")`, which mismatches `EncodedValue::Variable("?var")`, leading to evaluation errors.

---

## 4. Conformance Test Execution

Running the project test command requested in the task:
```bash
cargo test --workspace --lib --bins sparql11_conformance
```
- **Behavior**: Cargo compiles the library and workspace binaries but does not run the `sparql11_conformance` integration test, because integration tests are not included under `--lib --bins`. 
- **Correct Execution**: To execute the SPARQL integration test target, run:
  ```bash
  cargo test --test sparql11_conformance
  ```
  This command executes the single test in `lib/tests/sparql11_conformance/main.rs`, yielding:
  `test test_sparql11_conformance ... ok`.

---

## 5. Gap Inventory & Recommended Implementation Plan

### 5.1 Gap Inventory
To achieve W3C SPARQL 1.1 Query Evaluation conformance, the following elements must be implemented:
1. **Joins**: Resolve the empty intersection panic (perform Cartesian product) and properly match all shared variables during Hash Join.
2. **Aggregates**: Rewrite `PlanNode::Aggregate` evaluation to loop over and evaluate all projection aggregates, not just the first one. Support empty aggregates in `GROUP BY`.
3. **Extend/BIND**: Standardize variable registration in the `Encoder` (strip `?` globally for variable lookups). Resolve the `Extend` node to keep other bindings intact and only add the extended binding.
4. **LeftJoin/OPTIONAL**: Add `PlanNode::LeftJoin { left: Box<Self>, right: Box<Self>, expression: Option<PlanExpression> }` and implement a left outer join iterator.
5. **Union/UNION**: Add `PlanNode::Union { left: Box<Self>, right: Box<Self> }` and implement a flat-map chaining iterator.
6. **Minus/MINUS**: Add `PlanNode::Minus { left: Box<Self>, right: Box<Self> }` and implement an anti-join filter iterator.
7. **Filter & EBV**: Add support for logical operators, equality, inequality, and evaluate expressions using `to_bool` (EBV rules).

### 5.2 Recommended Implementation Steps for the Implementer

1. **Step 1: Set Up the Test Cases**
   - Copy `proposed_manifest.json` and `proposed_test_runner.rs` from `.agents/explorer_sparql11_conformance_1/` into `lib/tests/sparql11_conformance/`.
   - Populate `lib/tests/sparql11_conformance/` with the query, data, and expected JSON files for the 10 planned conformance cases.
2. **Step 2: Correct Joining & Aggregates**
   - Fix the empty variable intersection case in `PlanNode::Join` to compute the Cartesian product of left and right bindings.
   - Modify the hash join logic to check for equivalence across all intersection variables.
   - Refactor `PlanNode::Aggregate` to keep a list of accumulators for all requested aggregates, computing and projecting them collectively.
3. **Step 3: Unify Variable Encoding**
   - Standardize variables in the `Encoder` so lookups do not mismatch on the presence/absence of `?`.
   - Fix `PlanNode::Extend` to append the evaluated binding to the existing binding vector instead of substituting it.
4. **Step 4: Implement LeftJoin, Union, and Minus**
   - Add the new variants to `PlanNode` and map them in `extract_query_plan`.
   - Write corresponding iterators in `evaluate_plan`:
     - **Union**: Chain the left iterator and right iterator.
     - **LeftJoin**: For each binding in the left iterator, find matches in the right iterator. If none match (or if the filter expression evaluates to false), output the left binding with the right variables unbound.
     - **Minus**: For each binding in the left iterator, check if there is a compatible binding in the right iterator that shares at least one variable. If so, discard the left binding; otherwise, keep it.
5. **Step 5: Implement Filter Expressions and EBV**
   - Map `Expression::Equal`, `Expression::NotEqual`, `Expression::And`, `Expression::Or`, and `Expression::Not` to `PlanExpression`.
   - Update `eval_expression` to evaluate these operators and use the EBV `to_bool` logic during filtering.
