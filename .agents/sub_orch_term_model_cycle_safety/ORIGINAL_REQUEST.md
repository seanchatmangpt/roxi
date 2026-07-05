# Original User Request

## Initial Request — 2026-07-04T19:34:27-07:00

Resume work at /Users/sac/roxi/.agents/sub_orch_term_model_cycle_safety.
Read SCOPE.md, global PROJECT.md, and the JIRA tickets for TICKET-001, TICKET-002, and TICKET-003.
Your identity: sub_orch_term_model_cycle_safety
Your parent: project_orchestrator
Your parent conversation ID: b064e2cc-2bc3-4e64-859a-4eef2a4b6b7e

Objective:
Implement and verify:
- TICKET-001 (Literal and BlankNode variant extensions to VarOrTerm/TermImpl and Encoder interning table, and fix all matching sites in queryengine, backwardchaining, csprite, query/reasoning/matching machinery)
- TICKET-002 (Production cycle guard for backwardchaining)
- TICKET-003 (Csprite cycle guards)

Scope Boundaries:
- Focus only on TICKET-001, TICKET-002, TICKET-003. Do not modify rule engine negation, N3 parsers, SPARQL compliance, shape validation, or CI files.
- Keep implementation simple and correct, matching the tickets' specifications.

Output Requirements:
- Write `progress.md` and `handoff.md` under your directory.
- Report back when all tests pass.
