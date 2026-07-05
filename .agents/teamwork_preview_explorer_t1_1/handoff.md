# Handoff Report: TICKET-001 — Extend `VarOrTerm`/`TermImpl` with Literal and BlankNode variants

## 1. Observation

In the read-only investigation of the `roxi` codebase, the following files and definitions were examined:

### A. Term Model & Variants
* **`lib/src/triples.rs` (lines 3-9):**
  ```rust
  #[derive(Debug, Clone, Eq, PartialEq, Hash)]
  pub enum VarOrTerm {
      Var(Variable),
      Term(TermImpl),
      // Literal(Literal),
      // BlankNode(BlankNode)
  }
  ```
  Only `TermImpl` (which is an IRI) and `Variable` are supported.
* **`lib/src/triples.rs` (lines 79-81):**
  ```rust
  #[derive(Debug, Clone, Eq, PartialEq, Hash)]
  pub struct TermImpl {
      pub(crate) iri: usize,
  }
  ```
* **`lib/src/triples.rs` (lines 48-53):**
  ```rust
  pub fn to_encoded(&self) -> usize {
      match self {
          Self::Var(var) => var.name,
          Self::Term(term) => term.iri,
      }
  }
  ```

### B. Encoder Table
* **`lib/src/encoding.rs` (lines 8-13):**
  ```rust
  #[derive(Debug,  Clone, Eq, PartialEq)]
  pub struct InternalEncoder{
      encoded: HashMap<String, usize>,
      decoded: HashMap<usize,String>,
      counter: usize
  }
  ```
  The table does not differentiate kinds (IRI vs literal vs blank node), mapping string entries directly to integers.
* **`lib/src/encoding.rs` (lines 47-59):**
  ```rust
  pub fn add(uri:String) -> usize{
      let mut encoder = GLOBAL_ENCODER.lock().unwrap();
      if let Some(encoded_uri) = encoder.encoded.get(&uri){
          return *encoded_uri;
      }else{
          let current_counter = encoder.counter;
          encoder.encoded.insert(uri.clone(),current_counter);
          encoder.decoded.insert(current_counter,uri);
          encoder.counter+=1;
          encoder.counter -1
      }
  }
  ```

### C. Match Sites and Index Tables
* **`lib/src/tripleindex.rs` (lines 8-14):**
  ```rust
  pub struct TripleIndex{
      pub triples: Vec<Triple>,
      spo:HashMap<usize,  HashMap<usize,Vec<(usize,usize, Option<TermImpl>)>>>,
      pos:HashMap<usize,  HashMap<usize,Vec<(usize,usize, Option<TermImpl>)>>>,
      osp:HashMap<usize,  HashMap<usize,Vec<(usize,usize, Option<TermImpl>)>>>,
      counter:usize,
  }
  ```
  Graph names are indexed as `Option<TermImpl>`, and SPO/POS/OSP maps use `usize` keys retrieved via `to_encoded()`.
* **`lib/src/reasoner.rs` (lines 64-77):**
  ```rust
  for result_counter in 0..binding.len(){
      match &head.s{
          VarOrTerm::Var(s_var)=> s = binding.get(&s_var.name).unwrap().get(result_counter).unwrap(),
          VarOrTerm::Term(s_term)=> s = &s_term.iri
      }
      // ...
      new_heads.push(Triple{s:VarOrTerm::Term(TermImpl{iri:s.clone()}), ...})
  }
  ```
* **`lib/src/dred.rs` (lines 116-123):**
  ```rust
  fn eval_triple_element(left: &VarOrTerm, right: &VarOrTerm, bindings: &mut Binding) -> bool {
      if let (VarOrTerm::Var(left_name), VarOrTerm::Term(right_name)) = (left, right) {
          bindings.add(&left_name.name, right_name.iri);
          true
      } else {
          left.eq(right)
      }
  }
  ```
* **`lib/src/sparql.rs` (lines 33-35 & 186):**
  ```rust
  pub enum PlanExpression{
      Constant(TermImpl),
      // ...
  }
  // ...
  Expression::Literal(value)=>PlanExpression::Constant(TermImpl{iri:value.value().parse::<usize>().unwrap()}),
  ```

---

## 2. Logic Chain

