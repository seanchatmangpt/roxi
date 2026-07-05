# Basic Usage

## Getting Hands-On with Roxi

This guide provides a complete, runnable example showing how to initialize a `TripleStore`, parse RDF Turtle triples, register Notation3 rules, run the reasoning engine, and inspect the results.

## Writing the Code

Open your `src/main.rs` file and replace its contents with the following code. This example demonstrates transitive friend relations, which are computed dynamically by the forward chainer:

```rust
use roxi::store::TripleStore;
use roxi::parser::Syntax;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 1. Initialize the embedded memory-backed TripleStore.
    // This coordinates the TripleIndex, the interning Encoder, and rule-evaluation.
    let mut store = TripleStore::new();

    // 2. Define the input data using RDF Turtle syntax.
    let data = r#"
        @prefix : <http://example.org/> .
        :alice :friendOf :bob .
        :bob :friendOf :charlie .
        :charlie :friendOf :david .
    "#;

    // Load the triples into the store. Under the hood, this parses the string,
    // interns the IRIs using the Encoder, and populates the SPO indexes.
    store.load_triples(data, Syntax::Turtle)?;
    println!("Successfully loaded baseline RDF data.");

    // 3. Define a Notation3 rule for transitive friend relations.
    // The rule head is implication-driven.
    let rules = r#"
        @prefix : <http://example.org/> .
        { ?x :friendOf ?y . ?y :friendOf ?z } => { ?x :transitiveFriend ?z } .
    "#;

    // Load and compile the rules into the rule index.
    store.load_rules(rules)?;
    println!("Successfully compiled N3 rules.");

    // 4. Run the forward-chaining reasoning engine.
    // This executes a fixpoint loop until no more new facts can be derived.
    store.materialize()?;
    println!("Reasoning materialization complete.");

    // 5. Query the store to verify the inferred transitive relations.
    // We expect alice to be a transitiveFriend of charlie, and bob to be a transitiveFriend of david.
    let alice = "<http://example.org/alice>";
    let friend_relation = "<http://example.org/transitiveFriend>";
    let charlie = "<http://example.org/charlie>";
    let david = "<http://example.org/david>";

    assert!(store.contains_triple(alice, friend_relation, charlie));
    assert!(store.contains_triple(alice, friend_relation, david));
    
    println!("Verification passed! Transitive relations successfully verified in index.");
    Ok(())
}
```

---

## Running the Code

To run the application, execute:

```bash
cargo run
```

You should see the progress output and the final verification message, confirming the triples were successfully indexed, reasoned, and checked.
