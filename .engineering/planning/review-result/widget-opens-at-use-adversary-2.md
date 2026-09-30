---
format: aep.planning-md/3
id: review-result:widget-opens-at-use-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:widget-opens-at-use
relations:
- reviews: story:widget-opens-at-use
revision: 1
---
unit: story:widget-opens-at-use, 8cf170a plus adversary commit 3e26df2 on unit/widget-opens-at-use
verdict: NEEDS-CHANGE
cases: executed 95→101, red 4
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 files in ~/.cache/uilab-wave-w4/widget-opens-at-use, plus the assigned build dir
needs-coordinator: F1 conflicts with the implementor's case adversary_item_list_p2.rs:318-340, which asserts a pass-through reports nothing

**Cases** (`crates/uilab-doc/tests/adversary_widget_opens_p2.rs`)

| case | now | red output |
|---|---|---|
| `a_pass_through_bound_to_a_literal_at_the_outer_use_is_checked_against_that_literal` | red | ``relay bound to `nowhere` on members: []`` |
| `a_pass_through_of_an_outer_default_is_checked_against_that_default` | red | `relay_default on members: []` |
| `a_second_broken_unnamed_instance_beside_an_equal_one_is_refused` | red | `a second broken loan_card metric was admitted` |
| `a_recursive_leaf_under_a_doubling_chain_is_checked_in_bounded_time` | red | `depth 22: check 26.825584057s` |
| `a_runtime_reference_default_is_skipped_and_a_bound_literal_the_page_lacks_is_reported` | green | |
| `a_doubling_chain_without_recursion_is_checked_in_bounded_time` | green | `control depth 22: check 1.530225ms` |

Suite: `cargo test -p uilab-doc --no-fail-fast` EXIT=101, only this file (2 passed, 4 failed); clippy and fmt pass. The rewritten pass-1 cases still assert what pass 1 meant.

```findings
- file: crates/uilab-doc/src/check.rs
  line: 750
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a nested instance binding `args.<param>` of its holder is dropped as a runtime reference, so a literal or default bound at the outer use reports nothing and admit accepts it, against ess substitute-then-validate"
- file: crates/uilab-doc/src/patch.rs
  line: 158
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "index-free keys make two unnamed instances of one widget yield equal findings, and admit's before/after check only asks whether the finding existed, so a second broken loan_card header metric is admitted"
- file: crates/uilab-doc/src/check.rs
  line: 693
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a self-recursive leaf stops every ancestor's expansion from being cached, so check grows 2^depth with nothing to report: 26.8 s at depth 22 against 1.5 ms without recursion (debug build)"
```

Coordinator correction 04617c1: `Opened::bind` passes a bound `args.<param>` through as the holder's param; admit counts before-findings (a second equal error is new); an expansion is cached unless it stopped at a widget expanded around it. The implementor's nested-args case now expects the pass-through finding at the outer use. Verified: `cargo test -p uilab-doc --no-fail-fast` every binary ok (p2 adversary file 6/6, item_list_p2 12/12, doc 41); clippy (forced re-check) and fmt exit 0; `cargo test -p uilab-behaviour` ok.
