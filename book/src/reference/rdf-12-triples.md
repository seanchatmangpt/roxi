# RDF 1.2 Triple Terms

## Nested Assertions & Metadata

RDF 1.2 introduces **triple terms**, which provide a standard syntax for annotating other triples. This is often used for expressing confidence, origin, and temporal validity of statements.

In Roxi, triple terms are represented recursively within the `Term` enum:

```rust
pub enum Term {
    Iri(TermImpl),
    Literal(LiteralImpl),
    BlankNode(BlankNodeImpl),
    Triple(Box<Triple>), // Nested triple term
}
```

---

## Parsing Syntax

RDF 1.2 triple terms use double-angled braces:

```turtle
@prefix : <http://example.org/> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# A nested triple annotated with metadata
<< :alice :hasAccessTo :secureFolder >> :authorizedBy :admin ;
                                       :grantedDate "2026-07-04"^^xsd:date .
```

---

## Query Evaluation

In SPARQL 1.2, you can query these nested structures directly using matching patterns:

```sparql
PREFIX : <http://example.org/>
SELECT ?user ?grantor WHERE {
    << ?user :hasAccessTo :secureFolder >> :authorizedBy ?grantor .
}
```

Roxi's SPARQL query planner compiles this pattern into a nested `PlanNode`. The engine performs index lookups recursively, extracting the inner subject/predicate/object terms and matching them against the `TripleIndex`.
