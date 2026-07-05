# SHACL Conformance Pass-Rate Manifest

- **Dialect**: SHACL
- **Suite**: W3C data-shapes Test Suite (vendored core-constraint-component subset: node/, property/, targets/, misc/)
- **Source**: https://github.com/w3c/data-shapes/tree/gh-pages/data-shapes-test-suite/tests/core
- **Coverage note**: `core/path/` (complex SPARQL-property-path `sh:path` expressions -- sequence/inverse/alternative/zeroOrMore/etc.) is NOT vendored: `shacl.rs` only supports simple single-predicate `sh:path`, so those ~17 real cases are an honest, documented gap, not silently dropped.
- **Total Tests**: 60
- **Passed**: 60
- **Failed**: 0
- **Pass Rate**: 100.0%

## Test Details

| Test Name | Status | Details |
|---|---|---|
| node/and-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| node/and-002 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| node/class-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| node/class-002 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| node/class-003 | PASSED | expected conforms=false violations=5, got conforms=false violations=5 |
| node/closed-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| node/closed-002 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/datatype-001 | PASSED | expected conforms=false violations=3, got conforms=false violations=3 |
| node/datatype-002 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| node/disjoint-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/equals-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| node/hasValue-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/in-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/languageIn-001 | PASSED | expected conforms=false violations=3, got conforms=false violations=3 |
| node/maxLength-001 | PASSED | expected conforms=false violations=5, got conforms=false violations=5 |
| node/minExclusive-001 | PASSED | expected conforms=false violations=6, got conforms=false violations=6 |
| node/minInclusive-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/minInclusive-002 | PASSED | expected conforms=false violations=3, got conforms=false violations=3 |
| node/minInclusive-003 | PASSED | expected conforms=false violations=4, got conforms=false violations=4 |
| node/minLength-001 | PASSED | expected conforms=false violations=4, got conforms=false violations=4 |
| node/node-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/nodeKind-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/not-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/not-002 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/or-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| node/pattern-001 | PASSED | expected conforms=false violations=4, got conforms=false violations=4 |
| node/pattern-002 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/xone-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| node/qualified-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/datatype-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| property/datatype-002 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| property/datatype-003 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/disjoint-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| property/equals-001 | PASSED | expected conforms=false violations=5, got conforms=false violations=5 |
| property/hasValue-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/in-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/minCount-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/minCount-002 | PASSED | expected conforms=true violations=0, got conforms=true violations=0 |
| property/maxCount-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/maxCount-002 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/maxExclusive-001 | PASSED | expected conforms=false violations=3, got conforms=false violations=3 |
| property/maxInclusive-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| property/minExclusive-002 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| property/minLength-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/node-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/not-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/or-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| property/pattern-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| property/property-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| property/lessThan-001 | PASSED | expected conforms=false violations=3, got conforms=false violations=3 |
| property/lessThan-002 | PASSED | expected conforms=false violations=4, got conforms=false violations=4 |
| property/lessThanOrEquals-001 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
| property/uniqueLang-001 | PASSED | expected conforms=false violations=3, got conforms=false violations=3 |
| targets/targetClass-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| targets/targetNode-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| misc/deactivated-001 | PASSED | expected conforms=true violations=0, got conforms=true violations=0 |
| misc/deactivated-002 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| misc/message-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| misc/severity-001 | PASSED | expected conforms=false violations=1, got conforms=false violations=1 |
| misc/severity-002 | PASSED | expected conforms=false violations=2, got conforms=false violations=2 |
