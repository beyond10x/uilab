---
format: aep.planning-md/3
id: story:canvas-widget-instances
kind: story
status: active
title: Widget instances render their body on the canvas
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget/src/components/CanvasView.vue
- confidence: cited
  path: widget/src/components/CompositeView.vue
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:08:12Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T13:08:12Z", actor: "human:timo", revision: 4}
---
## Outcome

A widget instance on a page renders its widget's body on the canvas with the instance's args bound (rows for `row`/`rows.…`, sample args otherwise), the way the Components tab previews it, instead of a hatched placeholder box named after the widget. Selecting the instance selects its node; the body is not separately selectable on the page.

## Found by

components-gallery unit, wave 2026-09-30 w5: `page:members/section:spotlight: {component: member_card, args: {member: rows.first}}` draws as a hatched "member_card" box in the UI tab (`CompositeView.vue`), while the Components tab renders the same widget.

## Acceptance

- lib test: resolving an instance's body with its args (reuse `widget/src/lib/components.ts` substitution).
- The canvas shows the body for an instance in a section, a board widget and a collection item (one per row).
- Screenshot recorded as evidence.
