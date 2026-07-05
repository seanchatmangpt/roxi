# Literals & Datatypes

## Anatomy of an RDF Literal

Unlike IRIs, RDF literals have internal structures:
1. **Lexical Value**: The actual text value (e.g. `"42"` or `"hello"`).
2. **Datatype IRI**: The type of value (e.g. `xsd:integer` or `xsd:date`).
3. **Language Tag**: Optional string indicating the language (e.g. `"en"` or `"fr"`).

Roxi represents these values using the `LiteralImpl` struct:

```rust
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct LiteralImpl {
    pub value: usize,            // Encoded lexical value
    pub datatype: Option<usize>, // Encoded datatype IRI
    pub lang: Option<usize>,     // Encoded language tag
}
```

---

## Datatype and Language Tag Preservation

To maintain semantic compliance, the interning `Encoder` preserves datatype and language tag contexts during roundtrips:

* **Separation**: The encoder maintains distinct namespaces for values, datatypes, and language tags.
* **Uniqueness**: The literals `"42"^^xsd:integer` and `"42"^^xsd:string` resolve to distinct `LiteralImpl` instances, preventing incorrect matches during query evaluations.
* **SPARQL Filters**: During SPARQL filter execution (e.g., `?age > 21`), Roxi's query evaluator extracts the datatype IRI, decodes the lexical value, parses it to a native Rust numeric type (`i32` or `f64`), and performs the comparison. If the datatype is unrecognized, it falls back to lexical comparison.
