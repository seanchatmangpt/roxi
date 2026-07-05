# Running Queries

## Declarative Data Fetching with SPARQL

While direct index queries (like `contains_triple`) are fast for checking specific facts, complex data extraction requires a declarative language. Roxi provides full support for the **SPARQL 1.1 Query** specification.

Queries are evaluated against both parsed facts and derived facts (if the reasoner was run beforehand).

## Evaluating Queries in Rust

Below is a complete example of running a SPARQL query that selects all individuals and their friends, returning the results as bindings.

```rust
use roxi::store::TripleStore;
use roxi::parser::Syntax;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut store = TripleStore::new();

    // Load data
    let data = r#"
        @prefix : <http://example.org/> .
        :alice :friendOf :bob ;
               :age 25 .
        :bob :friendOf :charlie ;
             :age 30 .
    "#;
    store.load_triples(data, Syntax::Turtle)?;

    // Define a SPARQL query to select names and ages
    let sparql_query = r#"
        PREFIX : <http://example.org/>
        SELECT ?person ?age ?friend WHERE {
            ?person :age ?age .
            OPTIONAL { ?person :friendOf ?friend }
        }
    "#;

    // Execute the query
    let results = store.execute_sparql(sparql_query)?;

    println!("Query execution results:");
    println!("----------------------------------");
    
    // Iterate over the resulting bindings
    for binding in results.bindings() {
        let person = binding.get("person").map(|t| t.to_string()).unwrap_or_default();
        let age = binding.get("age").map(|t| t.to_string()).unwrap_or_default();
        let friend = binding.get("friend").map(|t| t.to_string()).unwrap_or_else(|| "none".to_string());
        
        println!("Person: {}, Age: {}, Friend: {}", person, age, friend);
    }
    
    Ok(())
}
```

---

## How It Works Under the Hood

1. **Algebra Compilation**: Roxi delegates SPARQL syntax parsing to `spargebra`. It converts the query string into a SPARQL algebra object tree.
2. **Plan Generation**: The algebra tree is compiled into a `PlanNode` execution tree. Triple patterns inside the query are mapped directly to lazy iterator lookups on the `TripleIndex`.
3. **Execution**: The plan is evaluated against the store. Variables (like `?person` and `?age`) are bound to the interned `usize` keys of the matching terms, and resolved back to strings only when calling `binding.get()`.
