---
format: aep.planning-md/3
id: review-result:canvas-widget-instances-adversary-2
kind: review-result
status: active
title: Adversary pass 2 on story:canvas-widget-instances
relations:
- reviews: story:canvas-widget-instances
revision: 1
---
unit: story:canvas-widget-instances, unit/canvas-widget-instances at 15b9cbc (adversary tests at 668c2ad)
verdict: NEEDS-CHANGE
cases: executed 302→326, red 10
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w8/canvas-widget-instances/adversary2/ (6 files)
needs-coordinator: whether a composite item's row.<field> title is in this unit's scope

Cases: `widget/src/lib/instance.adversary2.test.ts` (rowNode matches previewNode over 22 inputs; titles; no mutation — green) and `instance.canvas.adversary2.test.ts` (composite item title `row.name` per row, instance item title, primitive item `rows.first.name`, section and board instances over loaded-empty rows — red in both modes; record 0/2 rows, disconnected, draft, chart over no rows, proposal adding a primitive item, pages without instances or items — green).

```findings
- file: widget/src/lib/instance.ts
  line: 116
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a section or board widget instance reading rows.first of a loaded view with no rows draws its body from sample args (\"member standing\") with neither the empty line nor the sample-data tag, in structure and preview"
- file: widget/src/components/CompositeView.vue
  line: 213
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "rowNode fills only row.<field>, so a primitive item writing rows.first.<field> prints \"first: rows.first.name\" once per row where main drew a placeholder"
- file: widget/src/components/CompositeView.vue
  line: 124
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "a composite or instance item's title row.<field> is never read from its row; main printed the raw \"row.name\" once, the unit now prints it once per row"
```

Coordinator correction: rowNode reads `rows.first|<n>.<field>` with the item's rows; composite and instance items go through rowNode; a section or board instance whose `rows.*` args have nothing to read shows the empty line (`row` outside a collection keeps the sample, as pass 1's case requires). Verified: `pnpm check` 326 (325 pass, 1 todo, 0 fail), `pnpm build` ok.
