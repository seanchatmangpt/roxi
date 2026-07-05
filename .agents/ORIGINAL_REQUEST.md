# Original User Request

## Initial Request — 2026-07-05T02:33:00Z

Implement 10 dependency-ordered extension and conformance tickets for the `roxi` RDF engine to support SHACL, ShEx, Datalog, and N3, plus SPARQL 1.1 conformance and cycle-safe backward chaining.

Working directory: /Users/sac/roxi
Integrity mode: development

## Requirements

### R1. Term Model Support for Literals and Blank Nodes
Extend the internal term model of the engine to support typed literals (with datatypes and language tags) and blank nodes. The interning encoder and query/reasoning/matching machinery must preserve kind-level and value-level distinctions.

### R2. Cycle-Safe Rule Evaluation
Implement cycle-detection and termination guards in both the backward-chaining rule engine and the hierarchical rule-composition components to prevent infinite recursion during cyclic query paths.

### R3. Advanced Datalog Engine
Implement stratified negation, aggregates (including grouping), and rule-safety verification for Datalog queries. The reasoning engine must stratified-evaluate rules and reject unstratifiable or unsafe rules with informative errors.

### R4. Full Notation3 (N3) Specification
Implement complete N3 grammar parsing and reasoning, including support for lists, multi-triple heads, quantifiers, quoted graphs (formulae), and standard built-ins (`log:`, `math:`, `list:`, `string:`).

### R5. SPARQL 1.1 Compliance
Ensure complete query evaluation and update behavior matching the W3C SPARQL 1.1 specification.

### R6. SHACL and ShEx Validation
Implement shape validation functionality for the engine. For SHACL, validate against standard core and SPARQL-based constraints. For ShEx, validate shapes utilizing ShapeMaps and recursive shape references. Use the `rudof` ecosystem crates via an adapter layer.

### R7. Continuous Conformance Gate
Configure a repository CI workflow that automatically validates all five dialect conformance suites on pull requests and commits.

## Acceptance Criteria

### Conformance Suites (100% Pass Rate Required)
- [ ] **SPARQL 1.1**: The W3C `sparql11-test-suite` evaluation and update tests must pass 100% (excluding only spec-defined optional features with written justification).
- [ ] **Datalog**: 100% pass on a comprehensive custom conformance suite covering safety checks, stratified negation patterns (accepted and rejected), and aggregates with recursion.
- [ ] **N3**: 100% pass on the EYE/N3 community test corpus.
- [ ] **SHACL**: 100% pass on the W3C `data-shapes` SHACL test suite.
- [ ] **ShEx**: 100% pass on the `shexTest` conformance suite.
- [ ] **Cycle Guard verification**: Cyclic rules do not stack-overflow or hang, returning terminated results/errors.
- [ ] **CI Pipeline**: A `.github/workflows/conformance.yml` workflow successfully executes all five conformance test commands and generates a unified `CONFORMANCE-STATUS.md` report.

## Follow-up — 2026-07-05T02:46:03Z

The user requests that the system maximize parallelism, perform more work, and allocate more subagents to parallelizable tasks. 

Since TICKET-001 is now complete, please instruct the Project Orchestrator (b064e2cc-2bc3-4e64-859a-4eef2a4b6b7e) to aggressively spawn parallel workers for all currently unblocked tickets (e.g. TICKET-002, TICKET-007) and prepare to launch multiple parallel developers for SHACL, ShEx, and SPARQL 1.1 tasks as soon as their respective adapter dependencies clear. Raise the active worker pool size limit to accommodate this.

