---
format: aep.planning-md/3
id: review-result:canvas-widget-instances-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:canvas-widget-instances
relations:
- reviews: story:canvas-widget-instances
revision: 1
---
unit: story:canvas-widget-instances, unit/canvas-widget-instances at 9e69f41 (adversary tests at a2d389c)
verdict: NEEDS-CHANGE
cases: executed 275→291, red 4
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/uilab-wave-w8/canvas-widget-instances/adversary/
needs-coordinator: none

Cases: `widget/src/lib/instance.adversary.test.ts` and `instance.canvas.adversary.test.ts` (args.<param> on a page instance — red; an empty loaded collection draws no instance item — red; a primitive item never shows raw `row.<field>` in structure and preview — red ×2; nine green: rows.<n> edges, prototype fields, mutual recursion, undeclared widget, depth-3 pass-through, proposal on the body, preview labels, draft views, 50 rows × nested).

Browser probe: `name Robin Example Kim Sample Sam Placeholder row.standing row.standing row.standing` on a page with no widget instance, in both modes.

```findings
- file: widget/src/components/CompositeView.vue
  line: 213
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a primitive collection or record item writing row.<field> is now drawn once per row with the unresolved reference as its text (\"row.standing row.standing row.standing\"), on pages with no widget instance, in structure and preview, where main drew one placeholder card"
- file: widget/src/lib/instance.ts
  line: 134
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "itemScopes yields one row-less scope for a loaded empty collection, so an instance item is drawn once with sample data under \"no rows\", against the acceptance \"one per row\""
- file: widget/src/lib/instance.ts
  line: 96
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "bindArg binds args.<param> written on a page instance as the literal string, so the body shows the field name instead of the sample, while the server's is_reference treats args.* as a reference with no holder on a page"
```

Coordinator decisions for round 2: fix all three. Primitive items fill `row.<field>` from each row (the same substitution the instance body uses); an empty loaded collection draws no item (a not-yet-loaded one keeps its loading line); `args.*` on a page instance is a reference with no holder and falls back to the sample.
