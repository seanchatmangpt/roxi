# Installation

## Dependency Configuration

Roxi is available as a Rust crate. To add it to your project, specify it in your `Cargo.toml`. To leverage the complete feature set of v26.7.4 (such as the Datalog stratified engine), you should enable the relevant feature flags.

```toml
[package]
name = "roxi_quickstart"
version = "0.1.0"
edition = "2021"

[dependencies]
# Add the roxi library with desired feature flags
roxi = { version = "0.1.0", features = ["datalog"] }

# For RDF 1.2 and SPARQL 1.2 compatibility, add the corresponding crates:
oxrdf = { version = "0.3.3", features = ["rdf-12"] }
spargebra = { version = "0.4.6", features = ["sparql-12"] }
sparesults = { version = "0.3.3", features = ["sparql-12"] }
```

---

## Feature Flags Reference

Roxi uses Cargo feature flags to compile only the code your application needs. This keeps WebAssembly sizes tiny and build times short:

1. **`default`**: Includes the core RDF storage, the SPARQL 1.1 query engine, and the basic N3 reasoner (Horn rule evaluation).
2. **`datalog`**: Unlocks the advanced Datalog stratified reasoner, topological sorting of predicates, rule-safety checkers, and accumulators for head aggregations.
3. **`wasm`**: Optimizes the build for target `wasm32-unknown-unknown` (minimizes dependencies, enables JS-compatible clock utilities, and disables multi-threaded rayon indexing).

---

## Compiling for WebAssembly

If you are building for the browser using `wasm-pack`, you can compile the crate directly:

```bash
wasm-pack build --target web
```

Roxi automatically conditionalizes thread pools and file I/O operations when compiled under the WebAssembly target, ensuring it runs seamlessly inside browser threads and Web Workers.
