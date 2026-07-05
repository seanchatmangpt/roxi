# oxrdf Zero-Copy Adapter in Roxi

## 1. The Architectural Mapping Problem

Roxi and the Rust shape validation ecosystem (`rudof`/`shex-rs`) use distinct memory architectures to represent RDF data:

* **Roxi Architecture**: Interns all resource strings (IRIs, literals, datatype strings, and language tags) into a global table managed by the `Encoder`. It represents nodes in memory as simple machine-word-sized integer indices (`usize`). This minimizes allocation overhead, speeds up comparisons, and keeps the query engine fast.
* **oxrdf Architecture**: Represents nodes using strongly typed structures (`NamedNode`, `Literal`, `BlankNode`, `Term`) that own their underlying `String` values on the heap. External validators require data in these structures to execute schema validations correctly.

Converting the entire in-memory database to owned `oxrdf` types for every validation run is extremely expensive, introducing high heap-allocation costs and memory overhead.

Roxi resolves this using the **`oxrdf` Zero-Copy Adapter** located in [lib/src/oxrdf_adapter.rs](file:///Users/sac/roxi/lib/src/oxrdf_adapter.rs).

---

## 2. Zero-Copy Adapter Design

The adapter provides a bridge between the two architectures, implementing lazy, on-demand conversion of nodes only when requested by validators:

```
        Roxi Interned Memory                     oxrdf Owned Structures
    +--------------------------+               +--------------------------+
    |   TermImpl (iri: 42)     |  =========>   | NamedNode("http://...")  |
    |   LiteralImpl (val: 101) |  (On-Demand)  | Literal("25", xsd:int)   |
    |   BlankNodeImpl (id: 7)  |  =========>   | BlankNode("b7")          |
    +--------------------------+               +--------------------------+
```

### Forward Translation (Roxi $\to$ oxrdf)
* **IRIs**: Decodes the interned `usize` from the `Encoder` back to its string value and constructs an `oxrdf::NamedNodeRef` (borrowed) or `oxrdf::NamedNode` (owned).
* **Literals**: Extracts the lexical value index, datatype index, and language tag index. Decodes them and constructs an `oxrdf::Literal` with correct datatype and language properties.
* **Blank Nodes**: Converts internal `usize` identifiers into unique string labels (e.g., `b0`, `b1`, etc.) so `oxrdf` can identify nodes accurately.

### Reverse Translation (oxrdf $\to$ Roxi)
When a shape validator identifies a schema violation (e.g. "Value in `ex:age` property must be an integer"), it returns a validation report containing the violating `oxrdf::Term`. 

The adapter translates the `oxrdf::Term` back to Roxi's internal `usize` representation by querying the `Encoder` (or inserting it if new). This enables Roxi to locate the exact violating triple in the `TripleIndex` and report it to the user with its original file coordinates.

---

## 3. Rust Implementation Reference

Below is the Rust structural design of the adapter:

```rust
use oxrdf::{NamedNode, Literal, BlankNode, Term as OxTerm};
use std::rc::Rc;

// Represents Roxi's internal term representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RoxiTerm {
    Iri(usize),
    Literal {
        value: usize,
        datatype: Option<usize>,
        lang: Option<usize>,
    },
    BlankNode(usize),
}

pub struct Encoder {
    // String interning mappings
    id_to_string: Vec<String>,
    string_to_id: std::collections::HashMap<String, usize>,
}

impl Encoder {
    pub fn lookup(&self, id: usize) -> &str {
        &self.id_to_string[id]
    }

    pub fn encode(&mut self, val: &str) -> usize {
        if let Some(&id) = self.string_to_id.get(val) {
            id
        } else {
            let id = self.id_to_string.len();
            self.id_to_string.push(val.to_string());
            self.string_to_id.insert(val.to_string(), id);
            id
        }
    }
}

pub struct OxRdfAdapter;

impl OxRdfAdapter {
    /// Translates a Roxi internal term to an oxrdf Term
    pub fn to_oxrdf(term: &RoxiTerm, encoder: &Encoder) -> OxTerm {
        match term {
            RoxiTerm::Iri(id) => {
                let iri_str = encoder.lookup(*id);
                OxTerm::NamedNode(NamedNode::new(iri_str).unwrap())
            }
            RoxiTerm::Literal { value, datatype, lang } => {
                let lex = encoder.lookup(*value);
                let literal = if let Some(lang_id) = lang {
                    let lang_str = encoder.lookup(*lang_id);
                    Literal::new_language_tagged_literal(lex, lang_str).unwrap()
                } else if let Some(dt_id) = datatype {
                    let dt_str = encoder.lookup(*dt_id);
                    let dt_node = NamedNode::new(dt_str).unwrap();
                    Literal::new_typed_literal(lex, dt_node)
                } else {
                    Literal::new_simple_literal(lex)
                };
                OxTerm::Literal(literal)
            }
            RoxiTerm::BlankNode(id) => {
                let bnode_str = format!("b{}", id);
                OxTerm::BlankNode(BlankNode::new(bnode_str).unwrap())
            }
        }
    }

    /// Translates an oxrdf Term back to Roxi's internal representation
    pub fn from_oxrdf(term: &OxTerm, encoder: &mut Encoder) -> RoxiTerm {
        match term {
            OxTerm::NamedNode(node) => {
                let id = encoder.encode(node.as_str());
                RoxiTerm::Iri(id)
            }
            OxTerm::Literal(lit) => {
                let val_id = encoder.encode(lit.value());
                let dt_id = encoder.encode(lit.datatype().as_str());
                let lang_id = lit.language().map(|lang| encoder.encode(lang));
                RoxiTerm::Literal {
                    value: val_id,
                    datatype: Some(dt_id),
                    lang: lang_id,
                }
            }
            OxTerm::BlankNode(bnode) => {
                // Parse out the integer from the blank node string (e.g. "b7" -> 7)
                let id_str = &bnode.as_str()[1..];
                let id = id_str.parse::<usize>().unwrap_or(0);
                RoxiTerm::BlankNode(id)
            }
            _ => unimplemented!("Nested triple term translation"),
        }
    }
}
```
