---
format: aep.planning-md/3
id: review-result:replace-drops-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:replace-drops
relations:
- reviews: story:replace-drops
revision: 1
---
unit: story:replace-drops, branch unit/replace-drops. Findings cover the implementation at 0a61776; my cases are at 3e0a842.
verdict: NEEDS-CHANGE
cases: executed 61→67, red 4
origin: introduced 3 / pre-existing 1 / undecided 0
wrote-outside-worktree: 4 paths (3 logs and the assigned build dir)
needs-coordinator: yes. The proposal card never shows the warning (finding 4), and `/` had 9.4G free, which is under the 10G build line.

**1. Diff since 0a61776**
```
 crates/uilab-doc/tests/adversary_replace_drops.rs | 204 ++++++++++++++++++++++
 1 file changed, 204 insertions(+)
```
It touches one test file and no other path. I committed it as `b10x-bot[bot]` (both author and committer).

**2. Cases added.** File: `crates/uilab-doc/tests/adversary_replace_drops.rs`. The red output below is from running that file alone, before the suite:

| case | asserts | now |
|---|---|---|
| `a_string_field_rewritten_as_an_object_with_the_same_field_is_not_a_drop` (:53) | changing `fields: [due]` to `[{field: due, label: "Due date"}]` warns nothing | red |
| `a_batch_whose_later_replace_restores_the_columns_warns_nothing` (:77) | replace dropping 3 columns, then a replace restoring them, warns nothing | red |
| `an_insert_then_a_replace_without_it_names_nothing_the_document_had` (:106) | insert overlay, then a page replace without it, warns nothing | red |
| `one_of_two_columns_over_the_same_field_dropped_is_named` (:127) | keeping 1 of 2 `standing` columns warns "drops columns standing" | red |
| `a_drop_deep_under_children_that_stay_is_named_at_its_node` | a drop four levels down (page, section, widget, item) is named at the item | green |
| `reordering_and_changing_as_warn_nothing` | reordered columns, a changed `as`, reordered menu pages warn nothing | green |

```
left: [("page:loans/overlay:edit", "replace at page:loans/overlay:edit drops fields due")]   right: []
left: []   right: [("page:members/section:list", "replace at page:members/section:list drops columns standing")]
left: [("page:members", "replace at page:members drops overlay edit_member")]   right: []
left: [("page:members/section:list", "replace at page:members/section:list drops columns joined, loans, standing")]   right: []
test result: FAILED. 2 passed; 4 failed
```

**3. Suite:** `cargo test -p uilab-doc --no-fail-fast` exits 101. My file fails 4 of 6. The other test files (the two adversary_item_list, the two adversary_widget_model, and doc.rs) pass 61 of 61, and that 61 is the "before" count. Without `--no-fail-fast`, cargo stops at my file and doc.rs never runs.

**4. Findings**

| # | file:line | finding | what reaches it | verdict | origin |
|---|---|---|---|---|---|
| 1 | crates/uilab-doc/src/check.rs:209 | `identity` counts the kind of key, so the string `due` and `{field: due}` look like different entries. The renderer (`outline.ts` `fieldsOf`) and `fixtures.rs` treat them as the same field. Result: a false "drops fields due". Fix: compare the value only, and treat a string entry as its field. | the shipped example writes `fields: [due]`, and adding a label means writing the object form. Not seen in a live run. | NEEDS-CHANGE | introduced |
| 2 | crates/uilab-doc/src/patch.rs:176 | Inside a batch, each replace is compared with the step just before it, not with the stored document. It then names things the result still has, or never had. Fix: compare the stored doc with the final doc at each replace target, skipping names the stored doc lacks. | I built both batches myself. Agent batches seen so far are "a drawer plus a row action". | INFEASIBLE | introduced |
| 3 | crates/uilab-doc/src/check.rs:174 | Entries are matched with `contains`, which ignores how many there are, so dropping 1 of 2 columns over the same field goes unreported. Fix: match each entry once, then remove it from the list. | nothing found | INFEASIBLE | introduced |
| 4 | crates/uilab-app/src/app.rs:1108 | `admit(..)` findings are thrown away (`_`). The proposal card shows `check(after)` plus `field_findings` (:1115), so `replace_drops` never reaches the operator. The story's Outcome says "sees it on the proposal card". The behaviour crate (lib.rs:297) and the agent (lib.rs:425) also throw them away. This is outside the unit's surface and needs a follow-up story. | every proposal goes through `record()` | NEEDS-CHANGE | pre-existing |

**5. Attacked and could not break:**
- `admit` never refuses because of this warning: it is added after the error comparison (patch.rs:162).
- `check()` never reports it: every case in my file asserts this.
- A drop deep under children that stay is found.
- Reordering, changing `as`, and reordering the menu warn nothing.
- The message format and paths match the brief.
- Findings in a batch come out in patch order.
- A batch inside a batch is refused before `drops` runs.

**6. Written outside the worktree**
- `/home/timo/.cache/uilab-wave-w3/replace-drops/adversary-red.log`
- `/home/timo/.cache/uilab-wave-w3/replace-drops/adversary-suite.log`
- `/home/timo/.cache/uilab-wave-w3/replace-drops/adversary-suite-nff.log`
- `/home/timo/.cache/b10x-target/uilab-w3-replace-drops` (the assigned build dir; incremental builds only)

```findings
- file: crates/uilab-doc/src/check.rs
  line: 209
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "identity keys on the key name, so a string entry and an object with the same field are counted as a drop although the renderer and fixtures.rs treat them as one field"
- file: crates/uilab-doc/src/patch.rs
  line: 176
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "in a batch each replace is measured against the step before it, so it names things the admitted document still has or never had"
- file: crates/uilab-doc/src/check.rs
  line: 174
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "entry matching ignores counts, so dropping one of two columns over the same field is not reported"
- file: crates/uilab-app/src/app.rs
  line: 1108
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "record() discards admit's findings and shows check(after), so replace_drops never reaches the proposal card the story outcome names"
```
