---
format: aep.planning-md/3
id: review-result:replace-drops-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:replace-drops
relations:
- reviews: story:replace-drops
revision: 1
---
unit: story:replace-drops, branch unit/replace-drops. Findings cover 89d2c46; the cases are at 87159c0.
verdict: INFEASIBLE (1 finding: the correction brings in a false warning in a batch shape built by the adversary)
cases: executed 69→75, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths (logs, a p2-probe copy of the crate, the assigned build dir)
needs-coordinator: yes. Is a warning right when a `remove` in the batch took the child?

**Cases** (`crates/uilab-doc/tests/adversary_replace_drops_p2.rs`)

| case | now |
|---|---|
| `a_remove_then_a_replace_of_the_parent_does_not_blame_the_replace` | red |
| `a_replace_then_a_remove_of_its_child_does_not_blame_the_replace` | red |
| `a_remove_elsewhere_does_not_hide_what_the_replace_dropped` | green |
| `the_same_target_replaced_twice_is_judged_once` | green |
| `items_stored_as_a_map_are_matched_against_a_list` | green |
| `an_insert_then_a_replace_of_the_new_node_names_nothing` | green |

```
left: [("page:overview", "replace at page:overview drops section recent")]   right: []
left: [("page:overview", "replace at page:overview drops section recent")]   right: []
test result: FAILED. 4 passed; 2 failed
```

Mutant check: `j < i` → `j != i` at patch.rs:187, and removing that guard, are both caught by the same-target case.

Attacked and held: F1 identity by value, F3 multisets, F2 nested and repeated targets, admit never refuses, check() never reports it, the check-table patch fixture.

```findings
- file: crates/uilab-doc/src/patch.rs
  line: 171
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "comparing the stored doc with the final doc names a child removed by an explicit remove in the same batch as 'replace at <parent> drops', in either order"
```

Coordinator correction: 9c5f564 applies the batch's removes to the stored side before comparing. `cargo test -p uilab-doc --no-fail-fast` then passes every lane (41 doc, 6 p2, 6 p1, 6+3 item list, 9+4 widget model); clippy and fmt exit 0.
