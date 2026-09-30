---
format: aep.planning-md/3
id: review-result:goal-planner-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:goal-planner
relations:
- reviews: story:goal-planner
revision: 1
---
unit: goal-planner, branch unit/goal-planner, findings cover 1e5f26b (87a1844 plus one test-only commit)
verdict: NEEDS-CHANGE
cases: executed 20→26, red 2
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (deleted)
needs-coordinator: decide whether a step's target must resolve at the point it runs (a removed target), or only in the original document plus earlier steps' subtrees; this is a design call

**1. Diff stat (87a1844..HEAD):** `crates/uilab-agent/tests/plan_adversary.rs | 243 +++`, 1 file. It is a test file only, and no implementation path was touched. The commit is authored and committed by `b10x-bot[bot]` through `b10x-gates bot`.

**2. Cases added** (`crates/uilab-agent/tests/plan_adversary.rs`), first run alone with `cargo test -p uilab-agent --test plan_adversary`:

| case | asserts | now |
|---|---|---|
| `a_drawer_an_earlier_steps_batch_creates_is_a_target_the_plan_accepts` :119 | `admit` accepts the step-1 batch at `page:members/section:list`, which adds `page:members/overlay:edit`. `check_plan` should then accept step 2 at that overlay | red |
| `a_blank_instruction_is_not_a_step_the_runner_can_carry_out` :142 | `check_plan` refuses a step whose instruction is `""` or `"   "` | red |
| `a_target_under_no_earlier_target_is_refused_even_after_a_shorter_unrelated_step` :167 | `[page:loans, page:members/section:details]` gives `plan_target` naming step 2. It kills a mutant the old suite misses (below) | green |
| `the_cap_holds_at_exactly_n_and_refuses_n_plus_one` | N passes; N+1 and cap 0 give `plan_too_long`; empty with cap 0 gives `plan_empty` | green |
| `root_and_nav_are_plan_and_step_targets` | `/`, `nav` and `nav/nav_section:people` work as step targets; `plan_goal` works at `/` and at `nav` | green |
| `a_decline_after_a_refused_plan_is_a_decline` | a refused plan followed by `op: decline` returns `Declined` | green |

Red output from the run of this file alone:
```
---- a_blank_instruction_is_not_a_step_the_runner_can_carry_out stdout ----
panicked at crates/uilab-agent/tests/plan_adversary.rs:150:14:
a step with no instruction gives propose nothing to carry out: ()
---- a_drawer_an_earlier_steps_batch_creates_is_a_target_the_plan_accepts stdout ----
panicked at crates/uilab-agent/tests/plan_adversary.rs:138:10:
step 2 targets a node step 1 creates, which the acceptance says is valid: Refusal { check: "plan_target", message: "step 2 targets `page:members/overlay:edit`, which is no node of the document and not under any earlier step's target; put the step that creates it first, or target a node that exists" }
test result: FAILED. 4 passed; 2 failed
```
In the drawer case the `admit` expectation at :131 passed. So the failure is at `check_plan`, not in the fixture.

**3. Suite run:** `cargo test -p uilab-agent --no-fail-fast` exited 101.
- lib 4 ok, bin uilab-plan 1 ok, bin uilab-propose 0, propose.rs 15 ok, doc-tests 0.
- plan_adversary: 4 passed, 2 failed. The two failures are the same panics as above.
- The before count of 20 comes from a second run with plan_adversary left out: `--lib --bins --test propose` gave 4+1+0+15.
- `cargo clippy -p uilab-agent --all-targets -- -D warnings` is clean, and `cargo fmt -p uilab-agent --check` is clean.

**4. Findings** (all in `crates/uilab-agent/src/lib.rs`, at commit 1e5f26b)

| # | file:line | finding | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| F1 | lib.rs:189 | The target rule only accepts nodes under an earlier step's target. A batch at the list can insert the drawer under the page, and `admit` does not keep a batch's inner patches inside its target. So a plan whose later step targets that drawer is refused, although the acceptance says a target an earlier step creates is valid. | `PLAN_INSTRUCTIONS` says "A drawer or dialog and the row action that opens it belong in one step". The live goal asks for an edit drawer. A refusal costs a retry, and 2 refusals in a row end in `Refused`. | NEEDS-CHANGE | introduced |
| F2 | lib.rs:172 | `check_plan` accepts an empty or whitespace-only instruction. The schema has no `minLength` either. | Any model answer with `"instruction": ""`. The runner would then call `propose("")`. | NEEDS-CHANGE | introduced |
| F3 | lib.rs:189 | Mutant: without the `starts_with` test, any earlier step with a shorter target licenses every later target, and all old tests stay green. The unit's tests of `plan_target` refusal all have no shorter earlier step. | Suite gap. The new case at tests/plan_adversary.rs:167 kills it, and the in-test mutant `mutant_without_prefix` accepts that same input. | CONFIRMED | introduced |
| F4 | lib.rs:189 | A step whose target an earlier step removes still passes, because it resolves in the original document. A later target under that removed node also passes. `Step` has no op, so `check_plan` cannot see a removal. | A plan like "remove the member list, then add a column to it". No red case was written, because fixing it needs a design change. | INFEASIBLE | introduced |
| F5 | lib.rs:189 | One step at `/` makes every later target pass, because every path is under root. The same looseness applies at any depth, so the brief's "parent resolves or is an earlier target" rule is not what is enforced. | A plan that begins "add a reports page" at `/`. The unit's test named "at any depth" also passes under the brief's rule, so it does not tell the two rules apart. | CONFIRMED | introduced |

For F1, one possible fix: also accept a target under the parent of an earlier step's target. For F2: refuse a blank instruction with a new check such as `plan_step_blank`, and add `minLength: 1` to the schema. I did not apply either fix.

**5. Attacked and not broken**
- Cap boundaries (0, N, N+1): max_steps 0 gives `Config` before any turn, and the harness enforces `maxItems`.
- A decline, and a decline arriving on the retry.
- The single retry with the refusal fed back.
- Root and `nav` as targets.
- `propose` / `propose_with` after the shared attempt runner: the retry text is byte-identical to the base and all 6 older propose tests pass. The only visible change is the `Stopped` / `Refused` error text, "patch" → "answer", which `uilab-app` app.rs:688 shows as-is.

**6. Written outside the worktree:** `/home/timo/.cache/uilab-wave-w1/goal-planner/adversary-commitmsg.txt`, deleted after the commit. The build directory `/home/timo/.cache/b10x-target/uilab-w1-goal-planner` is the one the brief assigned.

```findings
- file: crates/uilab-agent/src/lib.rs
  line: 189
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: check_plan refuses a step targeting the drawer an earlier step's admitted batch creates under the page, because it only accepts targets under an earlier step's own target
- file: crates/uilab-agent/src/lib.rs
  line: 172
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a step with an empty or whitespace-only instruction passes check_plan and the plan schema
- file: crates/uilab-agent/src/lib.rs
  line: 189
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: dropping the starts_with conjunct of the target rule left the unit's suite green; killed by tests/plan_adversary.rs:167
- file: crates/uilab-agent/src/lib.rs
  line: 189
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a target removed by an earlier step still passes because Step carries no op, so removal is invisible to check_plan
- file: crates/uilab-agent/src/lib.rs
  line: 189
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: one step at the root makes every later target pass, which is looser than the brief's parent-resolves-or-earlier-target rule
```
