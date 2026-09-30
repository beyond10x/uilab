---
format: aep.planning-md/3
id: story:canvas-preview-mode
kind: story
status: implemented
title: Canvas structure and preview modes
relations:
- decomposes: epic:ux-fidelity
- serves: vision:website-harness
scope:
- confidence: cited
  path: widget/src/App.vue
- confidence: cited
  path: widget/src/components/CanvasView.vue
- confidence: cited
  path: widget/src/components/CompositeView.vue
- confidence: cited
  path: widget/src/components/HelpModal.vue
- confidence: cited
  path: widget/src/lib/canvasmode.ts
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T12:08:58Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T12:08:58Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T14:59:18Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome

The canvas has two modes, toggled with `p`: structure (today: each composite labelled `name · kind · view`, shell regions as chips) and preview, which hides the labels and draws shell regions as app chrome (account menu as the staff member's name from `staff.Me`, notifications as a bell, navigation as it is). Selection and proposal marks work in both.

## Found by

Screenshot `views-1-canvas.png`: every card carries a debug label line; the header shows `account account_menu`, `overlay overlay_outlet`, `notify notifications` as chips, so the canvas reads as a tree dump rather than the app.

## Acceptance

- `p` toggles the mode outside text fields; the mode is listed in help and kept in localStorage.
- lib test for the mode toggle and its persistence.
- Screenshot of both modes recorded as evidence.
