---
format: aep.planning-md/3
id: review-result:goal-planner-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:goal-planner
relations:
- reviews: story:goal-planner
revision: 1
---
unit: goal-planner pass 2, correction efc792e on unit/goal-planner; findings cover f90fb2c (efc792e plus one test-only commit)
verdict: CONFIRMED (one note-level suite gap, now closed by a new case; nothing red)
cases: executed 29→33, red 0
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (deleted)
needs-coordinator: none

**1. Diff stat (efc792e..HEAD):** `crates/uilab-agent/tests/plan_adversary.rs | 120 +++`, 1 file. It is a test file only, and the commit is authored and committed by `b10x-bot[bot]`.

**2. Cases added**, first run alone with `cargo test -p uilab-agent --test plan_adversary`: 10 passed, 0 failed.

| case | asserts | now |
|---|---|---|
| `a_step_before_the_step_whose_target_is_its_parent_is_refused` | `[…/section:details/item:loan, …/section:details]` gives `plan_target` naming step 1. The in-test mutant `mutant_any_step` accepts the same plan | green, kills a surviving mutant (F1) |
| `the_decided_target_rule_at_each_boundary` | parent resolves: ok; new child of `nav`: ok; parent is an earlier target: ok; only the grandparent is earlier: `plan_target` on step 2; neither it nor its parent resolves: `plan_target` | green |
| `an_instruction_of_only_unicode_whitespace_is_blank` | `\u{a0}`, `\u{3000}` and `\n\r` give `plan_step_blank` | green |
| `a_blank_target_after_a_valid_step_names_its_step` | a `"\t"` target in step 2 is refused twice as `plan_step_blank`, naming step 2 | green |

**3. Suite run:** `cargo test -p uilab-agent --no-fail-fast` exited 0.
- lib 6, bin uilab-plan 1, bin uilab-propose 0, plan_adversary 10, propose 16, doc-tests 0.
- The before count of 29 comes from the implementor's commit message ("26 -> 29"). It matches this run with my 4 new cases left out: 23 outside plan_adversary plus its 6 older cases.
- Clippy with `-D warnings` is clean, and `cargo fmt --check` is clean.

**4. Findings**

| # | file:line | finding | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| F1 | crates/uilab-agent/src/lib.rs:248 | Mutant: replacing `steps[..index]` with `steps` (which ignores step order) left the efc792e suite green. None of its `plan_target` refusals has a later step whose target is the parent. It is now killed by `a_step_before_the_step_whose_target_is_its_parent_is_refused`. | Suite gap only; the code is correct. | CONFIRMED | introduced |

**Did the rewrite of my pass-1 cases relax them? No.**
- **The shorter unrelated step case:** its old input, `[page:loans, page:members/section:details]`, is valid under the decided rule because `page:members` resolves. So it had to change. The new input still fails on an earlier step that has nothing to do with the target, and it kills the depth-only mutant.
- **The root/nav case:** only the `plan_goal` loop's second step changed, from `page:reports/section:table` to `page:reports`. The old one targets a grandchild of `/`, which the decided rule refuses. The `check_plan` block and the `steps[0] == root` assertion are unchanged.
- **The drawer and blank-instruction cases:** unchanged, and both are now green.

**5. Attacked and not broken**
- The decided rule at every boundary in the table above.
- `/` and `nav` as targets.
- A blank instruction or target, whether empty (schema `minLength`), whitespace or Unicode whitespace (`plan_step_blank`), including in a later step.
- The retry that follows a blank-target refusal.
- `propose` / `propose_with`: efc792e does not touch them, and all 6 older propose tests pass.

**6. Written outside the worktree**
- `/home/timo/.cache/uilab-wave-w1/goal-planner/adversary-p2-commitmsg.txt`, deleted after the commit.
- The build directory is the assigned `/home/timo/.cache/b10x-target/uilab-w1-goal-planner`.
- Free disk was 9.9G at the start of this pass, below the brief's 10G stop line. I did not build until it was back at 13.1G.

```findings
- file: crates/uilab-agent/src/lib.rs
  line: 248
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: replacing steps[..index] with steps in the parent-is-earlier-target test left the efc792e suite green; killed by tests/plan_adversary.rs a_step_before_the_step_whose_target_is_its_parent_is_refused
```
