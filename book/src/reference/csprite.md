# CSprite Hierarchy Traversal

## Ontological Hierarchy Compilations

`CSprite` in [lib/src/csprite.rs](file:///Users/sac/roxi/lib/src/csprite.rs) compiles ontological relationships (such as `rdfs:subClassOf` and `rdfs:subPropertyOf`) into optimized lookup indexes, avoiding general-purpose rule execution costs.

---

## Cycle Guards in Traversal Helpers

Because class ontologies can contain circular subclass definitions, the `CSprite` compiler uses cycle guards at all recursive traversal sites:

1. **`eval_backward_csprite`**: Resolves subclass relations recursively.
2. **`eval_backward_csprite_helper`**: Recursion helper carrying the visited set.
3. **`eval_backward_csprite_helper_with_stack`**: Stack-based iterative fallback for deep hierarchies.
4. **`rewrite_hierarchy`**: Optimizes rule bodies during schema compilations.

If a circular dependency is detected in the class tree, the traversal terminates, preventing infinite loops during query compilation.