1. **Indexed representation requirement**: `TripleIndex` uses `usize` keys for subjects, predicates, objects, and stores graph names as `Option<TermImpl>` (or `Option<Term>`). To avoid re-engineering the index structure (which would have high complexity and run-time overhead), `VarOrTerm::to_encoded` must continue to return a unique `usize` ID for every distinct term variant (IRIs, Literals, Blank Nodes, and Variables).
2. **Term-to-integer mapping**: A unique `usize` ID per distinct term (including literals with value/datatype/lang combinations) can be obtained by interning the full `Term` structures or their Canonical String Representations in the global `Encoder`.
3. **Round-tripping tagged entries**: By mapping each `usize` ID to an `EncodedValue` enum (with `Iri`, `LiteralLexical`, `BlankNodeLabel`, and composite `Literal` variants), the `Encoder` can round-trip the specific kind.
4. **Simplification of pattern matching**: If `Term` implements `PartialEq`, `VarOrTerm` matches can compare `Term` instances directly (via `==`), bypassing the manual field-unpacking.

---

## 3. Caveats

* **Doctest Failures**: Doctests (e.g. `lib/src/imars_window.rs`) fail during standard compilation due to missing workspace dependencies configured inside doctest blocks. This is a pre-existing workspace issue and does not relate to our changes. The unit tests compile and run successfully via `cargo test --lib --bins`.
* **Sparql Numeric Shortcut**: In `sparql.rs:186`, integer literals were parsed as `usize` and directly assigned to `TermImpl::iri` without interning. This needs to be correctly translated into interning a `Term::Literal` containing the integer lexical string and `xsd:integer` datatype IRI, then parsing this value inside `eval_expression`.

---

## 4. Conclusion

Implementing TICKET-001 requires:
1. Defining the structured `Term` enum (variants: `Iri`, `Literal`, `BlankNode`) and backing structs (`TermImpl`, `LiteralImpl`, `BlankNodeImpl`).
2. Extending `Encoder` to store and retrieve `EncodedValue` enums in both directions.
3. Updating all matches on `VarOrTerm` and field accesses to `.iri` (e.g. changing them to `.id()`) to align with the new model.

---

## 5. Implementation Strategy

### A. Precise Structures for `Term`, `LiteralImpl`, and `BlankNodeImpl`

We will add these to `lib/src/triples.rs`:

```rust
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Term {
    Iri(TermImpl),
    Literal(LiteralImpl),
    BlankNode(BlankNodeImpl),
}

impl Term {
    pub fn id(&self) -> usize {
        match self {
            Self::Iri(iri) => iri.iri,
            Self::Literal(lit) => lit.id,
            Self::BlankNode(bnode) => bnode.id,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TermImpl {
    pub(crate) iri: usize, // References an Iri entry in the encoder
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LiteralImpl {
    pub(crate) id: usize,                  // References a Literal entry in the encoder
    pub(crate) value: usize,               // References a LiteralLexical entry
    pub(crate) datatype: Option<usize>,    // References an Iri entry
    pub(crate) lang: Option<usize>,        // References a LiteralLexical entry
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BlankNodeImpl {
    pub(crate) id: usize,                  // References a BlankNodeLabel entry in the encoder
}
```

And update `VarOrTerm` to:
```rust
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum VarOrTerm {
    Var(Variable),
    Term(Term),
}
```

---

### B. Encoder Extension Strategy

We will update `lib/src/encoding.rs` to support `EncodedValue` enums:

```rust
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum EncodedValue {
    Iri(String),
    LiteralLexical(String),
    BlankNodeLabel(String),
    Literal {
        value: usize,            // Index of the lexical value string
        datatype: Option<usize>, // Index of the datatype IRI
        lang: Option<usize>,     // Index of the language tag
    },
    Variable(String),
}
```

The `InternalEncoder` mapping:
```rust
pub struct InternalEncoder {
    encoded: HashMap<EncodedValue, usize>,
    decoded: HashMap<usize, EncodedValue>,
    counter: usize,
}
```

