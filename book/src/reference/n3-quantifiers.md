# Quantifiers & Scoping

## Universal and Existential Quantifiers

Notation3 supports explicit variable quantification using `@forAll` and `@forSome`:

* **`@forAll`**: Universal quantification (declares variables that match any term in the domain).
* **`@forSome`**: Existential quantification (declares variables that assert the existence of a term).

---

## Variable Scoping

In Roxi, the scope of a quantified variable is strictly bound to the formula or rule in which it is declared:

* **Isolation**: Variables declared in a nested formula do not leak into parent scopes, preventing conflicts.
* **EYE Compliance**: Scoping rules are aligned with the EYE reasoner, ensuring standard N3 compatibility.
