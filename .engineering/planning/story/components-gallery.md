---
format: aep.planning-md/3
id: story:components-gallery
kind: story
status: implemented
title: Components tab as a gallery with realistic previews
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget/src/components/ComponentsView.vue
- confidence: cited
  path: widget/src/components/PrimitiveView.vue
- confidence: cited
  path: widget/src/lib/components.ts
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T12:08:57Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T12:39:37Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome

The Components tab reads as a gallery: each widget card leads with its preview, filled with realistic sample values, and keeps params and use sites in a collapsed details block. A use site is a link that shows that page.

## Found by

Screenshot `views-4-components.png`: previews show the param and field names (`member`, `standing`, `joined`) instead of values; a one-row params table spans the full width above the preview.

## Acceptance

- Sample args for a param whose type names an entity use the first fixture row of a view whose rows carry that entity (e.g. `Member` → `members.All`), falling back to today's name-shaped values.
- lib tests in `components.test.ts` for the fixture-backed sample args and the fallback.
- Clicking a use site shows its page in the UI tab.
