# N3 Procedural Built-ins Reference

## 1. Runtime Interception Mechanics

In standard RDF semantics, a triple like `?x math:sum (?y ?z)` is matched as a literal pattern search against the triple store. However, in Notation3 (N3), the `math:sum` predicate is a **built-in**. When the reasoner encounters a built-in predicate, it intercepts execution and executes procedural Rust code in the query loop instead of searching the database indexes.

```
                  Rule Body Matcher
                          |
            Check Predicate Namespace/IRI
               /                     \\
      [Standard IRI]             [Built-in IRI]
            /                           \\
           v                             v
   Scan TripleIndex              Execute Rust Function
   (Index Lookup)             (e.g., math_sum, string_concat)
                                         |
                                         v
                                Bind Output Variable
```

---

## 2. Complete Built-ins Directory

### 2.1. The `log:` Logical Namespace
The `log:` namespace provides operations for formula checking, term equality, and database queries.

#### `log:equalTo`
* **Syntax**: `?termA log:equalTo ?termB`
* **Input**: Both terms must be bound.
* **Behavior**: Evaluates to true if the interned ID of `?termA` is identical to `?termB`.
* **Example**:
  ```turtle
  { ?x :id ?id . ?id log:equalTo "admin" } => { ?x :role :sysAdmin } .
  ```

#### `log:implies`
* **Syntax**: `{body} log:implies {head}`
* **Behavior**: Equivalent to `=>`. Denotes forward logical implication.

---

### 2.2. The `math:` Mathematical Namespace
The `math:` namespace handles basic arithmetic and numeric comparison operations.

#### `math:sum`
* **Syntax**: `?list math:sum ?result`
* **Input**: `?list` must be bound to an RDF list of numeric terms. `?result` can be bound or unbound.
* **Behavior**: Sums the values of the list elements. If `?result` is unbound, binds it to the sum; if bound, verifies matching equality.
* **Example**:
  ```turtle
  { (?val1 ?val2) math:sum ?out } => { :total :value ?out } .
  ```

#### `math:greaterThan`
* **Syntax**: `?valA math:greaterThan ?valB`
* **Input**: Both values must be bound to numeric terms.
* **Behavior**: Succeeds if `?valA` is strictly greater than `?valB`.
* **Example**:
  ```turtle
  { ?emp :salary ?sal . ?sal math:greaterThan 120000 } => { ?emp :bracket :top } .
  ```

---

### 2.3. The `list:` List-Processing Namespace
The `list:` namespace contains operations for manipulating and querying ordered lists.

#### `list:in`
* **Syntax**: `?member list:in ?list`
* **Input**: `?list` must be bound to an RDF list.
* **Behavior**: Iterates over elements of `?list`. If `?member` is unbound, binds it to each element sequentially (backtracking); if bound, verifies membership.
* **Example**:
  ```turtle
  { ?color list:in (:red :green :blue) } => { ?color :status :primary } .
  ```

#### `list:length`
* **Syntax**: `?list list:length ?len`
* **Input**: `?list` must be bound to an RDF list.
* **Behavior**: Counts elements in `?list` and binds or verifies the integer result in `?len`.
* **Example**:
  ```turtle
  { ?user :hobbies ?list . ?list list:length ?len } => { ?user :hobbyCount ?len } .
  ```

---

### 2.4. The `string:` String Manipulation Namespace
The `string:` namespace contains string parsing and formatting operations.

#### `string:concat`
* **Syntax**: `?list string:concat ?result`
* **Input**: `?list` must be bound to an RDF list of string literals.
* **Behavior**: Concatenates the string representations of list elements and binds or verifies the result string in `?result`.
* **Example**:
  ```turtle
  { (?first " " ?last) string:concat ?fullName } => { ?user :name ?fullName } .
  ```

#### `string:length`
* **Syntax**: `?str string:length ?len`
* **Input**: `?str` must be bound to a string literal.
* **Behavior**: Computes the UTF-8 character length of the string and binds or verifies the integer result in `?len`.
* **Example**:
  ```turtle
  { ?pw :value ?val . ?val string:length ?len . ?len math:greaterThan 8 } => { ?pw :status :strong } .
  ```

---

## 3. Rust Evaluator Integration Reference

Below is the Rust structural design of the built-in dispatcher loop inside the query engine:

```rust
use std::rc::Rc;

pub enum EvaluatedTerm {
    Integer(i64),
    Float(f64),
    String(String),
}

pub struct BuiltinDispatcher;

impl BuiltinDispatcher {
    pub fn dispatch(
        predicate_iri: &str,
        args: &[EvaluatedTerm],
    ) -> Option<EvaluatedTerm> {
        match predicate_iri {
            "http://www.w3.org/2000/10/swap/math#sum" => {
                Self::eval_math_sum(args)
            }
            "http://www.w3.org/2000/10/swap/string#concat" => {
                Self::eval_string_concat(args)
            }
            _ => None, // Not a recognized built-in predicate
        }
    }

    fn eval_math_sum(args: &[EvaluatedTerm]) -> Option<EvaluatedTerm> {
        let mut sum = 0.0;
        for arg in args {
            match arg {
                EvaluatedTerm::Integer(val) => sum += *val as f64,
                EvaluatedTerm::Float(val) => sum += *val,
                _ => return None, // Math operations are numeric only
            }
        }
        Some(EvaluatedTerm::Float(sum))
    }

    fn eval_string_concat(args: &[EvaluatedTerm]) -> Option<EvaluatedTerm> {
        let mut result = String::new();
        for arg in args {
            match arg {
                EvaluatedTerm::String(val) => result.push_str(val),
                _ => return None, // Concatenation operates on string terms
            }
        }
        Some(EvaluatedTerm::String(result))
    }
}
```
