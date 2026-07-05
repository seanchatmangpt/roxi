# N3 Lists

## Ordered Sequences

In N3, ordered lists are represented using parentheses:

```turtle
:john :hobbies (:reading :cycling :swimming) .
```

---

## AST Expansion

During parsing, Roxi expands lists into standard `rdf:first` and `rdf:rest` triples:

```turtle
:john :hobbies _:b1 .
_:b1 rdf:first :reading ; rdf:rest _:b2 .
_:b2 rdf:first :cycling ; rdf:rest _:b3 .
_:b3 rdf:first :swimming ; rdf:rest rdf:nil .
```

This expansion allows standard rule chainers and SPARQL queries to traverse list structures natively.
