# Plan - Implement TICKET-001 (Term Model Expansion)

## Phase 1: Implement `Term` and associated structures in `triples.rs`
1. Define `Term` enum:
   - `Iri(TermImpl)`
   - `Literal(LiteralImpl)`
   - `BlankNode(BlankNodeImpl)`
2. Define `TermImpl`, `LiteralImpl`, `BlankNodeImpl` structs.
3. Update `VarOrTerm`:
   - Replace `Term(TermImpl)` with `Term(Term)`.
   - Update `new_term` to parse/intern the string and return a `VarOrTerm::Term(Term)`.
   - Update `new_encoded_term` to use `Encoder::decode_to_term(id)` or fallback to IRI if not found (or return `VarOrTerm::Term(...)`).
   - Implement `as_term(&self) -> &Term`.
   - Update `to_encoded(&self) -> usize` to return `term.id()` for `VarOrTerm::Term(term)`.
   - Update `convert` logic to parse terms appropriately.
4. Implement `test_literal_term_roundtrip` and `test_blank_node_term_encoding` in `triples.rs`.

## Phase 2: Extend `Encoder` and implement `EncodedValue` in `encoding.rs`
1. Define `EncodedValue` enum.
2. Update `InternalEncoder` and `Encoder` to map `EncodedValue` to `usize` and back.
3. Expose `add_iri`, `add_blank_node`, `add_literal` on `Encoder` and `InternalEncoder`.
4. Implement `add(String) -> usize` to auto-detect the kind (IRI, BlankNode, Literal, Variable) and delegate.
5. Implement `decode_to_term(usize) -> Option<Term>`.
6. Implement `decode(id: &usize) -> Option<String>` to format values back into standard Turtle/NTriples strings.
7. Implement `test_encoder_literal_vs_iri_distinct` and `test_literal_datatype_and_langtag_preserved` in `encoding.rs`.

## Phase 3: Update match/access sites across codebase
1. Update `lib/src/tripleindex.rs` to use `Option<Term>` instead of `Option<TermImpl>` and change `.iri` to `.id()`.
2. Update `lib/src/reasoner.rs` where `.iri` is accessed or `VarOrTerm::Term(s_term)` is matched.
3. Update `lib/src/dred.rs` where `.iri` is accessed.
4. Update `lib/src/sparql.rs`:
   - Change `PlanExpression::Constant(TermImpl)` to `PlanExpression::Constant(Term)`.
   - Parse `Expression::Literal(value)` in `extract_expression` into a proper interned `Term::Literal`.
   - Evaluate `PlanExpression::Constant(term)` in `eval_expression` by parsing its lexical value if it's an integer.

## Phase 4: Verification and compilation fixes
1. Run `cargo check` and resolve all compiler errors.
2. Run `cargo test --lib --bins` to verify all 68 tests pass and the 4 new verification tests pass.
