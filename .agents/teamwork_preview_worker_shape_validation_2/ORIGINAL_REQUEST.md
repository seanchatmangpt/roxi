## 2026-07-04T19:59:32Z
You are teamwork_preview_worker_shape_validation_2.
Your working directory is /Users/sac/roxi/.agents/teamwork_preview_worker_shape_validation_2.
Your mission is to fix the critical literal roundtrip bug in the TICKET-007 `oxrdf` adapter layer.

Instructions:
1. Read the reviewer handoff reports at:
   - /Users/sac/roxi/.agents/teamwork_preview_reviewer_shape_validation_1_1/handoff.md
   - /Users/sac/roxi/.agents/teamwork_preview_reviewer_shape_validation_1_2/handoff.md
2. Edit /Users/sac/roxi/lib/src/oxrdf_adapter.rs. Specifically in `oxrdf_term_to_roxi_term`, normalize converted literals by stripping implicit/default datatypes (mapping `http://www.w3.org/2001/XMLSchema#string` and `http://www.w3.org/1999/02/22-rdf-syntax-ns#langString` datatypes to `None` in the Roxi term representation).
3. Update /Users/sac/roxi/lib/tests/oxrdf_adapter.rs to add the roundtrip tests: `test_simple_literal_roundtrip_equality` and `test_literal_lang_roundtrip_equality`.
4. Verify by running `cargo test -p roxi --test oxrdf_adapter`. Ensure all tests in the adapter suite compile and pass.
5. Create a detailed handoff.md under /Users/sac/roxi/.agents/teamwork_preview_worker_shape_validation_2/ showing the test results.

MANDATORY INTEGRITY WARNING:
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.

Report back with a message when done.
