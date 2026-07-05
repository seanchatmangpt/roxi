use crate::{BodyLiteral, Rule as ReasonerRule, Triple, VarOrTerm};
use std::collections::HashMap;

use pest::iterators::{Pair, Pairs};
use pest::Parser;

// ---------------------------------------------------------------------------
// Pest-generated parser
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[grammar = "parser/n3.pest"]
pub struct N3Parser;

// ---------------------------------------------------------------------------
// Prefix mapper
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PrefixMapper {
    prefixes: HashMap<String, String>,
}

impl PrefixMapper {
    pub fn new() -> PrefixMapper {
        PrefixMapper {
            prefixes: HashMap::new(),
        }
    }

    pub fn add(&mut self, prefix: String, full_iri: String) {
        self.prefixes.insert(prefix, full_iri);
    }

    /// Expand a prefixed name, a bare `a`, or a `<IRI>` reference.
    /// Returns the canonical `<IRI>` form.
    pub fn expand(&self, raw: &str) -> String {
        // Trim whitespace and trailing dots (N3 TP terminator may have been consumed)
        let trimmed = raw.trim().trim_end_matches('.');

        // rdf:type shorthand
        if trimmed == "a" {
            return "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string();
        }

        // Already a full <IRI>
        if trimmed.starts_with('<') && trimmed.ends_with('>') {
            return trimmed.to_string();
        }

        // Prefixed name (prefix:local) — local may also have trailing dots that were consumed
        if let Some(colon) = trimmed.find(':') {
            let prefix = &trimmed[..colon];
            // Strip any residual trailing dot from the local name
            let local = trimmed[colon + 1..].trim_end_matches('.');
            if let Some(expanded) = self.prefixes.get(prefix) {
                return format!("<{}{}>", expanded, local);
            }
        }

        // Return as-is (e.g., variable or unknown term)
        trimmed.to_string()
    }
}

impl Default for PrefixMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Term building helpers
// ---------------------------------------------------------------------------

/// Convert a raw string value (already expanded) into a VarOrTerm.
///
/// NOTE: the fallback branch deliberately calls `VarOrTerm::new_term` directly
/// rather than `VarOrTerm::convert`. `convert` wraps any string that isn't
/// already `<...>`/`"..."`/`_:...` in angle brackets -- which is exactly right
/// for a *bare* prefixed name with no matching `@prefix` (e.g. "test:Foo" with
/// no `@prefix test:` declared, where `PrefixMapper::expand` returns the text
/// unchanged as a fallback). Using `new_term` here keeps that fallback case
/// encoded identically to the legacy line-based parser (`Parser::parse`),
/// which also interns such tokens raw/unwrapped. This matters because
/// `TripleStore::from` tries this pest-based parser first and falls back to
/// the legacy parser on failure -- if both parsers can succeed on the same
/// kind of document, they must agree on term encoding or a test comparing
/// pre-existing (legacy-encoded) terms against freshly-parsed (pest-encoded)
/// ones would silently break.
fn make_term(raw: &str) -> VarOrTerm {
    let trimmed = raw.trim();
    if trimmed.starts_with('?') {
        // Strip the leading '?' to match VarOrTerm::convert("?x") behaviour which stores "x"
        VarOrTerm::new_var(trimmed[1..].to_string())
    } else if trimmed.starts_with("_:") {
        VarOrTerm::new_blank_node(trimmed[2..].to_string())
    } else {
        VarOrTerm::new_term(trimmed.to_string())
    }
}

