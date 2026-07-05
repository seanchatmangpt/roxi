# BRIEFING — 2026-07-04T19:34:13-07:00

## Mission
Coordinate implementation and validation of 10 dependency-ordered JIRA tickets for roxi RDF engine enhancements.

## 🔒 My Identity
- Archetype: project_orchestrator
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /Users/sac/roxi/.agents/orchestrator
- Original parent: parent
- Original parent conversation ID: f48b6e1b-90aa-40b7-a6e2-350aa1e6ee76

## 🔒 My Workflow
- **Pattern**: Project Pattern
- **Scope document**: /Users/sac/roxi/PROJECT.md
1. **Decompose**: Decomposed into 5 milestones (M1 to M5) wrapping the 10 tickets.
2. **Dispatch & Execute**:
   - **Delegate (sub-orchestrator)**: Spawn a sub-orchestrator for each milestone.
3. **On failure**:
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (last resort)
4. **Succession**: Self-succeed at 16 spawns, write handoff.md, spawn successor.
- **Work items**:
  - Milestone 1: Term Model and Cycle Safety [done]
  - Milestone 2: Datalog and N3 Dialects [in-progress]
  - Milestone 3: SPARQL 1.1 Conformance [done]
  - Milestone 4: Shape Validation (SHACL & ShEx) [done]
  - Milestone 5: Continuous Conformance CI Gate [pending]
- **Current phase**: 2
- **Current focus**: Milestone 1: Term Model and Cycle Safety

## 🔒 Key Constraints
- NEVER write, modify, or create source code files directly.
- NEVER run build/test commands yourself — require workers to do so.
- Never reuse a subagent after it has delivered its handoff — always spawn fresh.
- Zero-tolerance on integrity violations (veto by Forensic Auditor).

## Current Parent
- Conversation ID: f48b6e1b-90aa-40b7-a6e2-350aa1e6ee76
- Updated: not yet

## Key Decisions Made
- Decomposed the 10 tickets into 5 compound milestones: M1 (001, 002, 003), M2 (004, 005), M3 (006), M4 (007, 008, 009), M5 (010).

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
| sub_orch_m1 | self | Milestone 1 Sub-Orchestrator | completed | c9990f75-efd3-4e99-9736-87040d0dee92 |
| sub_orch_m2 | self | Milestone 2 Sub-Orchestrator | in-progress | 4cca4913-9706-48d5-8463-afcaa7adf466 |
| sub_orch_m3 | self | Milestone 3 Sub-Orchestrator | completed | 2fab8506-c9ca-46cd-95f3-bd12eb736b6e |
| sub_orch_m4 | self | Milestone 4 Sub-Orchestrator | completed | e614f6e1-e383-41a5-8b6d-d31da60f6e5c (successor: 52d705e9-dc60-4e67-a0af-bb5ddd84d90a) |

## Succession Status
- Succession required: no
- Spawn count: 4 / 16
- Pending subagents: [4cca4913-9706-48d5-8463-afcaa7adf466]
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: task-63
- Safety timer: none
- On succession: kill all timers before spawning successor
- On context truncation: run `manage_task(Action="list")` — re-create if missing

## Artifact Index
- `/Users/sac/roxi/PROJECT.md` — Global project definition, milestones, and contracts.
- `/Users/sac/roxi/.agents/orchestrator/plan.md` — Execution plan and dependencies.
- `/Users/sac/roxi/.agents/orchestrator/progress.md` — Progress checklist and liveness heartbeat.
- `/Users/sac/roxi/.agents/orchestrator/context.md` — Active context variables.
