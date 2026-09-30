---
format: aep.planning-md/3
id: review-result:widget-opens-at-use-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:widget-opens-at-use
relations:
- reviews: story:widget-opens-at-use
revision: 1
---
unit: story:widget-opens-at-use, dba026d plus adversary commit 5d09145 on unit/widget-opens-at-use
verdict: NEEDS-CHANGE
cases: executed 84→91, red 4
origin: introduced 5 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 logs in ~/.cache/uilab-wave-w4/widget-opens-at-use
needs-coordinator: whether a use in `page_kinds` should be checked, since ess makes pages inherit from their kind

**Cases** (`crates/uilab-doc/tests/adversary_widget_opens.rs`)

| case | now |
|---|---|
| `an_opens_bound_through_args_is_checked_against_the_bound_overlay` | red |
| `a_widget_body_finding_path_parses_and_resolves_as_a_node_path` | red |
| `a_recursive_widget_reached_first_inside_another_is_still_expanded_at_its_own_use` | red |
| `a_clean_metric_added_in_front_of_a_broken_one_is_not_refused_for_the_old_error` | red |
| `an_unresolved_body_opens_already_present_does_not_block_an_unrelated_insert` | green |
| `every_page_position_is_a_use_site_and_shell_or_batch_overlays_resolve` | green |
| `a_chain_doubling_at_each_of_twelve_levels_is_checked_in_bounded_time` | green |

```
left: [("page:overview/section:open_extend/body/go", "widget `opener` body node `go` opens `args.target`, which neither the page nor its shell declares")]
finding path `page:members/section:list/item:card/body/extend` does not select a node: Err(Malformed("page:members/section:list/item:card/body/extend", "a segment is not `<layer>:<name>`"))
a clean metric in front of the pre-existing error is refused: opens_resolves: page:members/header/metrics/1/body/extend: widget `loan_card` body node `extend` opens `extend`, which neither the page nor its shell declares
  left: ["page:members/section:list/item:lb/body/go"]
 right: ["page:members/section:list/item:lb/body/go", "page:members/section:list/item:la/body/b/body/go"]
depth 12: 4096 findings, check 5.916945ms, admit 164.409204ms
test result: FAILED. 3 passed; 4 failed
```

Suite: `cargo test -p uilab-doc --no-fail-fast` EXIT=101; only this file fails; clippy and fmt pass. Admit at depth 14: 2.6 s, depth 16: 73 s (debug).

```findings
- file: crates/uilab-doc/src/check.rs
  line: 669
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "an `opens: args.<param>` in a widget body is checked as the literal text instead of the bound overlay, so a correct instance is reported and refused by admit"
- file: crates/uilab-doc/src/check.rs
  line: 561
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "body finding paths such as `page:members/section:list/item:card/body/extend` fail NodePath parsing, so selecting the finding in the UI answers NotFound, and the brief's fallback to the instance path was not taken"
- file: crates/uilab-doc/src/check.rs
  line: 620
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the recursion guard caches a truncated expansion, so a use of loop_a after a use of loop_b is not reported and inserting it is admitted"
- file: crates/uilab-doc/src/check.rs
  line: 547
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "an unnamed list entry's index in the finding makes a pre-existing error look new after a clean metric is put in front of it, so admit refuses a clean replace"
- file: crates/uilab-doc/src/patch.rs
  line: 155
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "admit's quadratic `before.contains` over the exponential body expansion takes 73s at depth 16 (65536 findings) and 2.6s at depth 14 (debug build)"
- file: crates/uilab-doc/src/check.rs
  line: 539
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "a widget used in `page_kinds` reports nothing, while ess makes pages inherit sections, overlays and header from their kind; uilab models no page-kind inheritance for any check"
```

Coordinator decisions: F1–F5 go to round 2. F6 stays as is: uilab models no page-kind inheritance for any check, so checking one widget use through it would be the only such check; no change this story.