/// Parse a literal pest pair into a VarOrTerm literal.
///
/// NOTE: numeric/boolean literals are encoded via `VarOrTerm::new_literal` with
/// a proper xsd datatype (mirroring the string-literal handling just below),
/// **not** via `VarOrTerm::convert`. `convert` would wrap a bare lexical form
/// like "42" in angle brackets ("<42>"), i.e. treat it as an opaque IRI-like
/// token instead of a numeric value -- which would make it impossible for the
/// math:* built-ins (queryengine.rs) to recover a numeric value from it.
fn parse_literal_pair(pair: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    // The outer Literal rule may contain StringValue + optional LangTag / DatatypeAnnotation,
    // or a numeric / boolean literal.
    let raw = pair.as_str().trim().to_string();
    let mut inner = pair.into_inner().peekable();

    if let Some(first) = inner.peek() {
        match first.as_rule() {
            Rule::StringValue => {
                let string_pair = inner.next().unwrap();
                let lex = unescape_string(string_pair.as_str());

                // Check for lang tag or datatype
                if let Some(annotation) = inner.next() {
                    match annotation.as_rule() {
                        Rule::LangTag => {
                            // @en → strip the @
                            let lang = &annotation.as_str()[1..];
                            return VarOrTerm::new_literal(lex, None, Some(lang.to_string()));
                        }
                        Rule::DatatypeAnnotation => {
                            // "^^" has been consumed by pest; the child is either a
                            // bare "<IRI>" (Rule::Iri) or a "prefix:local" name
                            // (Rule::Prefixed) that must be expanded via `prefixes`.
                            let dt_str = match annotation.into_inner().next() {
                                Some(p) if p.as_rule() == Rule::Iri => format!("<{}>", p.as_str()),
                                Some(p) if p.as_rule() == Rule::Prefixed => prefixes.expand(p.as_str()),
                                _ => String::new(),
                            };
                            return VarOrTerm::new_literal(lex, Some(dt_str), None);
                        }
                        _ => {}
                    }
                }
                // Plain string literal → xsd:string
                let xsd_string = "<http://www.w3.org/2001/XMLSchema#string>".to_string();
                return VarOrTerm::new_literal(lex, Some(xsd_string), None);
            }
            Rule::IntegerLiteral => {
                return VarOrTerm::new_literal(
                    raw,
                    Some("<http://www.w3.org/2001/XMLSchema#integer>".to_string()),
                    None,
                );
            }
            Rule::DecimalLiteral => {
                return VarOrTerm::new_literal(
                    raw,
                    Some("<http://www.w3.org/2001/XMLSchema#decimal>".to_string()),
                    None,
                );
            }
            Rule::DoubleLiteral => {
                return VarOrTerm::new_literal(
                    raw,
                    Some("<http://www.w3.org/2001/XMLSchema#double>".to_string()),
                    None,
                );
            }
            Rule::BoolLiteral => {
                return VarOrTerm::new_literal(
                    raw,
                    Some("<http://www.w3.org/2001/XMLSchema#boolean>".to_string()),
                    None,
                );
            }
            _ => {}
        }
    }

    // Should not normally be reached given the Literal grammar's alternatives.
    VarOrTerm::convert(raw)
}

/// Parse an RDF list ("(" ListItem* ")") into a single VarOrTerm list term.
fn parse_list(pair: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    let mut members = Vec::new();
    for list_item in pair.into_inner() {
        if let Some(child) = list_item.into_inner().next() {
            members.push(term_from_pair(child, prefixes));
        }
    }
    VarOrTerm::new_list(members)
}

/// Parse a quoted graph ("{" TP* "}") into a single VarOrTerm formula term.
fn parse_formula(pair: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    let mut triples = Vec::new();
    for tp_pair in pair.into_inner() {
        if tp_pair.as_rule() == Rule::TP {
            triples.extend(parse_tp(tp_pair.into_inner(), prefixes));
        }
    }
    VarOrTerm::new_formula(triples)
}

