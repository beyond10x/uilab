---
format: aep.planning-md/3
id: review-result:components-workspace-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:components-workspace
relations:
- reviews: story:components-workspace
revision: 1
---
unit: story:components-workspace, head 2111de9 plus adversary commit fcd4e86 (tests only)
verdict: NEEDS-CHANGE
cases: executed 238→241 (widget 96→98, cargo uilab-doc+uilab-app 142→143), red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w3/components-workspace/adversary2/ (4 logs)
needs-coordinator: yes. F1 needs either full documents when uses change or the widget computing uses; that spans app.rs and store.ts.

**Cases**

| file | asserts | red output |
|---|---|---|
| `widget/src/lib/components.delta.adversary.test.ts:91` | after a `changed` Insert of `section:more` (kind `loan_card`), `loan_card` lists both uses | `page:overview/section:more` missing |
| `widget/src/lib/components.delta.adversary.test.ts:119` | after a `changed` Remove of the only instance, `loan_card` has no uses | actual `['page:overview/section:latest']` |
| `crates/uilab-app/tests/adversary_components_workspace.rs:185` | every widget's outline uses equal the docs' "Used at" list, one per instance | `loan_card`: outline 3 entries, docs 4 |

Suite: `pnpm check` tests 98, fail 2, EXIT=1; `cargo test -p uilab-doc -p uilab-app --no-fail-fast` one failing binary, EXIT=101.

Attacked and held: page_kinds, header, board widgets, items, an item `choice`, widget-body instances and one path with two trails all reach `uses` in document order; `widget_uses` runs once per outline call.

```findings
- file: widget/src/store.ts
  line: 268
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A changed delta for a section, overlay, item or widget re-sends only that subtree, so props.uses on component nodes keeps the previous revision's use sites until a full document arrives. An inserted instance is missing from the Components tab and a removed one is still listed."
- file: crates/uilab-doc/src/outline.rs
  line: 52
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "uses_of drops entries with the same path and trail, which widget_uses only produces for distinct instances (two header metrics with the same name), so the tab counts fewer uses than /api/docs.md lists."
```

Coordinator correction 764c73c: `announce_change` sends a full document when the component nodes' uses differ before and after the accept; `uses_of` keeps one entry per instance. The two widget delta cases assumed a delta the server no longer sends for such a change; they are replaced by the rig test `a_change_that_adds_or_drops_a_widget_instance_sends_the_whole_outline` (insert → document with both uses, remove → document with one, an unrelated remove → still a delta). Verified: `cargo test -p uilab-app -p uilab-doc --no-fail-fast` every binary ok (app 62, adversary file 3); clippy and fmt exit 0; `pnpm check` 96/96; `pnpm build` ok.
