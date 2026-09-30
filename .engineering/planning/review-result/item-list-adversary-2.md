---
format: aep.planning-md/3
id: review-result:item-list-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:item-list
relations:
- reviews: story:item-list
revision: 1
---
unit: story:item-list pass 2 on unit/item-list, findings cover 9b1e617 (my test commit e8b4630 on top)
verdict: CONFIRMED
cases: executed 52→55, red 1
origin: introduced 0 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 paths under ~/.cache/uilab-wave-w2/item-list, all deleted (part 6)
needs-coordinator: yes. Is the pre-existing widget-body opens gap routed to its own story?

The correction holds: my pass-1 red case is now green, and I found no regression in finding paths, docs use sites, duplicate findings or speed. One red case remains, and it is pre-existing. You asked whether widget-body primitives should go unchecked by `opens_resolves`. The answer is no: ess says to check them at each use site, and the base did not do that either.

**1. Diff stat** (`git --no-pager diff --stat 9b1e617 HEAD`)
```
 crates/uilab-doc/tests/adversary_item_list_p2.rs | 222 +++++++++++++++++++++++++
 1 file changed, 222 insertions(+)
```
Only a test file changed. Committed as e8b4630 through `b10x-gates bot`. Author and committer are both b10x-bot[bot].

**2. Cases added** (in `crates/uilab-doc/tests/adversary_item_list_p2.rs`)

| case | asserts | now |
|---|---|---|
| `an_opens_in_a_widget_body_is_checked_at_each_use_site` | `loan_card` has a body button with `opens: extend`. Used on `overview`, which declares `extend`, it is clean. Inserted as an item on `members`, which does not, it must produce an `opens_resolves` finding or be refused | **red** |
| `a_fault_in_a_primitive_node_is_reported_once_and_a_use_site_listed_once` | no finding appears twice. `opens_resolves` and `widget_resolves` fire once each at `item:go`, and `widget_resolves` once at `component:state_badge/node:more`. The docs list the `item:pick` use site once | green |
| `checking_a_large_document_stays_fast` | 3000 item nodes and a chain of 200 widgets: `check` under 5 s, `admit` under 10 s, and exactly 1500 `opens_resolves` findings | green (0.82 s for the whole file) |

Red output from running this file alone, before the suite:
```
thread 'an_opens_in_a_widget_body_is_checked_at_each_use_site' panicked at crates/uilab-doc/tests/adversary_item_list_p2.rs:107:5:
loan_card is used on page `members`, whose body button opens `extend`, which neither page `members` nor its shell declares; no opens_resolves finding and the insert is admitted
test result: FAILED. 2 passed; 1 failed
```

**3. Suite runs, after the cases existed**

| command | result |
|---|---|
| `cargo test -p uilab-doc --no-fail-fast` | EXIT=101. adversary_item_list 6 ok (pass-1 red case now green); adversary_item_list_p2 2 ok / 1 failed; adversary_widget_model 9 ok; adversary_widget_model_p2 4 ok; doc 33 ok. The 52 before my file comes from this run's other binaries |
| `cargo check --workspace --all-targets` | EXIT=0 |
| `cargo test -p uilab-agent -p uilab-behaviour --no-fail-fast` | EXIT=0, 43 passed |
| `cargo fmt -p uilab-doc --check`, `cargo clippy -p uilab-doc --all-targets -- -D warnings` | clean |

**4. Findings** (against 9b1e617)

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/uilab-doc/src/check.rs:687 | CONFIRMED | pre-existing | ess Widget says it is "expanded at its use site and then checked like a built-in", and WidgetInstance.expansion says findings are reported at `<instance path>/body/<node>`. uilab never checks a widget body's `opens` (composite or primitive) against the page where the widget is used | an agent inserting a widget instance on a page without that overlay; `admit` accepts it |

How I established pre-existing: I exported base 8b337a1 into scratch with `git archive` and ran a probe (a widget body with a button and a record, both `opens: extend`, used on a page without `extend`). It got no finding. The tree was never moved.

Suggested fix: run `opens_resolve` over the body nodes of each instance, using the overlays of the page where the instance sits.

**5. Attacked and not broken**
- `composites()` is now a filter over `nodes()`: order and callers (docs.rs:18, fixtures.rs:132 and :200, uilab-agent lib.rs:874) are unchanged.
- The two walks don't overlap: `nodes` reads only typed nodes, and a primitive held in props is read only through its composite.
- Finding paths and the `action/choice` trail at primitives.
- `uses()` now covers primitive props and board-widget items, so recursion is found through a primitive's `choice`.
- Speed on the large document.

**6. Written outside the worktree** (all deleted)
- `/home/timo/.cache/uilab-wave-w2/item-list/base/`: the base export plus the origin probe and its own target (121M)

I also used the build dir `/home/timo/.cache/b10x-target/uilab-w2-item-list`, as assigned. My lease `adversary-item-list-p2` is released. Disk is at 12G free on `/`, close to the 10G stop line.

**7. Findings block**
```findings
- file: crates/uilab-doc/src/check.rs
  line: 687
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "a widget body's opens is never checked at its use site, so an instance on a page without that overlay is admitted, against ess WidgetInstance expansion"
```
