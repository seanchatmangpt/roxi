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
fn make_term(raw: &str) -> VarOrTerm {
    let trimmed = raw.trim();
    if trimmed.starts_with('?') {
        // Strip the leading '?' to match VarOrTerm::convert("?x") behaviour which stores "x"
        VarOrTerm::new_var(trimmed[1..].to_string())
    } else if trimmed.starts_with("_:") {
        VarOrTerm::new_blank_node(trimmed[2..].to_string())
    } else {
        VarOrTerm::convert(trimmed.to_string())
    }
}

/// Parse a literal pest pair into a VarOrTerm literal.
fn parse_literal_pair(pair: Pair<Rule>) -> VarOrTerm {
    // The outer Literal rule may contain StringValue + optional LangTag / DatatypeAnnotation,
    // or a numeric / boolean literal.
    let raw = pair.as_str();
    let mut inner = pair.into_inner().peekable();

    // Check if first child is a StringValue
    if let Some(first) = inner.peek() {
        if first.as_rule() == Rule::StringValue {
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
                        // "^^" has been consumed by pest; the child is the datatype IRI
                        let dt_str = annotation.into_inner().next()
                            .map(|p| format!("<{}>", p.as_str().trim_matches(|c| c == '<' || c == '>')))
                            .unwrap_or_default();
                        let dt_encoded = crate::encoding::Encoder::add(dt_str);
                        return VarOrTerm::new_literal(lex, Some(format!("{}", dt_encoded)), None);
                    }
                    _ => {}
                }
            }
            // Plain string literal → xsd:string
            let xsd_string = "<http://www.w3.org/2001/XMLSchema#string>".to_string();
            let dt_encoded = crate::encoding::Encoder::add(xsd_string);
            return VarOrTerm::new_literal(lex, Some(format!("{}", dt_encoded)), None);
        }
    }

    // Numeric / boolean literals — convert using VarOrTerm::convert on the raw text
    // which the triple store understands as encoded IDs.
    VarOrTerm::convert(raw.to_string())
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

fn parse_tp(pairs: Pairs<'_, Rule>, prefixes: &PrefixMapper) -> Triple {
    let mut subject_vot = VarOrTerm::new_var("s".to_string());
    let mut property_vot = VarOrTerm::new_var("p".to_string());
    let mut object_vot = VarOrTerm::new_var("o".to_string());

    for pair in pairs {
        match pair.as_rule() {
            Rule::Subject => {
                let expanded = expand_subject(pair, prefixes);
                subject_vot = make_term(&expanded);
            }
            Rule::Property => {
                let expanded = expand_property(pair, prefixes);
                property_vot = make_term(&expanded);
            }
            Rule::Object => {
                object_vot = parse_object(pair, prefixes);
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Triple {
        s: subject_vot,
        p: property_vot,
        o: object_vot,
        g: None,
    }
}

fn expand_subject(pair: Pair<Rule>, prefixes: &PrefixMapper) -> String {
    let inner = pair.into_inner().next();
    if let Some(child) = inner {
        match child.as_rule() {
            Rule::IriRef => {
                let iri = child.into_inner().next().map(|p| p.as_str()).unwrap_or("");
                format!("<{}>", iri)
            }
            Rule::Prefixed => prefixes.expand(child.as_str()),
            Rule::Var => child.as_str().to_string(),
            Rule::BlankNode => child.as_str().to_string(),
            _ => child.as_str().to_string(),
        }
    } else {
        String::new()
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
    let inner = pair.into_inner().next();
    if let Some(child) = inner {
        match child.as_rule() {
            Rule::IriRef => {
                let iri = child.into_inner().next().map(|p| p.as_str()).unwrap_or("");
                make_term(&format!("<{}>", iri))
            }
            Rule::Prefixed => make_term(&prefixes.expand(child.as_str())),
            Rule::Var => make_term(child.as_str()),
            Rule::BlankNode => make_term(child.as_str()),
            Rule::Literal => parse_literal_pair(child),
            _ => make_term(child.as_str()),
        }
    } else {
        VarOrTerm::new_var("?o".to_string())
    }
}

// ---------------------------------------------------------------------------
// Public parse function
// ---------------------------------------------------------------------------

/// Parse an N3-rule string into a list of Datalog `Rule`s.
///
/// Supports:
/// - `@prefix` declarations
/// - `{body} => {head} .` rules with negated literals (`not { TP }`)
/// - Named IRIs (`<...>`), prefixed names, variables (`?name`), blank nodes (`_:x`)
/// - String, numeric, boolean, lang-tagged, and datatyped literals in object position
/// - Multi-triple heads
/// - `#` line comments
///
/// Returns `Err(String)` on parse failure.
pub fn parse(input: &str) -> Result<Vec<ReasonerRule>, String> {
    let mut rules: Vec<ReasonerRule> = Vec::new();
    let mut prefix_mapper = PrefixMapper::new();

    let parsed = N3Parser::parse(Rule::document, input)
        .map_err(|e| format!("N3 parse error: {}", e))?;

    let document = match parsed.into_iter().next() {
        Some(p) => p,
        None => return Ok(rules),
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

            Rule::rule => {
                let mut body: Vec<BodyLiteral> = Vec::new();
                // Default head (overwritten by first Head TP)
                let mut head_triples: Vec<Triple> = Vec::new();

                for part in item.into_inner() {
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
                                let pattern = parse_tp(tp_pair.into_inner(), &prefix_mapper);
                                body.push(BodyLiteral { negated: is_negated, pattern });
                            }
                        }
                        Rule::Head => {
                            for tp_pair in part.into_inner() {
                                if tp_pair.as_rule() == Rule::TP {
                                    head_triples.push(parse_tp(tp_pair.into_inner(), &prefix_mapper));
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

            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(rules)
}

#[cfg(test)]
#[path = "n3rule_parser_test.rs"]
mod n3rule_parser_test;