/// Shared term-building logic for anything that can appear in a Subject or
/// Object position (IRI, prefixed name, variable, blank node, literal, list,
/// or quoted graph).
fn term_from_pair(child: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    match child.as_rule() {
        Rule::IriRef => {
            let iri = child.into_inner().next().map(|p| p.as_str()).unwrap_or("");
            make_term(&format!("<{}>", iri))
        }
        Rule::Prefixed => make_term(&prefixes.expand(child.as_str())),
        Rule::Var => make_term(child.as_str()),
        Rule::BlankNode => make_term(child.as_str()),
        Rule::Literal => parse_literal_pair(child, prefixes),
        Rule::List => parse_list(child, prefixes),
        Rule::Formula => parse_formula(child, prefixes),
        _ => make_term(child.as_str()),
    }
}

/// Strip surrounding quotes from a string literal.
fn unescape_string(raw: &str) -> String {
    let s = raw.trim();
    if s.starts_with("\"\"\"") && s.ends_with("\"\"\"") {
        s[3..s.len()-3].to_string()
    } else if s.starts_with("'''") && s.ends_with("'''") {
        s[3..s.len()-3].to_string()
    } else if (s.starts_with('"') && s.ends_with('"'))
        || (s.starts_with('\'') && s.ends_with('\''))
    {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

// ---------------------------------------------------------------------------
// Triple pattern parsing
// ---------------------------------------------------------------------------

/// Parse a single TP (triple pattern) production into one or more `Triple`s.
/// More than one triple results when the object position uses comma sugar
/// ("s p o1, o2, o3 .") -- each object shares the same subject/property.
fn parse_tp(pairs: Pairs<'_, Rule>, prefixes: &PrefixMapper) -> Vec<Triple> {
    let mut subject_vot = VarOrTerm::new_var("s".to_string());
    let mut property_vot = VarOrTerm::new_var("p".to_string());
    let mut objects_vot: Vec<VarOrTerm> = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::Subject => {
                subject_vot = parse_subject(pair, prefixes);
            }
            Rule::Property => {
                let expanded = expand_property(pair, prefixes);
                property_vot = make_term(&expanded);
            }
            Rule::ObjectList => {
                for obj_pair in pair.into_inner() {
                    if obj_pair.as_rule() == Rule::Object {
                        objects_vot.push(parse_object(obj_pair, prefixes));
                    }
                }
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    if objects_vot.is_empty() {
        objects_vot.push(VarOrTerm::new_var("o".to_string()));
    }

    objects_vot
        .into_iter()
        .map(|o| Triple {
            s: subject_vot.clone(),
            p: property_vot.clone(),
            o,
            g: None,
        })
        .collect()
}

fn parse_subject(pair: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    match pair.into_inner().next() {
        Some(child) => term_from_pair(child, prefixes),
        None => VarOrTerm::new_var("s".to_string()),
    }
}

fn expand_property(pair: Pair<Rule>, prefixes: &PrefixMapper) -> String {
    let inner = pair.into_inner().next();
    if let Some(child) = inner {
        match child.as_rule() {
            Rule::RdfType => "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>".to_string(),
            Rule::IriRef => {
                let iri = child.into_inner().next().map(|p| p.as_str()).unwrap_or("");
                format!("<{}>", iri)
            }
            Rule::Prefixed => prefixes.expand(child.as_str()),
            Rule::Var => child.as_str().to_string(),
            _ => child.as_str().to_string(),
        }
    } else {
        String::new()
    }
}

fn parse_object(pair: Pair<Rule>, prefixes: &PrefixMapper) -> VarOrTerm {
    match pair.into_inner().next() {
        Some(child) => term_from_pair(child, prefixes),
        None => VarOrTerm::new_var("o".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Public parse function
// ---------------------------------------------------------------------------

/// Parse a complete N3 document into its plain (non-rule) fact triples and
/// its rules, in one unified pest-based pass.
///
/// Supports:
/// - `@prefix` declarations (anywhere in the document)
/// - Plain top-level fact triples ("s p o .", including comma-sugar object lists)
/// - `{body} => {head} .` rules with negated literals (`not { TP }`)
/// - Named IRIs (`<...>`), prefixed names, variables (`?name`), blank nodes (`_:x`)
/// - String, numeric, boolean, lang-tagged, and datatyped literals
/// - RDF lists (`( a b c )`) and quoted graphs (`{ a b c }`) used as terms
/// - Multi-triple heads
/// - `#` line comments
///
/// Returns `Err(String)` on parse failure.
pub fn parse_document(input: &str) -> Result<(Vec<Triple>, Vec<ReasonerRule>), String> {
    let mut rules: Vec<ReasonerRule> = Vec::new();
    let mut content: Vec<Triple> = Vec::new();
    let mut prefix_mapper = PrefixMapper::new();

    let parsed = N3Parser::parse(Rule::document, input)
        .map_err(|e| format!("N3 parse error: {}", e))?;

    let document = match parsed.into_iter().next() {
        Some(p) => p,
        None => return Ok((content, rules)),
    };

    for item in document.into_inner() {
        match item.as_rule() {
            Rule::Prefix => {
                let mut prefix_name = String::new();
                let mut prefix_iri = String::new();
                for child in item.into_inner() {
                    match child.as_rule() {
                        Rule::PrefixIdentifier => prefix_name = child.as_str().to_string(),
                        Rule::Iri => prefix_iri = child.as_str().to_string(),
                        _ => {}
                    }
                }
                prefix_mapper.add(prefix_name, prefix_iri);
            }

            Rule::TP => {
                content.extend(parse_tp(item.into_inner(), &prefix_mapper));
            }

            // `rule` wraps exactly one of `forward_rule` ("{body} => {head}")
            // or `backward_rule` ("{head} <= {body}"). Both grammar
            // productions still name their braces `Body`/`Head` according to
            // their *semantic* role (see the grammar comment), so the same
            // extraction logic below handles both without needing to know
            // which one it is.
            Rule::rule => {
                for variant in item.into_inner() {
                    if variant.as_rule() != Rule::forward_rule && variant.as_rule() != Rule::backward_rule {
                        continue;
                    }

                    let mut body: Vec<BodyLiteral> = Vec::new();
                    let mut head_triples: Vec<Triple> = Vec::new();

                    for part in variant.into_inner() {
                        match part.as_rule() {
                            Rule::Body => {
                                for bl_pair in part.into_inner() {
                                    // bl_pair is a BodyLiteral
                                    let is_negated = bl_pair.as_str().trim_start().starts_with("not");
                                    // Find the TP inside the BodyLiteral
                                    let tp_pair = bl_pair
                                        .into_inner()
                                        .find(|p| p.as_rule() == Rule::TP)
                                        .expect("BodyLiteral must contain a TP");
                                    let patterns = parse_tp(tp_pair.into_inner(), &prefix_mapper);
                                    for pattern in patterns {
                                        body.push(BodyLiteral { negated: is_negated, pattern });
                                    }
                                }
                            }
                            Rule::Head => {
                                for tp_pair in part.into_inner() {
                                    if tp_pair.as_rule() == Rule::TP {
                                        head_triples.extend(parse_tp(tp_pair.into_inner(), &prefix_mapper));
                                    }
                                }
                            }
                            Rule::EOI => {}
                            _ => {}
                        }
                    }

                    // Emit one rule per head triple (multi-head rules desugar to multiple rules)
                    for head in head_triples {
                        rules.push(ReasonerRule {
                            body: body.clone(),
                            head,
                        });
                    }
                }
            }

            Rule::EOI => {}
            _ => {}
        }
    }

    Ok((content, rules))
}

/// Parse an N3-rule string into a list of Datalog `Rule`s (discarding any
/// plain top-level fact triples -- use `parse_document` to get both).
pub fn parse(input: &str) -> Result<Vec<ReasonerRule>, String> {
    parse_document(input).map(|(_content, rules)| rules)
}

#[cfg(test)]
#[path = "n3rule_parser_test.rs"]
mod n3rule_parser_test;