The `Encoder` will expose:
1. `pub fn add_iri(iri: String) -> usize`
2. `pub fn add_blank_node(label: String) -> usize`
3. `pub fn add_literal(value: String, datatype: Option<String>, lang: Option<String>) -> usize`
4. `pub fn decode_to_term(id: usize) -> Option<Term>`:
   ```rust
   pub fn decode_to_term(id: usize) -> Option<Term> {
       let encoder = GLOBAL_ENCODER.lock().unwrap();
       match encoder.decoded.get(&id)? {
           EncodedValue::Iri(_) => Some(Term::Iri(TermImpl { iri: id })),
           EncodedValue::BlankNodeLabel(_) => Some(Term::BlankNode(BlankNodeImpl { id })),
           EncodedValue::Literal { value, datatype, lang } => Some(Term::Literal(LiteralImpl {
               id,
               value: *value,
               datatype: *datatype,
               lang: *lang,
           })),
           _ => None,
       }
   }
   ```
5. `pub fn decode(id: &usize) -> Option<String>`: Recreates formatting for round-trips:
   - `Iri(s)` -> `s`
   - `LiteralLexical(s)` -> `s`
   - `BlankNodeLabel(label)` -> `format!("_:{}", label)`
   - `Literal { value, datatype, lang }` -> Recursively decode and return `'"value"@lang'` or `'"value"^^datatype'` or `'"value"'`.

---

### C. File & Code Block Update List

1. **`lib/src/triples.rs`**:
   * Replace `VarOrTerm`, `TermImpl`, `to_encoded()`, `new_term()`, `new_encoded_term()`, `as_term()`, and `convert()`.
2. **`lib/src/encoding.rs`**:
   * Replace the string mappings with `EncodedValue` maps. Add parser and kind helper functions.
3. **`lib/src/tripleindex.rs`**:
   * Replace `Option<TermImpl>` with `Option<Term>` in `spo`, `pos`, and `osp`.
   * Update references of `.iri` on graph names (lines 305, 331, 356, 383, 433, 446) to use `.id()`.
4. **`lib/src/reasoner.rs`**:
   * Line 66, 70, 74: change `s_term.iri` to `s_term.id()`.
   * Line 76: replace `TermImpl { iri: s.clone() }` with `Encoder::decode_to_term(s.clone()).unwrap()`.
   * Line 98, 102, 106: change `VarOrTerm::new_encoded_term(s_term.iri.clone())` to `VarOrTerm::Term(s_term.clone())`.
   * Line 222, 227, 232: change `VarOrTerm::new_encoded_term(triple.s.as_term().iri)` to `triple.s.clone()`.
   * Line 266, 272, 278: update `s.as_term().iri` to `s.as_term().id()`.
   * Line 267, 273, 279: update `VarOrTerm::Term(s_term) => if let (TermImpl{iri}, TermImpl{iri:iri2}) = ...` to `VarOrTerm::Term(s_term) => if s_term != s.as_term() { return None; }`.
5. **`lib/src/dred.rs`**:
   * Line 118: change `bindings.add(&left_name.name, right_name.iri)` to `bindings.add(&left_name.name, right_name.id())`.
6. **`lib/src/sparql.rs`**:
   * Line 34: Change `PlanExpression::Constant(TermImpl)` to `PlanExpression::Constant(Term)`.
   * Line 186: Update SPARQL Literal expression parsing to intern a proper `Term::Literal` containing the datatype and string value, then wrap it in `PlanExpression::Constant`.
   * Line 533: Update constant evaluation in `eval_expression` to decode the term value and parse it as `usize`.

---

## 6. Verification Method

Independent verification must confirm that:
1. `cargo test --lib --bins` executes and matches the baseline tests with zero regressions.
2. The newly added test targets round-trip literals and blank nodes correctly.

### Commands to Run:
```bash
cargo test --lib --bins
```

---

## 7. Remaining Work

1. Implement `Term` and related variant structs in `lib/src/triples.rs`.
2. Restructure `Encoder` maps and operations in `lib/src/encoding.rs`.
3. Update match and access sites in `tripleindex.rs`, `reasoner.rs`, `dred.rs`, and `sparql.rs`.
4. Add verification tests:
   - `test_literal_term_roundtrip`
   - `test_blank_node_term_encoding`
   - `test_encoder_literal_vs_iri_distinct`
   - `test_literal_datatype_and_langtag_preserved`
