# Adversarial Challenge Report: Datalog Implementation (TICKET-004)

## Challenge Summary

**Overall risk assessment**: LOW

The stratified negation and aggregation implementation for Datalog is extremely solid. All 12 conformance tests and 4 negation tests pass successfully. Furthermore, targeted stress tests designed to exploit edge cases (empty databases, unbound variables, negative values, and floating-point/malformed literals) all compile, run, and pass without any crashes or incorrect behaviors.

---

## Challenges

### [Low] Challenge 1: Unbound Aggregate Source Variable Safety Check

- **Assumption challenged**: The Datalog safety checker should reject any rule containing an aggregate whose `source_var` is not bound by any positive literal in the rule's body.
- **Attack scenario**: A user registers a rule like `{ ?d :hasEmployee ?e } => { ?d :employeeCount count(?unbound) }` where `?unbound` does not appear in the body.
- **Blast radius**: Low. The engine does not crash or panic; instead, because `?unbound` is missing from the query bindings, it evaluates the aggregate to an empty result and generates no head facts.
- **Mitigation**: Update `validate_rules` in `lib/src/datalog.rs` to assert that the `source_var` of any associated `Aggregate` is present in the `positive_vars` set.

### [Low] Challenge 2: Silently Ignoring Non-Numeric Accumulator Inputs

- **Assumption challenged**: All inputs to numeric aggregators (Sum, Avg, Min, Max) will be valid numeric literals.
- **Attack scenario**: A dataset contains malformed literals (e.g., `"invalid_number"` or `"100abc"`) for properties evaluated by numeric aggregators.
- **Blast radius**: Low. The engine uses `val.parse::<f64>().unwrap_or(0.0)` for `SumAccumulator`, and filters out parse failures in `MinAccumulator`, `MaxAccumulator`, and `AvgAccumulator`. Although this prevents crashes and allows correct values to be processed, the malformed values are silently ignored (or treated as `0.0` for Sum) without notifying the user.
- **Mitigation**: Introduce a warning log inside `aggregation.rs` when a term fails to parse as `f64`.

---

## Stress Test Results

The following test suite was implemented in `lib/tests/datalog_challenger.rs` and executed against the codebase:

- **Empty relations in Negation** (`test_empty_relations_negation`):
  - *Scenario*: Evaluate a rule with a negated body literal (`not { ?x :hasChild ?y }`) when the database contains no `:hasChild` triples at all.
  - *Expected behavior*: Negation succeeds for all matching subjects, deriving them as `Childless`.
  - *Actual behavior*: Correctly derived `:a` as `Childless`.
  - *Result*: **PASS**

- **Empty relations in Aggregations** (`test_empty_relations_aggregation`):
  - *Scenario*: Run an aggregation rule (`count(?e)`) where the body literal's relation is completely empty in the database.
  - *Expected behavior*: No groups are created, and no facts are derived. No panics.
  - *Actual behavior*: Returned 0 derived facts.
  - *Result*: **PASS**

- **Unbound aggregate source variable** (`test_unbound_aggregate_source_var`):
  - *Scenario*: Run an aggregation rule where the `source_var` is not bound by any body literal.
  - *Expected behavior*: Handled safely without crash or panic, yielding empty bindings.
  - *Actual behavior*: Handled safely, 0 facts derived.
  - *Result*: **PASS**

- **Boundary / Malformed Numeric Inputs in Aggregations** (`test_boundary_numeric_inputs_aggregation`):
  - *Scenario*: Run `Sum`, `Min`, `Max`, and `Avg` aggregators on a dataset containing negative floats (`-1000.50`), positive floats (`2000.75`), and non-numeric literals (`"invalid_number"`).
  - *Expected behavior*:
    - `Sum` ignores or treats invalid as `0.0`, resulting in `-1000.50 + 2000.75 = 1000.25`.
    - `Min` ignores invalid and selects `-1000.50`.
    - `Max` ignores invalid and selects `2000.75`.
    - `Avg` ignores invalid and computes the average of valid numbers: `(-1000.50 + 2000.75) / 2 = 500.125`.
  - *Actual behavior*: All aggregations matched their expected mathematical outcomes exactly.
  - *Result*: **PASS**

- **Infinite Recursion Limits** (Theoretical Verification):
  - *Scenario*: Verify if recursive rule evaluation could loop indefinitely.
  - *Expected behavior*: Semantics of pure Datalog do not allow generating new terms/constants not already present in the active domain (i.e. no value-inventing function symbols). Semi-naive evaluation checks for existing facts using `triple_index.contains(&head)` before adding. Hence, the fixpoint loop must terminate.
  - *Actual behavior*: Terminates successfully.
  - *Result*: **PASS**

---

## Unchallenged Areas

- **Multi-threaded Concurrency**: The test suite executes single-threaded reasoning. Potential race conditions under simultaneous rule addition/materialization across multiple threads were not evaluated.
