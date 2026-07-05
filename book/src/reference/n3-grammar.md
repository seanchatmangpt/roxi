# Notation3 (N3) Grammar & Parser

## 1. Syntax Structures

Notation3 (N3) extends the standard RDF Turtle syntax by adding logical rules, variables, and formulas. To parse these structures correctly, Roxi utilizes a parsing pipeline built on the **Pest** PEG (Parsing Expression Grammar) framework.

The parsing sequence executes as follows:

```
        Raw N3 Text String
                |
                v
       Pest Grammar Parser
  (Generates AST Pairs/Tokens)
                |
                v
     N3RuleParser Translator
  (Prefixes, Terms, Rule Blocks)
                |
                v
       Compiled Rule Structs
    (Inserted into RuleIndex)
```

---

## 2. Pest Grammar Specification

The grammar rules are defined in [lib/src/parser/n3.pest](file:///Users/sac/roxi/lib/src/parser/n3.pest). Here is the detailed layout of the core production rules:

```peg
// Whitespace and comments are automatically handled
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT = _{ "#" ~ (!"\n" ~ ANY)* ~ "\n" }

// Entrypoint Document structure
document = { SOI ~ (prefix_decl | rule_decl | triple_decl)* ~ EOI }

prefix_decl = { "@prefix" ~ prefix_label? ~ ":" ~ iri ~ "." }
prefix_label = { ASCII_ALPHANUMERIC+ }

// Rule Declarations: Swapping => and <= implications
rule_decl = { formula ~ implication_op ~ formula ~ "." }
implication_op = { "=>" | "<=" }

formula = { "{" ~ triple_pattern* ~ "}" }

triple_pattern = { subject ~ predicate ~ object ~ "."? }
triple_decl = { subject ~ predicate ~ object ~ "." }

subject = { var | iri | prefixed_name | blank_node | formula | rdf_list }
predicate = { var | iri | prefixed_name | a_shorthand }
object = { var | iri | prefixed_name | blank_node | literal | formula | rdf_list }

// Shorthands
a_shorthand = { "a" }

// Variables and Terms
var = { "?" ~ ASCII_ALPHANUMERIC+ }
prefixed_name = { prefix_label? ~ ":" ~ local_name }
local_name = { (ASCII_ALPHANUMERIC | "_" | "-")+ }

blank_node = { "_:" ~ ASCII_ALPHANUMERIC+ }
rdf_list = { "(" ~ object* ~ ")" }

iri = { "<" ~ (!">" ~ ANY)* ~ ">" }
literal = { string_literal ~ (lang_tag | datatype_suffix)? }
string_literal = { "\"" ~ (!"\"" ~ ANY)* ~ "\"" }
lang_tag = { "@" ~ ASCII_ALPHA+ }
datatype_suffix = { "^^" ~ iri }
```

---

## 3. Rule Compilation & Decomposition

During the AST translation pass in [lib/src/parser/n3rule_parser.rs](file:///Users/sac/roxi/lib/src/parser/n3rule_parser.rs), the compiler performs several key transformations:

### Implication Operator Resolution
* **Forward Implication (`=>`)**: The left-hand formula becomes the `body` (premises), and the right-hand formula becomes the `head` (conclusions).
* **Backward Implication (`<=`)**: The parser swaps the formulas: the right-hand formula becomes the rule `body`, and the left-hand formula becomes the `head`.

### Multi-Triple Head Decomposition
If the head formula of a rule contains multiple triple patterns, Roxi decomposes it into separate rules at compile time. This allows the engine to evaluate rules efficiently without requiring complex multi-conclusion reasoning code.

For example, the rule:
```turtle
{ ?x :parentOf ?y } => { ?y :childOf ?x . ?x :hasOffspring :true } .
```

Is compiled and registered in the `RuleIndex` as two independent rules:
1. `{ ?x :parentOf ?y } => { ?y :childOf ?x }`
2. `{ ?x :parentOf ?y } => { ?x :hasOffspring :true }`

---

## 4. Error Propagation & Result Mapping

Roxi's parser propagates parse errors cleanly up the call stack:

* **No Panics**: The parser returns a `Result<Vec<Rule>, ParseError>` instead of panicking on malformed text.
* **Error Context**: The `ParseError` struct captures the line and column number of the syntax failure, allowing developers to debug rules files easily.

---

## 5. Rust Parser Implementation Reference

Below is the Rust code interface for compiling N3 rules:

```rust
extern crate pest;
use pest::Parser;
use pest_derive::Parser as PestParser;

#[derive(PestParser)]
#[grammar = "parser/n3.pest"]
pub struct N3Parser;

#[derive(Debug)]
pub enum ParseError {
    PestError(String),
    UnsupportedSyntax(String),
}

pub struct N3Compiler;

impl N3Compiler {
    /// Parses an N3 rules string and returns compiled Rules
    pub fn compile_rules(input: &str) -> Result<Vec<RawRule>, ParseError> {
        let mut rules = Vec::new();
        
        let parsed_doc = N3Parser::parse(Rule::document, input)
            .map_err(|e| ParseError::PestError(e.to_string()))?
            .next()
            .unwrap();

        for pair in parsed_doc.into_inner() {
            match pair.as_rule() {
                Rule::rule_decl => {
                    let mut inner = pair.into_inner();
                    let body_pair = inner.next().unwrap();
                    let op_pair = inner.next().unwrap();
                    let head_pair = inner.next().unwrap();

                    let raw_body = Self::parse_formula(body_pair)?;
                    let raw_head = Self::parse_formula(head_pair)?;

                    let is_forward = op_pair.as_str() == "=>";

                    // Create rule bindings, swapping if backward implication
                    let (body, head) = if is_forward {
                        (raw_body, raw_head)
                    } else {
                        (raw_head, raw_body)
                    };

                    // Decompose multi-head rule into individual rules
                    for head_triple in head {
                        rules.push(RawRule {
                            body: body.clone(),
                            head: head_triple,
                        });
                    }
                }
                _ => {} // Skip prefixes and triples in rule compiler
            }
        }

        Ok(rules)
    }

    fn parse_formula(_pair: pest::iterators::Pair<Rule>) -> Result<Vec<RawTriple>, ParseError> {
        // Mock parser helper
        Ok(Vec::new())
    }
}

pub struct RawTriple {
    pub s: String,
    pub p: String,
    pub o: String,
}

pub struct RawRule {
    pub body: Vec<RawTriple>,
    pub head: RawTriple,
}
```
