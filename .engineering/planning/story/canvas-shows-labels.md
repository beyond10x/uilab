---
format: aep.planning-md/3
id: story:canvas-shows-labels
kind: story
status: implemented
title: The canvas shows the labels ESS gives a node
relations:
- serves: vision:website-harness
- decomposes: epic:ess-ui-adoption
scope:
- confidence: cited
  path: widget/src
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T15:50:40Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-01T15:50:40Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-01T16:50:17Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":14}}}
---
## Outcome

The canvas shows what ESS gives a metric: its `label` ("Copies on loan" on the library overview) above the value, in preview and structure mode.

## Found by

Docs screenshots, 2026-10-01 (story:essui-docs second half): the overview's `on_loan` metric is drawn without its label.

## Acceptance

- Widget test `metric.label.test.ts`: a metric section with `label` renders that text; without one, nothing is invented.
- Every other composite field ESS lets the author write as visible text (`label` on a field, a column, an action; `title` on a page and an overlay) is checked the same way; any the canvas drops is listed in the unit report and fixed.
- `task check` exits 0.

## Scope

- widget/src/** (components and lib), widget tests
