# Term Model & RDF 1.2

## Unified RDF Representation

The foundation of Roxi is its unified term model, defined in [lib/src/triples.rs](file:///Users/sac/roxi/lib/src/triples.rs). To understand why this model is designed the way it is, we must look at the constraints of semantic graph databases.

In a typical graph, the same term (for example, the IRI `<http://example.org/friendOf>`) can appear millions of times as a predicate, subject, or object. Storing this IRI as a heap-allocated string inside every triple structure leads to massive memory overhead and slow comparison speeds (since comparing two triples would require string comparison).

Roxi solves this by representing terms using the `Term` sibling enum:

```rust
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Term {
    Iri(TermImpl),
    Literal(LiteralImpl),
    BlankNode(BlankNodeImpl),
    Triple(Box<Triple>), // Nested triple terms for RDF 1.2
}
```

---

## Sibling Enum Architecture vs. Side Tables

During the design phase of the v26.7.4 release, two options were considered for representing literals and blank nodes:
1. **Side-Table Mapping**: Keep terms as simple `usize` integers, and store literal/blank-node properties in a separate global mapping table.
2. **Sibling Enum (Chosen)**: Expand `Term` to be a type-safe enum containing distinct `TermImpl`, `LiteralImpl`, and `BlankNodeImpl` variants.

The sibling enum architecture was chosen for several critical reasons:
* **Exhaustive Matching**: The Rust compiler enforces that every match block on `Term` (inside the query parser, rule evaluator, and index builders) handles all variants. This eliminates runtime mismatch bugs where a literal is mistakenly treated as an IRI.
* **Alignment with oxrdf**: This enum mirrors `oxrdf::Term` (`NamedNode`, `Literal`, `BlankNode`), making the adapter layer zero-copy and highly optimized.
* **No Synchronization Overhead**: Side tables require locks (`Mutex`/`RwLock`) or thread-local contexts for reads, whereas the sibling enum carries its type information in-place, allowing lock-free parallel query matches.
