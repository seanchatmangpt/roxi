# N3 Conformance Tests (EYE Reasoner Test Suite)

This directory is designed to contain the N3 and EYE reasoner conformance test cases.
Due to the lack of an official W3C Recommendation test suite for N3, we vendor community-standard test cases derived from the official EYE test corpus.

## Folder Structure

```text
lib/tests/n3_conformance/
├── README.md               # Documentation of the suite structure
├── manifest.json           # JSON index of all conformance test cases
├── syntax/                 # N3 syntax and structure test cases (planned)
│   ├── multi_head.n3
│   ├── literals.n3
│   └── ...
└── builtins/               # Built-ins (log:, math:, list:, string:) test cases (planned)
    ├── log_equal_to.n3
    ├── math_sum.n3
    └── ...
```

## Running Conformance Tests

A test runner harness (planned as part of TICKET-005 / TICKET-010) will:
1. Parse `manifest.json`.
2. For each test case with status `active`:
   - Load the `input` N3 file.
   - Parse rules and facts.
   - Run forward-chaining reasoning to compute the deductive closure.
   - Match the result against the expected N3 output defined in `expected` file.
   - Report success/failure.
