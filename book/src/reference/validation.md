# Shape Validation

## Graph Schema Enforcement

Reasoning engines generate new facts, but without schemas, data quality can degrade. Roxi includes first-class shape validation using both **SHACL** and **ShEx**.

---

## The Unified Substrate

Instead of running a separate validator process, Roxi runs shape checks directly on the `TripleIndex` using an optimized adapter layer:

```
[Input Data] ---> [TripleIndex] <---> [oxrdf Adapter] <---> [rudof Validators]
```

This architecture allows Roxi to interleave shape validation with forward reasoning, terminating executions early if data violations are detected during derivation.
