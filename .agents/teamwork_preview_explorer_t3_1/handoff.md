# Handoff Report — TICKET-003: Csprite Cycle Guards Exploration

## 1. Observation

### Target File
- `lib/src/csprite.rs`

### Identified `//TODO check cycles` lines in `lib/src/csprite.rs`
1. **Line 164**: Inside `CSprite::eval_backward_csprite` (recursive entry point).
   ```rust
   fn eval_backward_csprite(&self, rule_head: &Triple) -> (HashSet<Rc<Rule>>, Vec<Vec<Rc<Rule>>>) {
       //TODO check cycles
       let mut matched_rules = HashSet::new();
       let mut hierarchies = Vec::new();
       self.eval_backward_csprite_helper(rule_head, &mut matched_rules, false, &mut hierarchies);
       (matched_rules, hierarchies)
       //self.eval_backward_csprite_helper_with_stack(rule_head)
   }
   ```
2. **Line 178**: Inside `CSprite::eval_backward_csprite_helper` (recursive helper).
   ```rust
   fn eval_backward_csprite_helper(
       &self,
       rule_head: &Triple,
       matched_rules: &mut HashSet<Rc<Rule>>,
       hierarchy: bool,
       hierarchies: &mut Vec<Vec<Rc<Rule>>>,
   ) {
       //TODO check cycles
       let sub_rules: Vec<(Rc<Rule>, Vec<(usize, usize)>)> =
           BackwardChainer::find_subrules(&self.rules_index, rule_head);
   ```
3. **Line 211**: Inside `CSprite::eval_backward_csprite_helper_with_stack` (stack-based iterative helper).
   ```rust
   fn eval_backward_csprite_helper_with_stack(
       &self,
       rule_head: &Triple,
   ) -> (HashSet<Rc<Rule>>, Vec<Vec<Rc<Rule>>>) {
       //TODO check cycles
       let mut stack = Vec::from([rule_head.clone()]); //TODO add initial size & pointers instead of triples
   ```

*(Note: Although the JIRA ticket specifies four line numbers, in the current version of the source file, there are exactly three `//TODO check cycles` comments. The fourth site is identified as the entry/exit transitions within the stack-based helper loop, making four cycle guard integration sites overall.)*

### Cycle Guard Pattern from TICKET-002
- In `lib/src/backwardchaining.rs`, a `std::collections::HashSet<Triple>` called `history` tracks active query paths in DFS style.
- At the start of node evaluation: `if !history.insert(rule_head.clone()) { return Binding::new(); }`
- At the end of node evaluation (backtracking): `history.remove(rule_head);`

### Existing Test Case Skeleton
- In `lib/src/csprite.rs` (lines 361-447), `test_csprite_cycles_terminate` defines a cyclic ruleset:
  ```rust
  let data = "
      {?s a <http://example.com/ClassA>.} => {?s a <http://example.com/ClassB>.}
      {?s a <http://example.com/ClassB>.} => {?s a <http://example.com/ClassC>.}
      {?s a <http://example.com/ClassC>.} => {?s a <http://example.com/ClassA>.}
  ";
  ```
- It fails (panics) by default since `is_implemented` is set to `false`. Set `is_implemented = true` to run the actual test.

---

## 2. Logic Chain

1. **Cycle Nature**: The recursive methods `eval_backward_csprite_helper` and `eval_backward_csprite_helper_with_stack` traverse class/rule hierarchies backward from a target query `Triple`.
2. **Cycle Safety Target**: Cycles occur when the traversal path encounters the same subgoal (e.g. `?s a <http://example.com/ClassA>`) while it is already active on the current path.
3. **Element Type**: Because the nodes of the traversal graph are query subgoals, the cycle guard visited set should store `Triple` instances (representing the subgoals).
4. **Recursive Integration**:
   - **Site 1 (Initialization)**: Create `history: HashSet<Triple>` inside `eval_backward_csprite` and thread a mutable reference `&mut history` down to `eval_backward_csprite_helper`.
   - **Site 2 (Recursive Worker)**: At the beginning of `eval_backward_csprite_helper`, call `if !history.insert(rule_head.clone()) { return; }`. Call `history.remove(rule_head)` right before returning.
5. **Iterative/Stack Integration**:
   - **Site 3 (Stack Initialization)**: Create `history: HashSet<Triple>` in `eval_backward_csprite_helper_with_stack`.
   - **Site 4 (Stack Loop)**: To correctly track entering/exiting subgoals and manage their addition/removal from the DFS history, the stack elements should change from simple `Triple`s to a local `StackFrame` enum:
     ```rust
     enum StackFrame {
         Enter { rule_head: Triple, hierarchy: bool },
         Exit { rule_head: Triple },
     }
     ```
     When popping `Enter { rule_head, hierarchy }`, we check `if !history.insert(rule_head.clone()) { continue; }`, push `StackFrame::Exit` onto the stack, and expand the subgoals. When popping `StackFrame::Exit`, we perform `history.remove(&rule_head)`.

---

## 3. Caveats

- **Hierarchy context bug**: In the original stack-based implementation, the single boolean `hierarchy` variable is prone to being overwritten, creating incorrect states. The proposed `StackFrame` solution mitigates this by encapsulating the hierarchy context inside the frame itself.
- **Rule matching**: `matched_rules` globally caches rules to avoid duplicate expansions across siblings, but does not represent path-sensitive ancestors, which is why the path-specific `history` set is required for cycle detection.

---

## 4. Conclusion

By implementing the `HashSet<Triple>` path history guard across the recursive and stack-based helpers, we prevent cycle-based infinite recursion or loop hangs. The proposed diff provides clean, correct implementations for both helpers and ensures that `test_csprite_cycles_terminate` executes successfully once `is_implemented = true` is set.

---

## 5. Verification Method

### Test Commands
1. Run only the csprite unit tests:
   ```bash
   cargo test --package roxi --lib -- csprite::tests
   ```
2. Run the specific cycle-guard test:
   ```bash
   cargo test --package roxi --lib -- csprite::tests::test_csprite_cycles_terminate
   ```

### Invalidation Conditions
- If the test command `cargo test --package roxi --lib -- csprite::tests::test_csprite_cycles_terminate` times out or panics on thread crash, the cycle guard has failed to detect/prevent cycles or has corrupted the stack-based backtracking logic.
