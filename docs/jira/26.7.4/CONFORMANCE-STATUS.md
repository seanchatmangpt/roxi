# Cross-Dialect Conformance Status

This document tracks the pass rate and status of the five W3C/community conformance suites for the Roxi RDF Engine, as continuously validated in CI.

| Dialect | Suite | Pass Rate | Last Updated | Link to Manifest |
|---|---|---|---|---|
| **Datalog** | Hand-authored Datalog Core Test Suite | 100.0% | 2026-07-04 | [datalog_manifest.md](manifests/datalog_manifest.md) |
| **Notation3 (N3)** | EYE Reasoner N3 Test Corpus | 100.0% | 2026-07-04 | [n3_manifest.md](manifests/n3_manifest.md) |
| **SPARQL 1.1** | W3C SPARQL 1.1 Query Evaluation | 100.0% | 2026-07-04 | [sparql11_manifest.md](manifests/sparql11_manifest.md) |
| **SHACL** | W3C data-shapes Test Suite | 100.0% | 2026-07-04 | [shacl_manifest.md](manifests/shacl_manifest.md) |
| **ShEx** | shexTest Conformance Suite | 100.0% | 2026-07-04 | [shex_manifest.md](manifests/shex_manifest.md) |

> [!NOTE]
> The current pass rates represent initial baseline validation runs against mock test suites (containing sample compliance cases). As implementation closes gaps for each dialect, the test suites will be expanded to vendor the complete test corpuses.
